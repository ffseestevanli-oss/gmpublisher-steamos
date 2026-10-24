//! Publishing to and reading from the Steam Workshop. Every function here blocks, so call them from a
//! worker thread; Steam's replies arrive on the callback thread that `steam::connect` runs.

use std::{
	fs,
	path::{Path, PathBuf},
	sync::mpsc,
	thread,
	time::Duration,
};

use gmpublisher_core::{pack, AddonJson};
use serde::Serialize;
use steamworks::{AppIDs, Client, FileType, PublishedFileId, PublishedFileVisibility, UGCType, UpdateStatus, UserList, UserListOrder};

use crate::steam::{self, GMOD_APP_ID};

const DEFAULT_PREVIEW: &[u8] = include_bytes!("../assets/default_preview.png");
const PREVIEW_MAX_BYTES: u64 = 1024 * 1024;
const STEAM_TIMEOUT: Duration = Duration::from_secs(60);

fn connected() -> Result<Client, String> {
	steam::client().ok_or_else(|| "Steam isn't connected. Start Steam and sign in, then try again.".to_owned())
}

fn wait<T>(rx: mpsc::Receiver<T>) -> Result<T, String> {
	rx.recv_timeout(STEAM_TIMEOUT).map_err(|_| "Steam didn't answer in time.".to_owned())
}

/// Parses public, friends, unlisted or private. Anything else means "leave it as it is".
pub fn visibility(name: &str) -> Option<PublishedFileVisibility> {
	match name {
		"public" => Some(PublishedFileVisibility::Public),
		"friends" => Some(PublishedFileVisibility::FriendsOnly),
		"unlisted" => Some(PublishedFileVisibility::Unlisted),
		"private" => Some(PublishedFileVisibility::Private),
		_ => None,
	}
}

fn visibility_name(visibility: &PublishedFileVisibility) -> &'static str {
	match visibility {
		PublishedFileVisibility::Public => "public",
		PublishedFileVisibility::FriendsOnly => "friends",
		PublishedFileVisibility::Unlisted => "unlisted",
		PublishedFileVisibility::Private => "private",
	}
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyItem {
	pub id: String,
	pub title: String,
	pub tags: Vec<String>,
	pub visibility: &'static str,
	pub updated: u32,
	pub size: u32,
	pub preview_url: String,
	pub upvotes: u32,
	pub downvotes: u32,
}

/// The signed-in user's published Garry's Mod addons, most recently updated first.
pub fn my_items() -> Result<Vec<MyItem>, String> {
	let client = connected()?;
	let account = client.user().steam_id().account_id();
	let mut items = Vec::new();

	for page in 1.. {
		let (tx, rx) = mpsc::channel();
		client
			.ugc()
			.query_user(
				account,
				UserList::Published,
				UGCType::Items,
				UserListOrder::LastUpdatedDesc,
				AppIDs::Both { creator: GMOD_APP_ID, consumer: GMOD_APP_ID },
				page,
			)
			.map_err(|_| "Steam couldn't start the Workshop query.".to_owned())?
			.fetch(move |result| {
				let _ = tx.send(result.map(|results| {
					let page: Vec<MyItem> = results
						.iter()
						.enumerate()
						.filter_map(|(i, item)| {
							let item = item?;
							Some(MyItem {
								id: item.published_file_id.0.to_string(),
								title: item.title,
								tags: item.tags,
								visibility: visibility_name(&item.visibility),
								updated: item.time_updated,
								size: item.file_size,
								preview_url: results.preview_url(i as u32).unwrap_or_default(),
								upvotes: item.num_upvotes,
								downvotes: item.num_downvotes,
							})
						})
						.collect();
					(page, results.total_results())
				}));
			});
		let (page_items, total) = wait(rx)?.map_err(|err| format!("Steam couldn't list your Workshop items: {err}"))?;
		let empty = page_items.is_empty();
		items.extend(page_items);
		if empty || items.len() >= total as usize {
			break;
		}
	}
	Ok(items)
}

pub struct PublishRequest {
	pub folder: PathBuf,
	pub addon: AddonJson,
	/// Update this item instead of creating a new one.
	pub existing: Option<u64>,
	pub preview: Option<PathBuf>,
	/// None keeps an existing item's visibility. New items default to private.
	pub visibility: Option<PublishedFileVisibility>,
	pub change_note: String,
}

pub struct Published {
	pub id: u64,
	pub needs_legal_agreement: bool,
}

fn check_preview(path: &Path) -> Result<(), String> {
	let ext = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).unwrap_or_default();
	if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "gif") {
		return Err("The preview image must be a PNG, JPEG or GIF.".into());
	}
	let size = fs::metadata(path).map_err(|err| format!("Couldn't read the preview image: {err}"))?.len();
	if size > PREVIEW_MAX_BYTES {
		return Err("The preview image must be 1 MiB or smaller.".into());
	}
	Ok(())
}

/// Packs the addon and uploads it. `status` receives a description of the current step and its
/// progress from 0 to 1 (or a negative number when the step's length isn't known).
pub fn publish(req: PublishRequest, mut status: impl FnMut(&str, f64)) -> Result<Published, String> {
	let client = connected()?;
	if let Some(problem) = req.addon.problems().first() {
		return Err(problem.clone());
	}
	if let Some(preview) = &req.preview {
		check_preview(preview)?;
	}

	// Steam uploads a whole folder, so the GMA goes into an empty one of its own
	let work_dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join(format!("gmpublisher/publish-{}", std::process::id()));
	let _ = fs::remove_dir_all(&work_dir);
	let content_dir = work_dir.join("content");
	fs::create_dir_all(&content_dir).map_err(|err| format!("Couldn't create {}: {err}", content_dir.display()))?;
	let result = (|| {
		status("Packing", 0.0);
		let plan = pack::plan(&req.folder, &req.addon).map_err(|err| err.to_string())?;
		pack::write(&plan, &req.addon, content_dir.join("addon.gma"), |done, total| {
			status("Packing", done as f64 / total.max(1) as f64);
			true
		})
		.map_err(|err| err.to_string())?;

		let preview = match (&req.preview, req.existing) {
			(Some(path), _) => Some(path.clone()),
			// New items need a preview, so use gmpublisher's icon until the user picks one
			(None, None) => {
				let path = work_dir.join("preview.png");
				fs::write(&path, DEFAULT_PREVIEW).map_err(|err| err.to_string())?;
				Some(path)
			}
			(None, Some(_)) => None,
		};

		let (id, mut needs_legal_agreement, created) = match req.existing {
			Some(id) => (PublishedFileId(id), false, false),
			None => {
				status("Creating the Workshop item", -1.0);
				let (tx, rx) = mpsc::channel();
				client.ugc().create_item(GMOD_APP_ID, FileType::Community, move |result| {
					let _ = tx.send(result);
				});
				let (id, legal) = wait(rx)?.map_err(|err| format!("Steam couldn't create the Workshop item: {err}"))?;
				(id, legal, true)
			}
		};

		let uploaded = upload(&client, id, &req, &content_dir, preview.as_deref(), created, &mut status);
		match uploaded {
			Ok(legal) => {
				needs_legal_agreement |= legal;
				Ok(Published { id: id.0, needs_legal_agreement })
			}
			Err(err) => {
				// Don't leave an empty item behind on the Workshop
				if created {
					client.ugc().delete_item(id, |_| {});
				}
				Err(err)
			}
		}
	})();
	let _ = fs::remove_dir_all(&work_dir);
	result
}

fn upload(
	client: &Client,
	id: PublishedFileId,
	req: &PublishRequest,
	content_dir: &Path,
	preview: Option<&Path>,
	created: bool,
	status: &mut impl FnMut(&str, f64),
) -> Result<bool, String> {
	let mut tags: Vec<&str> = vec!["Addon", &req.addon.addon_type];
	tags.extend(req.addon.tags.iter().map(String::as_str));

	let mut update = client
		.ugc()
		.start_item_update(GMOD_APP_ID, id)
		.content_path(content_dir)
		.tags(tags, false);
	match (&req.visibility, created) {
		(Some(visibility), _) => update = update.visibility(*visibility),
		(None, true) => update = update.visibility(PublishedFileVisibility::Private),
		(None, false) => {}
	}
	if created {
		update = update.title(&req.addon.title).description(if req.addon.description.trim().is_empty() {
			"Published from Linux with [url=https://github.com/ffseestevanli-oss/gmpublisher-steamos]gmpublisher[/url]."
		} else {
			&req.addon.description
		});
	}
	if let Some(preview) = preview {
		update = update.preview_path(preview);
	}

	let (tx, rx) = mpsc::channel();
	let note = Some(req.change_note.trim()).filter(|n| !n.is_empty());
	let watch = update.submit(note, move |result| {
		let _ = tx.send(result);
	});

	let started = std::time::Instant::now();
	loop {
		match rx.try_recv() {
			Ok(result) => {
				return result.map(|(_, legal)| legal).map_err(|err| format!("Steam rejected the upload: {err}"));
			}
			Err(mpsc::TryRecvError::Disconnected) => return Err("Steam dropped the upload.".into()),
			Err(mpsc::TryRecvError::Empty) => {}
		}
		let (stage, done, total) = watch.progress();
		let label = match stage {
			UpdateStatus::PreparingConfig => "Preparing",
			UpdateStatus::PreparingContent => "Preparing the files",
			UpdateStatus::UploadingContent => "Uploading",
			UpdateStatus::UploadingPreviewFile => "Uploading the preview image",
			UpdateStatus::CommittingChanges => "Finishing",
			UpdateStatus::Invalid => "Waiting for Steam",
		};
		status(label, if total > 0 { done as f64 / total as f64 } else { -1.0 });
		// Uploads can be large, so only give up if Steam stops reporting anything for a long time
		if matches!(stage, UpdateStatus::Invalid) && started.elapsed() > STEAM_TIMEOUT * 10 {
			return Err("Steam didn't finish the upload.".into());
		}
		thread::sleep(Duration::from_millis(200));
	}
}

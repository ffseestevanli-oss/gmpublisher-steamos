//! Publishing to and reading from the Steam Workshop. Every function here blocks, so call them from a
//! worker thread; Steam's replies arrive on the callback thread that `steam::connect` runs.

use std::{
	fs,
	path::{Path, PathBuf},
	sync::mpsc,
	thread,
	time::{Duration, Instant},
};

use gmpublisher_core::{addon_json, pack, AddonJson, Gma};
use serde::Serialize;
use steamworks::{
	AppIDs, Client, DownloadItemResult, FileType, ItemState, PublishedFileId, PublishedFileVisibility, UGCType, UpdateStatus, UserList,
	UserListOrder,
};

use crate::steam::{self, GMOD_APP_ID};

const DEFAULT_PREVIEW: &[u8] = include_bytes!("../assets/default_preview.png");
const PREVIEW_MAX_BYTES: u64 = 1024 * 1024;
const STEAM_TIMEOUT: Duration = Duration::from_secs(60);
// Steam's limits for Workshop titles (characters) and descriptions (bytes)
const TITLE_MAX: usize = 128;
const DESCRIPTION_MAX: usize = 8000;

fn connected() -> Result<Client, String> {
	steam::client().ok_or_else(|| "Steam isn't connected. Start Steam and sign in, then try again.".to_owned())
}

fn wait<T>(rx: mpsc::Receiver<T>) -> Result<T, String> {
	rx.recv_timeout(STEAM_TIMEOUT).map_err(|_| "Steam didn't answer in time.".to_owned())
}

/// Reads a Workshop item ID from a bare number or a link such as `...filedetails/?id=123456`.
pub fn parse_id(id_or_url: &str) -> Option<u64> {
	let text = id_or_url.trim();
	let id = text.rsplit("id=").next().unwrap_or(text).split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
	id.parse().ok().filter(|&id| id != 0)
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
	pub description: String,
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
			// Without this Steam cuts descriptions off at 255 bytes, and editing would lose the rest
			.set_return_long_description(true)
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
								description: item.description,
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

/// What to upload as the item's files.
pub enum Content {
	/// Leave the files on the Workshop as they are. Only for existing items.
	Keep,
	/// Pack an addon folder, using `addon` (its addon.json) for the ignore list and the GMA's details.
	Folder { folder: PathBuf, addon: AddonJson },
	/// Upload a GMA that is already packed.
	Gma(PathBuf),
}

pub struct PublishRequest {
	/// Update this item instead of creating a new one.
	pub existing: Option<u64>,
	pub content: Content,
	/// None keeps an existing item's title. New items fall back to the addon's own title.
	pub title: Option<String>,
	/// None keeps an existing item's description.
	pub description: Option<String>,
	/// The addon type and its tags. None keeps an existing item's tags (new items take them from the addon).
	pub tags: Option<(String, Vec<String>)>,
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

fn check_tags((addon_type, tags): &(String, Vec<String>)) -> Result<(), String> {
	if !addon_json::TYPES.contains(&addon_type.as_str()) {
		return Err("Choose the addon's type.".into());
	}
	if tags.len() > 2 {
		return Err("Garry's Mod addons can have at most two tags.".into());
	}
	Ok(())
}

/// Packs or copies the content (if any) and uploads it with the requested changes. `status`
/// receives a description of the current step and its progress from 0 to 1 (or a negative number
/// when the step's length isn't known).
pub fn publish(req: PublishRequest, mut status: impl FnMut(&str, f64)) -> Result<Published, String> {
	let client = connected()?;

	// Check everything before anything is created on the Workshop
	let gma = match &req.content {
		Content::Keep if req.existing.is_none() => return Err("Choose the addon folder or GMA to publish.".into()),
		Content::Keep => None,
		Content::Folder { addon, .. } => {
			if let Some(problem) = addon.problems().first() {
				return Err(problem.clone());
			}
			None
		}
		Content::Gma(path) => Some(Gma::open(path).map_err(|err| format!("Couldn't open {}: {err}", path.display()))?),
	};
	let addon = match &req.content {
		Content::Folder { addon, .. } => Some(addon.clone()),
		Content::Gma(_) => gma.as_ref().and_then(Gma::addon_json),
		Content::Keep => None,
	};
	let title = match (&req.title, req.existing) {
		(Some(title), _) => Some(title.trim().to_owned()),
		(None, None) => addon.as_ref().map(|a| a.title.clone()).filter(|t| !t.trim().is_empty()).or_else(|| gma.as_ref().map(|g| g.title.clone())),
		(None, Some(_)) => None,
	};
	if let Some(title) = &title {
		if title.is_empty() {
			return Err("Give the addon a title.".into());
		}
		if title.chars().count() > TITLE_MAX {
			return Err(format!("The title is too long. Steam allows {TITLE_MAX} characters."));
		}
	}
	let description = match (&req.description, req.existing) {
		(Some(description), _) => Some(description.clone()),
		(None, None) => Some(addon.as_ref().map(|a| a.description.clone()).unwrap_or_default()),
		(None, Some(_)) => None,
	};
	if description.as_ref().is_some_and(|d| d.len() > DESCRIPTION_MAX) {
		return Err(format!("The description is too long. Steam allows {DESCRIPTION_MAX} bytes."));
	}
	let tags = match (&req.tags, req.existing) {
		(Some(tags), _) => Some(tags.clone()),
		(None, None) => addon.as_ref().map(|a| (a.addon_type.clone(), a.tags.clone())),
		(None, Some(_)) => None,
	};
	match &tags {
		Some(tags) => check_tags(tags)?,
		None if req.existing.is_none() => return Err("Choose the addon's type.".into()),
		None => {}
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
		let content = match (&req.content, &gma) {
			(Content::Folder { folder, addon }, _) => {
				status("Packing", 0.0);
				let plan = pack::plan(folder, addon).map_err(|err| err.to_string())?;
				pack::write(&plan, addon, content_dir.join("addon.gma"), |done, total| {
					status("Packing", done as f64 / total.max(1) as f64);
					true
				})
				.map_err(|err| err.to_string())?;
				Some(content_dir.as_path())
			}
			(Content::Gma(_), Some(gma)) => {
				// Also unpacks compressed downloads, since Garry's Mod expects a plain GMA
				status("Copying the GMA", -1.0);
				gma.save_as(content_dir.join("addon.gma")).map_err(|err| format!("Couldn't copy the GMA: {err}"))?;
				Some(content_dir.as_path())
			}
			_ => None,
		};

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

		let changes = Changes {
			content,
			title: title.as_deref(),
			description: description.as_deref(),
			tags: tags.as_ref(),
			preview: preview.as_deref(),
			visibility: req.visibility.or(created.then_some(PublishedFileVisibility::Private)),
			change_note: &req.change_note,
		};
		match upload(&client, id, changes, &mut status) {
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

/// What one item update sends to Steam. Every None is left as it is.
struct Changes<'a> {
	content: Option<&'a Path>,
	title: Option<&'a str>,
	description: Option<&'a str>,
	tags: Option<&'a (String, Vec<String>)>,
	preview: Option<&'a Path>,
	visibility: Option<PublishedFileVisibility>,
	change_note: &'a str,
}

fn upload(client: &Client, id: PublishedFileId, changes: Changes, status: &mut impl FnMut(&str, f64)) -> Result<bool, String> {
	let mut update = client.ugc().start_item_update(GMOD_APP_ID, id);
	if let Some(content) = changes.content {
		update = update.content_path(content);
	}
	if let Some((addon_type, tags)) = changes.tags {
		let mut all: Vec<&str> = vec!["Addon", addon_type];
		all.extend(tags.iter().map(String::as_str));
		update = update.tags(all, false);
	}
	if let Some(visibility) = changes.visibility {
		update = update.visibility(visibility);
	}
	if let Some(title) = changes.title {
		update = update.title(title);
	}
	if let Some(description) = changes.description {
		update = update.description(if description.trim().is_empty() {
			"Published from Linux with [url=https://github.com/ffseestevanli-oss/gmpublisher-steamos]gmpublisher[/url]."
		} else {
			description
		});
	}
	if let Some(preview) = changes.preview {
		update = update.preview_path(preview);
	}

	let (tx, rx) = mpsc::channel();
	let note = Some(changes.change_note.trim()).filter(|n| !n.is_empty());
	let watch = update.submit(note, move |result| {
		let _ = tx.send(result);
	});

	let started = Instant::now();
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

pub struct Downloaded {
	pub title: String,
	/// The GMA (or compressed `.bin`) in Steam's Workshop folder.
	pub gma: PathBuf,
}

/// Has Steam download a Workshop item, unless it is already installed and up to date, and finds
/// the GMA it installed.
pub fn download(id: u64, mut status: impl FnMut(&str, f64)) -> Result<Downloaded, String> {
	let client = connected()?;
	let ugc = client.ugc();
	let file = PublishedFileId(id);

	status("Looking up the item", -1.0);
	let (tx, rx) = mpsc::channel();
	ugc.query_item(file)
		.map_err(|_| "Steam couldn't start the Workshop query.".to_owned())?
		.fetch(move |result| {
			let _ = tx.send(result.map(|results| results.get(0)));
		});
	let item = wait(rx)?
		.map_err(|err| format!("Steam couldn't look up item {id}: {err}"))?
		.ok_or_else(|| format!("There's no Workshop item {id}, or it isn't visible to you."))?;
	if item.consumer_app_id.is_some_and(|app| app != GMOD_APP_ID) {
		return Err(format!("Workshop item {id} isn't a Garry's Mod addon."));
	}

	let busy = ItemState::NEEDS_UPDATE | ItemState::DOWNLOADING | ItemState::DOWNLOAD_PENDING;
	let ready = |state: ItemState| state.contains(ItemState::INSTALLED) && !state.intersects(busy);
	if !ready(ugc.item_state(file)) {
		let (tx, rx) = mpsc::channel();
		let _finished = client.register_callback(move |result: DownloadItemResult| {
			if result.published_file_id == file {
				let _ = tx.send(result.error);
			}
		});
		if !ugc.download_item(file, true) {
			return Err("Steam wouldn't start the download.".into());
		}

		let started = Instant::now();
		let mut last_progress = (Instant::now(), (0, 0));
		loop {
			match rx.try_recv() {
				Ok(None) => break,
				Ok(Some(err)) => return Err(format!("Steam couldn't download item {id}: {err}")),
				Err(_) if started.elapsed() > Duration::from_secs(1) && ready(ugc.item_state(file)) => break,
				Err(_) => {}
			}
			let progress = ugc.item_download_info(file).unwrap_or_default();
			if progress != last_progress.1 {
				last_progress = (Instant::now(), progress);
			}
			let (done, total) = progress;
			status("Downloading", if total > 0 { done as f64 / total as f64 } else { -1.0 });
			if last_progress.0.elapsed() > STEAM_TIMEOUT * 5 {
				return Err("The download stopped. Check Steam's Downloads page, then try again.".into());
			}
			thread::sleep(Duration::from_millis(200));
		}
	}

	let info = ugc.item_install_info(file).ok_or("Steam downloaded the item but didn't say where it put it.")?;
	let gma = installed_gma(Path::new(&info.folder)).ok_or_else(|| format!("Couldn't find the GMA Steam downloaded into {}.", info.folder))?;
	Ok(Downloaded { title: item.title, gma })
}

/// The addon file in an item's install folder. Legacy items are installed as a single file.
fn installed_gma(folder: &Path) -> Option<PathBuf> {
	if folder.is_file() {
		return Some(folder.to_owned());
	}
	let mut files: Vec<PathBuf> = fs::read_dir(folder).ok()?.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
	files.sort_by_key(|path| match path.extension().and_then(|e| e.to_str()) {
		Some("gma") => 0,
		Some("bin") => 1,
		_ => 2,
	});
	files.into_iter().next()
}

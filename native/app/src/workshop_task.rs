/// Creating, editing and downloading Workshop items, one job at a time.
#[cxx_qt::bridge]
pub mod qobject {
	unsafe extern "C++" {
		include!("cxx-qt-lib/qstring.h");
		type QString = cxx_qt_lib::QString;
	}

	#[auto_cxx_name]
	extern "RustQt" {
		#[qobject]
		#[qml_element]
		#[qproperty(bool, busy)]
		#[qproperty(QString, status)]
		// 0 to 1, or negative while the length of the step isn't known
		#[qproperty(f64, progress)]
		#[qproperty(QString, error)]
		// Set when an item was created or edited
		#[qproperty(QString, published_id)]
		#[qproperty(bool, created)]
		#[qproperty(bool, needs_legal_agreement)]
		// Set when a download was saved: the .gma file, or the folder it was extracted into
		#[qproperty(QString, saved_to)]
		#[qproperty(bool, extracted)]
		type WorkshopTask = super::WorkshopTaskRust;

		// Creates or edits an item from a JSON object (see EditRequest).
		#[qinvokable]
		fn submit(self: Pin<&mut WorkshopTask>, request_json: &QString);

		// Downloads an item by ID or link and saves its .gma into `dest_dir`, or extracts it into a
		// new folder there.
		#[qinvokable]
		fn download(self: Pin<&mut WorkshopTask>, id_or_url: &QString, dest_dir: &QString, extract: bool);

		#[qinvokable]
		fn is_workshop_id(self: &WorkshopTask, id_or_url: &QString) -> bool;

		#[qinvokable]
		fn default_download_dir(self: &WorkshopTask) -> QString;

		// Reads an addon folder's addon.json (or a GMA's) as JSON {title, type, tags, description},
		// so the editor can fill itself in. Empty when there's nothing to read.
		#[qinvokable]
		fn read_addon(self: &WorkshopTask, path: &QString) -> QString;

		#[qinvokable]
		fn clear(self: Pin<&mut WorkshopTask>);

		// JSON arrays of the Workshop's addon types and tags
		#[qinvokable]
		fn addon_types(self: &WorkshopTask) -> QString;

		#[qinvokable]
		fn addon_tags(self: &WorkshopTask) -> QString;
	}

	impl cxx_qt::Threading for WorkshopTask {}
}

use core::pin::Pin;
use std::path::{Path, PathBuf};

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use gmpublisher_core::{addon_json, AddonJson, Gma};
use serde::Deserialize;
use serde_json::json;

use crate::{cli, settings, workshop};

#[derive(Default)]
pub struct WorkshopTaskRust {
	busy: bool,
	status: QString,
	progress: f64,
	error: QString,
	published_id: QString,
	created: bool,
	needs_legal_agreement: bool,
	saved_to: QString,
	extracted: bool,
}

/// What the item editor sends. Fields left out are kept as they are on the Workshop.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EditRequest {
	/// Empty to create a new item.
	#[serde(default)]
	id: String,
	title: Option<String>,
	/// The title shown in the editor, sent even when unchanged, for the GMA packed from a folder.
	gma_title: Option<String>,
	description: Option<String>,
	#[serde(rename = "type")]
	addon_type: Option<String>,
	tags: Option<Vec<String>>,
	#[serde(default)]
	visibility: String,
	#[serde(default)]
	preview: String,
	/// "keep", "folder" or "gma"
	content: String,
	#[serde(default)]
	content_path: String,
	#[serde(default)]
	change_note: String,
}

impl EditRequest {
	fn into_publish(self) -> Result<(workshop::PublishRequest, Option<String>), String> {
		let existing = match self.id.trim() {
			"" => None,
			id => Some(workshop::parse_id(id).ok_or("That isn't a Workshop item ID or link.")?),
		};
		let tags = self.addon_type.map(|t| (t, self.tags.unwrap_or_default()));
		let mut folder = None;
		let content = match self.content.as_str() {
			"keep" => workshop::Content::Keep,
			"gma" if !self.content_path.is_empty() => workshop::Content::Gma(PathBuf::from(&self.content_path)),
			"folder" if !self.content_path.is_empty() => {
				let path = PathBuf::from(&self.content_path);
				let mut addon = AddonJson::load(&path)
					.map_err(|err| format!("Couldn't read addon.json: {err}"))?
					.unwrap_or_default();
				// The GMA gets the same details as the Workshop page, even when the folder has no
				// addon.json (or one without a title) and the title isn't being changed
				if let Some(title) = self.title.as_ref().or(self.gma_title.as_ref()).filter(|t| !t.trim().is_empty()) {
					addon.title = title.trim().to_owned();
				}
				if let Some((addon_type, tags)) = &tags {
					addon.addon_type = addon_type.clone();
					addon.tags = tags.clone();
				}
				folder = Some(self.content_path.clone());
				workshop::Content::Folder { folder: path, addon }
			}
			_ => return Err("Choose the addon folder or GMA to upload.".into()),
		};
		let request = workshop::PublishRequest {
			existing,
			content,
			title: self.title,
			description: self.description,
			tags,
			preview: Some(self.preview).filter(|p| !p.is_empty()).map(PathBuf::from),
			visibility: workshop::visibility(&self.visibility),
			change_note: self.change_note,
		};
		Ok((request, folder))
	}
}

/// A name for a file or folder made from an item's title, or its ID if the title won't do.
fn file_name(title: &str, id: u64) -> String {
	let name: String = title.chars().map(|c| if c == '/' || c.is_control() { '_' } else { c }).collect();
	let name = name.trim().trim_start_matches('.').trim();
	if name.is_empty() {
		id.to_string()
	} else {
		name.chars().take(100).collect()
	}
}

impl qobject::WorkshopTask {
	fn clear(mut self: Pin<&mut Self>) {
		if *self.busy() {
			return;
		}
		self.as_mut().set_error(QString::default());
		self.as_mut().set_published_id(QString::default());
		self.as_mut().set_created(false);
		self.as_mut().set_needs_legal_agreement(false);
		self.as_mut().set_saved_to(QString::default());
		self.set_extracted(false);
	}

	/// Clears the last result and shows the job as started. False if a job is already running.
	fn begin(mut self: Pin<&mut Self>) -> bool {
		if *self.busy() {
			return false;
		}
		self.as_mut().clear();
		self.as_mut().set_status(QString::from("Starting"));
		self.as_mut().set_progress(-1.0);
		self.set_busy(true);
		true
	}

	/// A status callback for the worker thread that only passes on visible changes, in 1% steps.
	fn status_reporter(&self) -> impl FnMut(&str, f64) {
		let thread = self.qt_thread();
		let mut last = (String::new(), -2i64);
		move |stage: &str, progress: f64| {
			let step = (progress * 100.0) as i64;
			if last.0 != stage || last.1 != step {
				last = (stage.to_owned(), step);
				let stage = stage.to_owned();
				let _ = thread.queue(move |mut task| {
					task.as_mut().set_status(QString::from(stage.as_str()));
					task.set_progress(progress);
				});
			}
		}
	}

	fn finish(mut self: Pin<&mut Self>, error: Option<String>) {
		self.as_mut().set_busy(false);
		self.as_mut().set_status(QString::default());
		if let Some(error) = error {
			self.set_error(QString::from(error.as_str()));
		}
	}

	fn submit(mut self: Pin<&mut Self>, request_json: &QString) {
		let parsed = serde_json::from_str::<EditRequest>(&request_json.to_string())
			.map_err(|err| format!("Bad request from the editor: {err}"))
			.and_then(EditRequest::into_publish);
		if !self.as_mut().begin() {
			return;
		}
		let (request, folder) = match parsed {
			Ok(parsed) => parsed,
			Err(err) => return self.finish(Some(err)),
		};
		let creating = request.existing.is_none();

		let thread = self.qt_thread();
		let status = self.status_reporter();
		std::thread::spawn(move || {
			let result = workshop::publish(request, status);
			let _ = thread.queue(move |mut task| match result {
				Ok(published) => {
					// Remember which item the folder publishes to, for the folder's own page
					if let Some(folder) = &folder {
						settings::set_workshop_id(folder, Some(published.id));
					}
					task.as_mut().set_published_id(QString::from(published.id.to_string().as_str()));
					task.as_mut().set_created(creating);
					task.as_mut().set_needs_legal_agreement(published.needs_legal_agreement);
					task.finish(None);
				}
				Err(err) => task.finish(Some(err)),
			});
		});
	}

	fn download(mut self: Pin<&mut Self>, id_or_url: &QString, dest_dir: &QString, extract: bool) {
		let id = workshop::parse_id(&id_or_url.to_string());
		let dest_dir = PathBuf::from(dest_dir.to_string());
		if !self.as_mut().begin() {
			return;
		}
		let Some(id) = id else {
			return self.finish(Some("That isn't a Workshop item ID or link.".into()));
		};
		if !dest_dir.is_dir() {
			return self.finish(Some(format!("{} isn't a folder.", dest_dir.display())));
		}

		let thread = self.qt_thread();
		let mut status = self.status_reporter();
		std::thread::spawn(move || {
			let result = (|| {
				let downloaded = workshop::download(id, &mut status)?;
				status("Reading the GMA", -1.0);
				let gma = Gma::open(&downloaded.gma).map_err(|err| format!("Couldn't open {}: {err}", downloaded.gma.display()))?;
				let name = file_name(&downloaded.title, id);
				if extract {
					let target = cli::unused_path(&dest_dir, &name, "");
					gma.extract(&target, |done, total| {
						status("Extracting", done as f64 / total.max(1) as f64);
						true
					})
					.map_err(|err| format!("Extraction failed: {err}"))?;
					Ok(target)
				} else {
					status("Saving the GMA", -1.0);
					let target = cli::unused_path(&dest_dir, &name, "gma");
					gma.save_as(&target).map_err(|err| format!("Couldn't save {}: {err}", target.display()))?;
					Ok(target)
				}
			})();
			let _ = thread.queue(move |mut task| match result {
				Ok(path) => {
					task.as_mut().set_saved_to(QString::from(path.to_string_lossy().as_ref()));
					task.as_mut().set_extracted(extract);
					task.finish(None);
				}
				Err(err) => task.finish(Some(err)),
			});
		});
	}

	fn is_workshop_id(&self, id_or_url: &QString) -> bool {
		workshop::parse_id(&id_or_url.to_string()).is_some()
	}

	fn default_download_dir(&self) -> QString {
		let dir = dirs::download_dir().or_else(dirs::home_dir).unwrap_or_else(|| PathBuf::from("/"));
		QString::from(dir.to_string_lossy().as_ref())
	}

	fn addon_types(&self) -> QString {
		QString::from(json!(addon_json::TYPES).to_string().as_str())
	}

	fn addon_tags(&self) -> QString {
		QString::from(json!(addon_json::TAGS).to_string().as_str())
	}

	fn read_addon(&self, path: &QString) -> QString {
		let path = PathBuf::from(path.to_string());
		let addon = if path.is_dir() { AddonJson::load(&path).ok().flatten() } else { read_gma_addon(&path) };
		match addon {
			Some(a) => QString::from(
				json!({ "title": a.title, "type": a.addon_type, "tags": a.tags, "description": a.description }).to_string().as_str(),
			),
			None => QString::default(),
		}
	}
}

fn read_gma_addon(path: &Path) -> Option<AddonJson> {
	let gma = Gma::open(path).ok()?;
	let mut addon = gma.addon_json().unwrap_or_default();
	if addon.title.trim().is_empty() {
		addon.title = gma.title;
	}
	Some(addon)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn updating_from_a_folder_without_addon_json_keeps_the_title() {
		let folder = std::env::temp_dir().join(format!("gmpublisher-edit-test-{}", std::process::id()));
		std::fs::create_dir_all(&folder).unwrap();
		let request: EditRequest = serde_json::from_value(json!({
			"id": "123",
			"gmaTitle": "testie1",
			"type": "tool",
			"tags": ["fun"],
			"content": "folder",
			"contentPath": folder.to_string_lossy(),
		}))
		.unwrap();
		let (request, _) = request.into_publish().unwrap();
		std::fs::remove_dir_all(&folder).ok();

		// The Workshop title isn't touched, but the packed GMA still has one
		assert!(request.title.is_none());
		let workshop::Content::Folder { addon, .. } = request.content else { panic!("expected a folder") };
		assert_eq!(addon.title, "testie1");
		assert!(addon.problems().is_empty());
	}
}

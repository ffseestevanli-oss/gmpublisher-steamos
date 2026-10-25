/// An addon folder being edited and packed.
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
		#[qproperty(QString, folder)]
		#[qproperty(QString, title)]
		#[qproperty(QString, addon_type)]
		// JSON array of tag names
		#[qproperty(QString, tags_json)]
		// One ignore pattern per line
		#[qproperty(QString, ignore_text)]
		#[qproperty(bool, has_addon_json)]
		// JSON array of messages that would stop the addon from being accepted
		#[qproperty(QString, problems_json)]
		// JSON {files: [{path, size}], notAllowed: [..], ignored: [..], totalSize}
		#[qproperty(QString, plan_json)]
		#[qproperty(bool, busy)]
		#[qproperty(f64, progress)]
		#[qproperty(QString, error)]
		#[qproperty(QString, packed_to)]
		// JSON arrays of the Workshop's addon types and tags
		#[qproperty(QString, types_json)]
		#[qproperty(QString, all_tags_json)]
		// Workshop description, used when the item is first published
		#[qproperty(QString, description)]
		// The Workshop item this folder publishes to, empty for a new one
		#[qproperty(QString, workshop_id)]
		#[qproperty(bool, publishing)]
		#[qproperty(QString, publish_status)]
		// 0 to 1, or negative while the length of the step isn't known
		#[qproperty(f64, publish_progress)]
		#[qproperty(QString, published_id)]
		#[qproperty(bool, needs_legal_agreement)]
		type AddonProject = super::AddonProjectRust;

		#[qinvokable]
		fn load(self: Pin<&mut AddonProject>, folder: &QString);

		// Writes addon.json from the current properties and refreshes the file list.
		#[qinvokable]
		fn save(self: Pin<&mut AddonProject>);

		#[qinvokable]
		fn refresh(self: Pin<&mut AddonProject>);

		#[qinvokable]
		fn pack(self: Pin<&mut AddonProject>, out: &QString);

		#[qinvokable]
		fn default_output(self: &AddonProject) -> QString;

		// Packs and uploads to the Workshop. visibility is public, friends, unlisted or private;
		// an empty preview keeps the current image (or uses the default for a new item).
		// update_details also replaces an existing item's title and description with this folder's.
		#[qinvokable]
		fn publish(self: Pin<&mut AddonProject>, visibility: &QString, preview: &QString, change_note: &QString, update_details: bool);

		// Links this folder to an existing Workshop item, from its ID or URL. Empty unlinks it.
		#[qinvokable]
		fn link_workshop_item(self: Pin<&mut AddonProject>, id_or_url: &QString) -> bool;
	}

	impl cxx_qt::Threading for AddonProject {}
}

use core::pin::Pin;
use std::path::PathBuf;

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use gmpublisher_core::{addon_json, pack, AddonJson};

use crate::{settings, workshop};
use serde_json::json;

pub struct AddonProjectRust {
	folder: QString,
	title: QString,
	addon_type: QString,
	tags_json: QString,
	ignore_text: QString,
	has_addon_json: bool,
	problems_json: QString,
	plan_json: QString,
	busy: bool,
	progress: f64,
	error: QString,
	packed_to: QString,
	types_json: QString,
	all_tags_json: QString,
	description: QString,
	workshop_id: QString,
	publishing: bool,
	publish_status: QString,
	publish_progress: f64,
	published_id: QString,
	needs_legal_agreement: bool,
}

impl Default for AddonProjectRust {
	fn default() -> Self {
		Self {
			folder: QString::default(),
			title: QString::default(),
			addon_type: QString::default(),
			tags_json: QString::from("[]"),
			ignore_text: QString::default(),
			has_addon_json: false,
			problems_json: QString::from("[]"),
			plan_json: QString::from("{}"),
			busy: false,
			progress: 0.0,
			error: QString::default(),
			packed_to: QString::default(),
			types_json: QString::from(json!(addon_json::TYPES).to_string().as_str()),
			all_tags_json: QString::from(json!(addon_json::TAGS).to_string().as_str()),
			description: QString::default(),
			workshop_id: QString::default(),
			publishing: false,
			publish_status: QString::default(),
			publish_progress: 0.0,
			published_id: QString::default(),
			needs_legal_agreement: false,
		}
	}
}

impl qobject::AddonProject {
	fn folder_path(&self) -> PathBuf {
		PathBuf::from(self.folder().to_string())
	}

	fn current(&self) -> AddonJson {
		AddonJson {
			title: self.title().to_string().trim().to_owned(),
			addon_type: self.addon_type().to_string(),
			tags: serde_json::from_str(&self.tags_json().to_string()).unwrap_or_default(),
			ignore: self.ignore_text().to_string().lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned).collect(),
			description: self.description().to_string(),
		}
	}

	fn load(mut self: Pin<&mut Self>, folder: &QString) {
		self.as_mut().set_folder(folder.clone());
		self.as_mut().set_error(QString::default());
		self.as_mut().set_packed_to(QString::default());

		let (addon, has_json) = match AddonJson::load(self.folder_path()) {
			Ok(Some(addon)) => (addon, true),
			Ok(None) => {
				// Suggest the folder name as a title for a new addon
				let title = self.folder_path().file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
				(AddonJson { title, ..Default::default() }, false)
			}
			Err(err) => {
				self.as_mut().set_error(QString::from(format!("Couldn't read addon.json: {err}").as_str()));
				(AddonJson::default(), true)
			}
		};
		self.as_mut().set_has_addon_json(has_json);
		self.as_mut().set_title(QString::from(addon.title.as_str()));
		self.as_mut().set_addon_type(QString::from(addon.addon_type.as_str()));
		self.as_mut().set_tags_json(QString::from(json!(addon.tags).to_string().as_str()));
		self.as_mut().set_description(QString::from(addon.description.as_str()));
		let linked = settings::workshop_id(&folder.to_string()).map(|id| id.to_string()).unwrap_or_default();
		self.as_mut().set_workshop_id(QString::from(linked.as_str()));
		self.as_mut().set_ignore_text(QString::from(addon.ignore.join("\n").as_str()));
		self.refresh();
	}

	fn save(mut self: Pin<&mut Self>) {
		match self.current().save(self.folder_path()) {
			Ok(()) => {
				self.as_mut().set_error(QString::default());
				self.as_mut().set_has_addon_json(true);
			}
			Err(err) => self.as_mut().set_error(QString::from(format!("Couldn't save addon.json: {err}").as_str())),
		}
		self.refresh();
	}

	fn refresh(mut self: Pin<&mut Self>) {
		let addon = self.current();
		self.as_mut().set_problems_json(QString::from(json!(addon.problems()).to_string().as_str()));

		let folder = self.folder_path();
		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let plan = match pack::plan(&folder, &addon) {
				Ok(plan) => json!({
					"files": plan.files.iter().map(|(path, disk)| json!({
						"path": path,
						"size": std::fs::metadata(disk).map(|m| m.len()).unwrap_or(0),
					})).collect::<Vec<_>>(),
					"notAllowed": plan.not_allowed,
					"ignored": plan.ignored,
					"totalSize": plan.total_size,
				}),
				Err(err) => json!({ "error": err.to_string() }),
			};
			let _ = thread.queue(move |project| project.set_plan_json(QString::from(plan.to_string().as_str())));
		});
	}

	fn default_output(&self) -> QString {
		QString::from(self.folder_path().with_extension("gma").to_string_lossy().as_ref())
	}

	fn pack(mut self: Pin<&mut Self>, out: &QString) {
		if *self.busy() {
			return;
		}
		let addon = self.current();
		let folder = self.folder_path();
		let out = match out.to_string() {
			o if o.is_empty() => PathBuf::from(self.default_output().to_string()),
			o => PathBuf::from(o),
		};
		self.as_mut().set_error(QString::default());
		self.as_mut().set_packed_to(QString::default());
		self.as_mut().set_progress(0.0);
		self.as_mut().set_busy(true);

		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let progress_thread = thread.clone();
			let mut last_step = 0;
			let result = pack::plan(&folder, &addon).and_then(|plan| {
				pack::write(&plan, &addon, &out, |done, total| {
					let step = (done * 100).checked_div(total).unwrap_or(100);
					if step != last_step {
						last_step = step;
						let _ = progress_thread.queue(move |project| project.set_progress(step as f64 / 100.0));
					}
					true
				})
			});
			let _ = thread.queue(move |mut project| {
				project.as_mut().set_busy(false);
				match result {
					Ok(()) => project.set_packed_to(QString::from(out.to_string_lossy().as_ref())),
					Err(err) => project.set_error(QString::from(format!("Packing failed: {err}").as_str())),
				}
			});
		});
	}

	fn link_workshop_item(mut self: Pin<&mut Self>, id_or_url: &QString) -> bool {
		let text = id_or_url.to_string();
		let id = match (text.trim().is_empty(), workshop::parse_id(&text)) {
			(true, _) => None,
			(false, Some(id)) => Some(id),
			(false, None) => return false,
		};
		settings::set_workshop_id(&self.folder().to_string(), id);
		self.as_mut().set_workshop_id(QString::from(id.map(|id| id.to_string()).unwrap_or_default().as_str()));
		true
	}

	fn publish(mut self: Pin<&mut Self>, visibility: &QString, preview: &QString, change_note: &QString, update_details: bool) {
		if *self.publishing() {
			return;
		}
		let folder = self.folder().to_string();
		let addon = self.current();
		let details = update_details || self.workshop_id().is_empty();
		let request = workshop::PublishRequest {
			existing: self.workshop_id().to_string().parse().ok(),
			title: details.then(|| addon.title.clone()),
			description: details.then(|| addon.description.clone()),
			tags: Some((addon.addon_type.clone(), addon.tags.clone())),
			content: workshop::Content::Folder { folder: PathBuf::from(&folder), addon },
			preview: Some(preview.to_string()).filter(|p| !p.is_empty()).map(PathBuf::from),
			visibility: workshop::visibility(&visibility.to_string()),
			change_note: change_note.to_string(),
		};
		self.as_mut().set_error(QString::default());
		self.as_mut().set_published_id(QString::default());
		self.as_mut().set_needs_legal_agreement(false);
		self.as_mut().set_publish_status(QString::from("Starting"));
		self.as_mut().set_publish_progress(-1.0);
		self.as_mut().set_publishing(true);

		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let status_thread = thread.clone();
			let mut last = (String::new(), -2i64);
			let result = workshop::publish(request, |stage, progress| {
				// Only pass on visible changes, in 1% steps
				let step = (progress * 100.0) as i64;
				if last.0 != stage || last.1 != step {
					last = (stage.to_owned(), step);
					let stage = stage.to_owned();
					let _ = status_thread.queue(move |mut project| {
						project.as_mut().set_publish_status(QString::from(stage.as_str()));
						project.set_publish_progress(progress);
					});
				}
			});
			let _ = thread.queue(move |mut project| {
				project.as_mut().set_publishing(false);
				project.as_mut().set_publish_status(QString::default());
				match result {
					Ok(published) => {
						let id = published.id.to_string();
						settings::set_workshop_id(&folder, Some(published.id));
						project.as_mut().set_workshop_id(QString::from(id.as_str()));
						project.as_mut().set_needs_legal_agreement(published.needs_legal_agreement);
						project.set_published_id(QString::from(id.as_str()));
					}
					Err(err) => project.set_error(QString::from(format!("Publishing failed: {err}").as_str())),
				}
			});
		});
	}
}

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
	}

	impl cxx_qt::Threading for AddonProject {}
}

use core::pin::Pin;
use std::path::PathBuf;

use cxx_qt::Threading;
use cxx_qt_lib::QString;
use gmpublisher_core::{addon_json, pack, AddonJson};
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
			..Default::default()
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
		self.as_mut().set_ignore_text(QString::from(addon.ignore.join("\n").as_str()));
		self.refresh();
	}

	fn save(mut self: Pin<&mut Self>) {
		// Keep anything in addon.json this app doesn't edit, such as the Workshop description
		let existing = AddonJson::load(self.folder_path()).ok().flatten().unwrap_or_default();
		let addon = AddonJson { description: existing.description, ..self.current() };
		match addon.save(self.folder_path()) {
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
}

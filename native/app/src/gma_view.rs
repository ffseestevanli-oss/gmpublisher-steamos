/// One GMA opened for browsing and extraction.
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
		#[qproperty(QString, path)]
		#[qproperty(QString, title)]
		#[qproperty(QString, addon_type)]
		#[qproperty(QString, tags)]
		// JSON array of {path, size}
		#[qproperty(QString, entries_json)]
		#[qproperty(f64, content_size)]
		#[qproperty(i32, rejected_count)]
		#[qproperty(bool, loaded)]
		#[qproperty(bool, busy)]
		#[qproperty(f64, progress)]
		#[qproperty(QString, error)]
		// Where the last extraction went, for "Open folder"
		#[qproperty(QString, extracted_to)]
		type GmaDocument = super::GmaDocumentRust;

		#[qinvokable]
		fn open(self: Pin<&mut GmaDocument>, path: &QString);

		// Extracts into `dest`, or next to the GMA when `dest` is empty.
		#[qinvokable]
		fn extract(self: Pin<&mut GmaDocument>, dest: &QString);

		#[qinvokable]
		fn default_extract_dir(self: &GmaDocument) -> QString;
	}

	impl cxx_qt::Threading for GmaDocument {}
}

use core::pin::Pin;
use std::{
	path::{Path, PathBuf},
	sync::Arc,
};

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use gmpublisher_core::Gma;
use serde_json::json;

use crate::cli;

#[derive(Default)]
pub struct GmaDocumentRust {
	path: QString,
	title: QString,
	addon_type: QString,
	tags: QString,
	entries_json: QString,
	content_size: f64,
	rejected_count: i32,
	loaded: bool,
	busy: bool,
	progress: f64,
	error: QString,
	extracted_to: QString,
	gma: Option<Arc<Gma>>,
}

impl qobject::GmaDocument {
	fn open(mut self: Pin<&mut Self>, path: &QString) {
		self.as_mut().set_path(path.clone());
		self.as_mut().set_error(QString::default());
		self.as_mut().set_loaded(false);
		self.as_mut().set_busy(true);

		let path = PathBuf::from(path.to_string());
		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let result = Gma::open(&path);
			let _ = thread.queue(move |mut doc| {
				doc.as_mut().set_busy(false);
				match result {
					Ok(gma) => doc.show(gma),
					Err(err) => doc.set_error(QString::from(format!("Couldn't open {}: {err}", path.display()).as_str())),
				}
			});
		});
	}

	fn show(mut self: Pin<&mut Self>, gma: Gma) {
		let addon = gma.addon_json();
		let entries: Vec<_> = gma.entries.iter().map(|e| json!({ "path": e.path, "size": e.size })).collect();

		self.as_mut().set_title(QString::from(gma.title.as_str()));
		self.as_mut().set_addon_type(QString::from(addon.as_ref().map(|a| a.addon_type.as_str()).unwrap_or_default()));
		self.as_mut().set_tags(QString::from(addon.as_ref().map(|a| a.tags.join(", ")).unwrap_or_default().as_str()));
		self.as_mut().set_entries_json(QString::from(serde_json::Value::from(entries).to_string().as_str()));
		self.as_mut().set_content_size(gma.content_size() as f64);
		self.as_mut().set_rejected_count(gma.rejected.len() as i32);
		self.as_mut().rust_mut().gma = Some(Arc::new(gma));
		self.set_loaded(true);
	}

	fn default_extract_dir(&self) -> QString {
		QString::from(cli::default_extract_dir(Path::new(&self.path().to_string())).to_string_lossy().as_ref())
	}

	fn extract(mut self: Pin<&mut Self>, dest: &QString) {
		let Some(gma) = self.as_ref().rust().gma.clone() else { return };
		if *self.busy() {
			return;
		}
		let dest = match dest.to_string() {
			d if d.is_empty() => PathBuf::from(self.default_extract_dir().to_string()),
			d => PathBuf::from(d),
		};
		self.as_mut().set_error(QString::default());
		self.as_mut().set_extracted_to(QString::default());
		self.as_mut().set_progress(0.0);
		self.as_mut().set_busy(true);

		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let progress_thread = thread.clone();
			let mut last_step = 0;
			let result = gma.extract(&dest, |done, total| {
				// Report in 1% steps so the UI thread isn't flooded
				let step = (done * 100).checked_div(total).unwrap_or(100);
				if step != last_step {
					last_step = step;
					let _ = progress_thread.queue(move |doc| doc.set_progress(step as f64 / 100.0));
				}
				true
			});
			let _ = thread.queue(move |mut doc| {
				doc.as_mut().set_busy(false);
				match result {
					Ok(_) => doc.set_extracted_to(QString::from(dest.to_string_lossy().as_ref())),
					Err(err) => doc.set_error(QString::from(format!("Extraction failed: {err}").as_str())),
				}
			});
		});
	}
}

/// App-wide state for QML: the Steam connection, installed addons and remembered addon folders.
#[cxx_qt::bridge]
pub mod qobject {
	unsafe extern "C++" {
		include!("cxx-qt-lib/qstring.h");
		type QString = cxx_qt_lib::QString;
	}

	// Expose properties to QML in camelCase (steamStatus, gmodDir, ...)
	#[auto_cxx_name]
	extern "RustQt" {
		#[qobject]
		#[qml_element]
		#[qproperty(QString, steam_status)]
		#[qproperty(QString, persona)]
		#[qproperty(QString, gmod_dir)]
		#[qproperty(i32, subscribed_items)]
		#[qproperty(bool, connected)]
		// JSON array of installed addons (see library::InstalledAddon)
		#[qproperty(QString, installed_json)]
		#[qproperty(bool, scanning)]
		// JSON array of remembered addon folder paths
		#[qproperty(QString, projects_json)]
		// A file to open as soon as the window is up, from the command line
		#[qproperty(QString, open_at_start)]
		#[qproperty(QString, open_folder_at_start)]
		// JSON array of the user's published Workshop items (see workshop::MyItem)
		#[qproperty(QString, my_items_json)]
		#[qproperty(bool, loading_my_items)]
		#[qproperty(QString, my_items_error)]
		type Backend = super::BackendRust;

		#[qinvokable]
		fn start(self: Pin<&mut Backend>);

		#[qinvokable]
		fn rescan(self: Pin<&mut Backend>);

		#[qinvokable]
		fn add_project(self: Pin<&mut Backend>, folder: &QString);

		#[qinvokable]
		fn remove_project(self: Pin<&mut Backend>, folder: &QString);

		#[qinvokable]
		fn refresh_my_items(self: Pin<&mut Backend>);
	}

	impl cxx_qt::Threading for Backend {}
}

use core::pin::Pin;
use std::path::PathBuf;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use crate::{gmod, library, settings, steam, workshop};

pub struct BackendRust {
	steam_status: QString,
	persona: QString,
	gmod_dir: QString,
	subscribed_items: i32,
	connected: bool,
	installed_json: QString,
	scanning: bool,
	projects_json: QString,
	open_at_start: QString,
	open_folder_at_start: QString,
	my_items_json: QString,
	loading_my_items: bool,
	my_items_error: QString,
	started: bool,
}

impl Default for BackendRust {
	fn default() -> Self {
		Self {
			steam_status: QString::from("Connecting to Steam…"),
			persona: QString::default(),
			gmod_dir: QString::from(gmod::locate().map(|p| p.display().to_string()).unwrap_or_default().as_str()),
			subscribed_items: -1,
			connected: false,
			installed_json: QString::from("[]"),
			scanning: false,
			projects_json: QString::from(json(&settings::projects()).as_str()),
			open_at_start: QString::from(crate::OPEN_AT_START.get().map(String::as_str).unwrap_or_default()),
			open_folder_at_start: QString::from(crate::OPEN_FOLDER_AT_START.get().map(String::as_str).unwrap_or_default()),
			my_items_json: QString::from("[]"),
			loading_my_items: false,
			my_items_error: QString::default(),
			started: false,
		}
	}
}

fn json<T: serde::Serialize + ?Sized>(value: &T) -> String {
	serde_json::to_string(value).unwrap_or_else(|_| "[]".into())
}

impl qobject::Backend {
	fn start(mut self: Pin<&mut Self>) {
		if self.as_ref().rust().started {
			return;
		}
		self.as_mut().rust_mut().started = true;
		self.as_mut().rescan();

		let waiting = self.qt_thread();
		let connected = self.qt_thread();
		steam::connect(
			move || {
				let _ = waiting.queue(|backend| {
					backend.set_steam_status(QString::from("Waiting for Steam. Make sure Steam is running and you're signed in."));
				});
			},
			move |info| {
				let _ = connected.queue(move |mut backend| {
					backend.as_mut().set_steam_status(QString::from("Connected"));
					backend.as_mut().set_persona(QString::from(format!("{} ({})", info.persona, info.steam_id).as_str()));
					backend.as_mut().set_subscribed_items(info.subscribed_items as i32);
					let found_gmod = backend.gmod_dir().is_empty() && info.gmod_dir.is_some();
					if let Some(dir) = info.gmod_dir {
						backend.as_mut().set_gmod_dir(QString::from(dir.display().to_string().as_str()));
					}
					backend.as_mut().set_connected(true);
					if found_gmod {
						backend.rescan();
					}
				});
			},
		);
	}

	fn rescan(mut self: Pin<&mut Self>) {
		if *self.scanning() {
			return;
		}
		self.as_mut().set_scanning(true);
		let gmod_dir = Some(PathBuf::from(self.gmod_dir().to_string())).filter(|p| !p.as_os_str().is_empty());
		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let installed = json(&library::scan(gmod_dir.as_ref()));
			let _ = thread.queue(move |mut backend| {
				backend.as_mut().set_installed_json(QString::from(installed.as_str()));
				backend.set_scanning(false);
			});
		});
	}

	fn add_project(self: Pin<&mut Self>, folder: &QString) {
		let mut projects = settings::projects();
		let folder = folder.to_string();
		if !folder.is_empty() && !projects.contains(&folder) {
			projects.push(folder);
		}
		self.save_projects(projects);
	}

	fn remove_project(self: Pin<&mut Self>, folder: &QString) {
		let folder = folder.to_string();
		let projects = settings::projects().into_iter().filter(|p| *p != folder).collect();
		self.save_projects(projects);
	}

	fn refresh_my_items(mut self: Pin<&mut Self>) {
		if *self.loading_my_items() {
			return;
		}
		self.as_mut().set_my_items_error(QString::default());
		self.as_mut().set_loading_my_items(true);
		let thread = self.qt_thread();
		std::thread::spawn(move || {
			let result = workshop::my_items();
			let _ = thread.queue(move |mut backend| {
				backend.as_mut().set_loading_my_items(false);
				match result {
					Ok(items) => backend.set_my_items_json(QString::from(json(&items).as_str())),
					Err(err) => backend.set_my_items_error(QString::from(err.as_str())),
				}
			});
		});
	}

	fn save_projects(self: Pin<&mut Self>, projects: Vec<String>) {
		settings::set_projects(&projects);
		self.set_projects_json(QString::from(json(&projects).as_str()));
	}
}

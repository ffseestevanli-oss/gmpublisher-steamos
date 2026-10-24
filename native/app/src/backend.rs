/// The QML-facing backend object. Its properties are bound directly in qml/Main.qml.
#[cxx_qt::bridge]
pub mod qobject {
	unsafe extern "C++" {
		include!("cxx-qt-lib/qstring.h");
		type QString = cxx_qt_lib::QString;
	}

	extern "RustQt" {
		#[qobject]
		#[qml_element]
		#[qproperty(QString, steam_status)]
		#[qproperty(QString, persona)]
		#[qproperty(QString, gmod_dir)]
		#[qproperty(i32, subscribed_items)]
		#[qproperty(bool, connected)]
		type Backend = super::BackendRust;

		#[qinvokable]
		fn start(self: Pin<&mut Backend>);
	}

	impl cxx_qt::Threading for Backend {}
}

use core::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;

use crate::{gmod, steam};

pub struct BackendRust {
	steam_status: QString,
	persona: QString,
	gmod_dir: QString,
	subscribed_items: i32,
	connected: bool,
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
			started: false,
		}
	}
}

impl qobject::Backend {
	fn start(mut self: Pin<&mut Self>) {
		if self.as_ref().rust().started {
			return;
		}
		self.as_mut().rust_mut().started = true;

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
					if let Some(dir) = info.gmod_dir {
						backend.as_mut().set_gmod_dir(QString::from(dir.display().to_string().as_str()));
					}
					backend.set_connected(true);
				});
			},
		);
	}
}

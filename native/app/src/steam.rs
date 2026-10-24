use std::{
	path::PathBuf,
	sync::OnceLock,
	thread,
	time::Duration,
};

use steamworks::{AppId, Client};

pub const GMOD_APP_ID: AppId = AppId(4000);

static CLIENT: OnceLock<Client> = OnceLock::new();

/// The Steam client, once connected. Its callbacks are pumped by the thread `connect` starts, so
/// other threads can make calls and wait for the results.
pub fn client() -> Option<Client> {
	CLIENT.get().cloned()
}

#[derive(Debug)]
pub struct SteamInfo {
	pub persona: String,
	pub steam_id: u64,
	pub gmod_dir: Option<PathBuf>,
	pub subscribed_items: usize,
}

/// Connects to the running Steam client on a background thread, retrying until Steam is up,
/// then keeps pumping Steam callbacks for the lifetime of the app.
pub fn connect(on_waiting: impl Fn() + Send + 'static, on_connected: impl FnOnce(SteamInfo) + Send + 'static) {
	thread::spawn(move || {
		let (client, single) = loop {
			match Client::init_app(GMOD_APP_ID) {
				Ok(connection) => break connection,
				Err(_) => {
					on_waiting();
					thread::sleep(Duration::from_secs(2));
				}
			}
		};

		let _ = CLIENT.set(client.clone());
		let gmod_dir = PathBuf::from(client.apps().app_install_dir(GMOD_APP_ID));
		on_connected(SteamInfo {
			persona: client.friends().name(),
			steam_id: client.user().steam_id().raw(),
			gmod_dir: gmod_dir.is_dir().then_some(gmod_dir),
			subscribed_items: client.ugc().subscribed_items().len(),
		});

		loop {
			single.run_callbacks();
			thread::sleep(Duration::from_millis(50));
		}
	});
}

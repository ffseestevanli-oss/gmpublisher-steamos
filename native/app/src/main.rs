mod app;
mod backend;
mod cli;
mod gma_view;
mod gmod;
mod library;
mod project;
mod settings;
mod steam;

use std::sync::OnceLock;

/// A file passed on the command line (or by the file manager) for the app to open at startup.
pub static OPEN_AT_START: OnceLock<String> = OnceLock::new();

fn main() {
	match cli::parse() {
		cli::Launch::Exit(code) => std::process::exit(code),
		cli::Launch::Gui { open } => {
			if let Some(path) = open {
				let path = std::fs::canonicalize(&path).unwrap_or(path);
				let _ = OPEN_AT_START.set(path.to_string_lossy().into_owned());
			}
			std::process::exit(app::ffi::run_app());
		}
	}
}

//! A small log of what gmpublisher asked Steam and what came back, in
//! `$XDG_CACHE_HOME/gmpublisher/gmpublisher-qt.log`, so problems can be diagnosed when the app
//! was started from the menu and its output isn't visible.

use std::{
	fs::{self, OpenOptions},
	io::Write,
	path::PathBuf,
	time::{SystemTime, UNIX_EPOCH},
};

const MAX_BYTES: u64 = 1024 * 1024;

pub fn path() -> PathBuf {
	dirs::cache_dir().unwrap_or_else(std::env::temp_dir).join("gmpublisher/gmpublisher-qt.log")
}

pub fn log(message: &str) {
	eprintln!("gmpublisher-qt: {message}");
	let path = path();
	if let Some(dir) = path.parent() {
		let _ = fs::create_dir_all(dir);
	}
	// Start over rather than grow forever
	if fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES) {
		let _ = fs::remove_file(&path);
	}
	let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
	if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
		let _ = writeln!(file, "[{seconds}] {message}");
	}
}

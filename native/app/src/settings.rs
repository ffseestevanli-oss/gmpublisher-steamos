//! Remembered addon folders, kept in `$XDG_CONFIG_HOME/gmpublisher/projects.json`.

use std::{fs, path::PathBuf};

fn file() -> Option<PathBuf> {
	Some(dirs::config_dir()?.join("gmpublisher/projects.json"))
}

pub fn projects() -> Vec<String> {
	file().and_then(|f| fs::read_to_string(f).ok()).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

pub fn set_projects(projects: &[String]) {
	let Some(file) = file() else { return };
	if let Some(dir) = file.parent() {
		let _ = fs::create_dir_all(dir);
	}
	if let Ok(json) = serde_json::to_string_pretty(projects) {
		let _ = fs::write(file, json);
	}
}

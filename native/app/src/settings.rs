//! What gmpublisher remembers between runs, kept in `$XDG_CONFIG_HOME/gmpublisher`.

use std::{collections::HashMap, fs, path::PathBuf};

/// Remembered addon folders, in `projects.json`.
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

/// Which Workshop item each addon folder publishes to, in `workshop.json`.
fn workshop_file() -> Option<PathBuf> {
	Some(dirs::config_dir()?.join("gmpublisher/workshop.json"))
}

pub fn workshop_ids() -> HashMap<String, u64> {
	workshop_file().and_then(|f| fs::read_to_string(f).ok()).and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

pub fn workshop_id(folder: &str) -> Option<u64> {
	workshop_ids().get(folder).copied()
}

pub fn set_workshop_id(folder: &str, id: Option<u64>) {
	let Some(file) = workshop_file() else { return };
	let mut ids = workshop_ids();
	match id {
		Some(id) => ids.insert(folder.to_owned(), id),
		None => ids.remove(folder),
	};
	if let Some(dir) = file.parent() {
		let _ = fs::create_dir_all(dir);
	}
	if let Ok(json) = serde_json::to_string_pretty(&ids) {
		let _ = fs::write(file, json);
	}
}

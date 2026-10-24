//! Finds the addons Garry's Mod has installed: legacy GMAs in `garrysmod/addons` and Workshop
//! downloads in each Steam library.

use std::{fs, path::PathBuf};

use gmpublisher_core::Gma;
use serde::Serialize;

use crate::gmod;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAddon {
	pub path: PathBuf,
	pub title: String,
	pub workshop_id: Option<u64>,
	/// Size on disk.
	pub size: u64,
	/// File count, when the header could be read without decompressing.
	pub files: Option<usize>,
	pub source: &'static str,
}

pub fn scan(gmod_dir: Option<&PathBuf>) -> Vec<InstalledAddon> {
	let mut addons = Vec::new();

	if let Some(addons_dir) = gmod_dir.and_then(|dir| gmod::content_dir(dir)).map(|dir| dir.join("addons")) {
		for path in gma_files(&addons_dir) {
			addons.push(describe(path, None, "Addons folder"));
		}
	}

	for workshop_dir in gmod::workshop_dirs() {
		let Ok(items) = fs::read_dir(&workshop_dir) else { continue };
		for item in items.flatten() {
			let id = item.file_name().to_str().and_then(|name| name.parse().ok());
			for path in gma_files(&item.path()) {
				addons.push(describe(path, id, "Workshop"));
			}
		}
	}

	addons.sort_by_cached_key(|addon| addon.title.to_lowercase());
	addons
}

fn gma_files(dir: &PathBuf) -> Vec<PathBuf> {
	let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };
	entries
		.flatten()
		.map(|entry| entry.path())
		.filter(|path| path.is_file() && matches!(path.extension().and_then(|e| e.to_str()), Some("gma" | "bin")))
		.collect()
}

fn describe(path: PathBuf, workshop_id: Option<u64>, source: &'static str) -> InstalledAddon {
	let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
	let fallback = || match workshop_id {
		Some(id) => format!("Workshop item {id}"),
		None => path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
	};

	// Compressed downloads are only unpacked when opened, so the list stays quick to build.
	let header = (path.extension().and_then(|e| e.to_str()) == Some("gma")).then(|| Gma::open(&path).ok()).flatten();
	InstalledAddon {
		title: header.as_ref().map(|gma| gma.title.clone()).filter(|t| !t.trim().is_empty()).unwrap_or_else(fallback),
		files: header.as_ref().map(|gma| gma.entries.len()),
		workshop_id,
		size,
		source,
		path,
	}
}

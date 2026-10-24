use std::{fs, path::PathBuf};

/// Every place a Linux Steam install keeps its root, native first, then Flatpak and Snap.
fn steam_roots() -> Vec<PathBuf> {
	let Some(home) = dirs::home_dir() else { return Vec::new() };
	[
		".steam/steam",
		".steam/root",
		".local/share/Steam",
		".var/app/com.valvesoftware.Steam/.steam/steam",
		".var/app/com.valvesoftware.Steam/.local/share/Steam",
		".var/app/com.valvesoftware.Steam/data/Steam",
		"snap/steam/common/.steam/steam",
		"snap/steam/common/.local/share/Steam",
	]
	.into_iter()
	.map(|p| home.join(p))
	.filter(|p| p.is_dir())
	.collect()
}

/// Steam library folders from every Steam root, including SD cards and external drives.
pub fn steam_libraries() -> Vec<PathBuf> {
	let path_regex = regex::Regex::new(r#"(?m)^\s*"path"\s+"(.+?)"\s*$"#).unwrap();

	let mut libraries: Vec<PathBuf> = Vec::new();
	let mut add = |path: PathBuf| {
		let path = fs::canonicalize(&path).unwrap_or(path);
		if !libraries.contains(&path) {
			libraries.push(path);
		}
	};

	for root in steam_roots() {
		add(root.clone());
		if let Ok(vdf) = fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
			for path in path_regex.captures_iter(&vdf) {
				add(PathBuf::from(path[1].replace("\\\\", "\\")));
			}
		}
	}

	libraries
}

pub fn is_gmod_dir(path: &PathBuf) -> bool {
	path.join("GarrysMod/addons").is_dir()
}

/// Finds Garry's Mod without needing Steam to be running.
pub fn locate() -> Option<PathBuf> {
	steam_libraries()
		.into_iter()
		.map(|library| library.join("steamapps/common/GarrysMod"))
		.find(is_gmod_dir)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn finds_gmod_in_secondary_library() {
		let home = std::env::temp_dir().join(format!("gmpublisher-test-{}", std::process::id()));
		let steam = home.join(".local/share/Steam");
		let sd = home.join("sd");
		fs::create_dir_all(steam.join("steamapps")).unwrap();
		fs::create_dir_all(sd.join("steamapps/common/GarrysMod/GarrysMod/addons")).unwrap();
		// An empty Flatpak Steam directory must not hide the native install
		fs::create_dir_all(home.join(".var/app/com.valvesoftware.Steam/.steam/steam")).unwrap();
		fs::write(
			steam.join("steamapps/libraryfolders.vdf"),
			format!("\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n", steam.display(), sd.display()),
		)
		.unwrap();

		std::env::set_var("HOME", &home);
		assert_eq!(locate(), Some(fs::canonicalize(&sd).unwrap().join("steamapps/common/GarrysMod")));
		fs::remove_dir_all(&home).ok();
	}
}

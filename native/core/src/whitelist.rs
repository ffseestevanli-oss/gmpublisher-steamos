//! Which files Garry's Mod accepts inside a GMA.
//!
//! The list mirrors Facepunch's gmad `AddonWhiteList.h`. A leading `!` excludes paths that an earlier
//! pattern allowed.

const ALLOWED: &[&str] = &[
	"lua/*.lua",
	"scenes/*.vcd",
	"particles/*.pcf",
	"resource/fonts/*.ttf",
	"scripts/vehicles/*.txt",
	"resource/localization/*/*.properties",
	"maps/*.bsp",
	"maps/*.lmp",
	"maps/*.nav",
	"maps/*.ain",
	"maps/thumb/*.png",
	"sound/*.wav",
	"sound/*.mp3",
	"sound/*.ogg",
	"materials/*.vmt",
	"materials/*.vtf",
	"materials/*.png",
	"materials/*.jpg",
	"materials/*.jpeg",
	"materials/colorcorrection/*.raw",
	"models/*.mdl",
	"models/*.phy",
	"models/*.ani",
	"models/*.vvd",
	"models/*.vtx",
	"!models/*.sw.vtx",
	"!models/*.360.vtx",
	"!models/*.xbox.vtx",
	"gamemodes/*/*.txt",
	"!gamemodes/*/*/*.txt",
	"gamemodes/*/*.fgd",
	"!gamemodes/*/*/*.fgd",
	"gamemodes/*/logo.png",
	"gamemodes/*/icon24.png",
	"gamemodes/*/gamemode/*.lua",
	"gamemodes/*/entities/effects/*.lua",
	"gamemodes/*/entities/weapons/*.lua",
	"gamemodes/*/entities/entities/*.lua",
	"gamemodes/*/backgrounds/*.png",
	"gamemodes/*/backgrounds/*.jpg",
	"gamemodes/*/backgrounds/*.jpeg",
	"gamemodes/*/content/models/*.mdl",
	"gamemodes/*/content/models/*.phy",
	"gamemodes/*/content/models/*.ani",
	"gamemodes/*/content/models/*.vvd",
	"gamemodes/*/content/models/*.vtx",
	"!gamemodes/*/content/models/*.sw.vtx",
	"!gamemodes/*/content/models/*.360.vtx",
	"!gamemodes/*/content/models/*.xbox.vtx",
	"gamemodes/*/content/materials/*.vmt",
	"gamemodes/*/content/materials/*.vtf",
	"gamemodes/*/content/materials/*.png",
	"gamemodes/*/content/materials/*.jpg",
	"gamemodes/*/content/materials/*.jpeg",
	"gamemodes/*/content/materials/colorcorrection/*.raw",
	"gamemodes/*/content/scenes/*.vcd",
	"gamemodes/*/content/particles/*.pcf",
	"gamemodes/*/content/resource/fonts/*.ttf",
	"gamemodes/*/content/scripts/vehicles/*.txt",
	"gamemodes/*/content/resource/localization/*/*.properties",
	"gamemodes/*/content/maps/*.bsp",
	"gamemodes/*/content/maps/*.nav",
	"gamemodes/*/content/maps/*.ain",
	"gamemodes/*/content/maps/thumb/*.png",
	"gamemodes/*/content/sound/*.wav",
	"gamemodes/*/content/sound/*.mp3",
	"gamemodes/*/content/sound/*.ogg",
	"data_static/*.txt",
	"data_static/*.dat",
	"data_static/*.json",
	"data_static/*.xml",
	"data_static/*.csv",
	"shaders/fxc/*.vcs",
];

/// Files that never belong in an addon: editor projects, VCS metadata, licences and the like.
pub const DEFAULT_IGNORE: &[&str] = &[
	".git/*",
	".github/*",
	".vscode/*",
	".idea/*",
	".vs/*",
	"*.psd",
	"*.pdn",
	"*.xcf",
	"*.kra",
	"*.blend",
	"*.blend1",
	"*.svn",
	"*.ini",
	"*.rtf",
	"*.pdf",
	"*.log",
	"*.prt",
	"*.vmf",
	"*.vmx",
	"*~",
	".directory",
	".DS_Store",
	".gitignore",
	".gitmodules",
	".gitattributes",
	".editorconfig",
	"LICENSE",
	"LICENSE.*",
	"license",
	"license.*",
	"README",
	"README.*",
	"readme",
	"readme.*",
	"addon.json",
	"addon.txt",
	"addon.jpg",
	"thumbs.db",
	"desktop.ini",
];

/// Matches `path` against a gmad-style wildcard, where `*` spans any characters including `/`
/// and `?` matches exactly one.
pub fn wildcard_match(pattern: &str, path: &str) -> bool {
	let (pattern, path) = (pattern.as_bytes(), path.as_bytes());
	let (mut p, mut s) = (0, 0);
	let mut backtrack: Option<(usize, usize)> = None;

	while s < path.len() {
		match pattern.get(p) {
			Some(b'*') => {
				backtrack = Some((p, s));
				p += 1;
			}
			Some(&c) if c == b'?' || c == path[s] => {
				p += 1;
				s += 1;
			}
			_ => match backtrack {
				Some((star_p, star_s)) => {
					backtrack = Some((star_p, star_s + 1));
					p = star_p + 1;
					s = star_s + 1;
				}
				None => return false,
			},
		}
	}

	pattern[p..].iter().all(|&c| c == b'*')
}

/// Whether Garry's Mod allows this (lowercase, `/`-separated, relative) path inside an addon.
pub fn is_allowed(path: &str) -> bool {
	let mut allowed = false;
	for pattern in ALLOWED {
		match pattern.strip_prefix('!') {
			Some(excluded) => {
				if allowed && wildcard_match(excluded, path) {
					allowed = false;
				}
			}
			None => {
				if !allowed && wildcard_match(pattern, path) {
					allowed = true;
				}
			}
		}
	}
	allowed
}

/// Whether any of `patterns` matches `path`.
pub fn is_ignored<S: AsRef<str>>(path: &str, patterns: &[S]) -> bool {
	patterns.iter().any(|pattern| wildcard_match(pattern.as_ref(), path))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn allows_addon_content() {
		for path in [
			"lua/autorun/init.lua",
			"lua/lua/deep/file.lua",
			"gamemodes/test/test.txt",
			"gamemodes/test/content/sound/a.wav",
			"materials/models/thing.vmt",
			"gamemodes/base_defence/backgrounds/1.jpg",
			"models/player.vtx",
			"gamemodes/sandbox/sandbox.fgd",
		] {
			assert!(is_allowed(path), "{path}");
		}
	}

	#[test]
	fn every_pattern_accepts_a_matching_path() {
		for pattern in ALLOWED.iter().filter(|p| !p.starts_with('!')) {
			assert!(is_allowed(&pattern.replace('*', "x")), "{pattern}");
		}
	}

	#[test]
	fn rejects_other_files() {
		for path in [
			"init.lua",
			"lua/evil.exe",
			"lua/lib/evil.so",
			"gamemodes/test",
			"gamemodes/test/nested/info.txt",
			"materials/thing.blend",
			"models/player.sw.vtx",
			"models/weapons/gun.xbox.vtx",
			"gamemodes/test/content/models/player.360.vtx",
		] {
			assert!(!is_allowed(path), "{path}");
		}
	}

	#[test]
	fn wildcards_match_whole_strings() {
		assert!(wildcard_match("LICENSE", "LICENSE"));
		assert!(!wildcard_match("LICENSE", "LICENSE.bak"));
		assert!(!wildcard_match("a", "abc"));
		assert!(wildcard_match("*.psd", "art/layers/logo.psd"));
		assert!(wildcard_match("a?c", "abc"));
		assert!(wildcard_match("lua/*", "lua/x/y.lua"));
		assert!(wildcard_match("*", ""));
	}

	#[test]
	fn default_ignores() {
		for path in [".git/HEAD", ".git/objects/00/abc", "src/logo.psd", "addon.json", ".directory", "notes.txt~"] {
			assert!(is_ignored(path, DEFAULT_IGNORE), "{path}");
		}
		assert!(!is_ignored(".gitignore.bak", DEFAULT_IGNORE));
		assert!(!is_ignored("lua/autorun/init.lua", DEFAULT_IGNORE));
		assert!(!is_ignored::<&str>("anything", &[]));
	}
}

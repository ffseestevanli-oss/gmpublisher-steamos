//! Command-line use, for the terminal, scripts and file manager actions.

use std::path::{Path, PathBuf};

use gmpublisher_core::{pack, AddonJson, Gma};

const USAGE: &str = "\
Usage:
  gmpublisher-qt                       open the app
  gmpublisher-qt FILE.gma              open a GMA in the app
  gmpublisher-qt FOLDER                open an addon folder to edit, pack or publish
  gmpublisher-qt list FILE.gma         list the files in a GMA
  gmpublisher-qt extract FILE.gma [DIR]
                                       extract a GMA (default: a folder next to it)
  gmpublisher-qt pack FOLDER [OUT.gma] pack an addon folder using its addon.json";

pub enum Launch {
	Gui { open: Option<PathBuf> },
	Exit(i32),
}

pub fn parse() -> Launch {
	let args: Vec<String> = std::env::args().skip(1).collect();
	let args: Vec<&str> = args.iter().map(String::as_str).collect();
	let result = match args.as_slice() {
		[] => return Launch::Gui { open: None },
		["-h" | "--help"] => {
			println!("{USAGE}");
			Ok(())
		}
		["list", gma] => list(Path::new(gma)),
		["extract", gma] => extract(Path::new(gma), None),
		["extract", gma, dest] => extract(Path::new(gma), Some(Path::new(dest))),
		["pack", folder] => pack_folder(Path::new(folder), None),
		["pack", folder, out] => pack_folder(Path::new(folder), Some(Path::new(out))),
		[file] if !file.starts_with('-') => return Launch::Gui { open: Some(PathBuf::from(file)) },
		_ => Err(format!("unrecognised arguments\n\n{USAGE}")),
	};
	match result {
		Ok(()) => Launch::Exit(0),
		Err(err) => {
			eprintln!("gmpublisher-qt: {err}");
			Launch::Exit(1)
		}
	}
}

/// Where a GMA is extracted when no folder is given: a new folder next to it, named after it.
/// Never an existing folder, so extracting can't overwrite anything (such as the addon's source).
pub fn default_extract_dir(gma: &Path) -> PathBuf {
	let stem = gma.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "addon".into());
	unused_path(gma.parent().unwrap_or(Path::new(".")), &stem, "")
}

/// `dir/name.ext`, or `dir/name (2).ext` and so on if that's taken. `ext` may be empty.
pub fn unused_path(dir: &Path, name: &str, ext: &str) -> PathBuf {
	let ext = if ext.is_empty() { String::new() } else { format!(".{ext}") };
	(1..)
		.map(|n| dir.join(if n == 1 { format!("{name}{ext}") } else { format!("{name} ({n}){ext}") }))
		.find(|path| !path.exists())
		.expect("some name is free")
}

fn list(path: &Path) -> Result<(), String> {
	let gma = Gma::open(path).map_err(|e| e.to_string())?;
	println!("{} ({} files)", gma.title, gma.entries.len());
	for entry in &gma.entries {
		println!("{:>12}  {}", entry.size, entry.path);
	}
	Ok(())
}

fn extract(path: &Path, dest: Option<&Path>) -> Result<(), String> {
	let gma = Gma::open(path).map_err(|e| e.to_string())?;
	let dest = dest.map(Path::to_path_buf).unwrap_or_else(|| default_extract_dir(path));
	let count = gma.extract(&dest, |_, _| true).map_err(|e| e.to_string())?;
	println!("Extracted {count} files to {}", dest.display());
	for skipped in &gma.rejected {
		eprintln!("Skipped unsafe path: {skipped}");
	}
	Ok(())
}

fn pack_folder(folder: &Path, out: Option<&Path>) -> Result<(), String> {
	let addon = AddonJson::load(folder).map_err(|e| e.to_string())?.ok_or("the folder has no addon.json")?;
	if let Some(problem) = addon.problems().first() {
		return Err(problem.clone());
	}
	let plan = pack::plan(folder, &addon).map_err(|e| e.to_string())?;
	for path in &plan.not_allowed {
		eprintln!("Not allowed by Garry's Mod: {path}");
	}
	let out = out.map(Path::to_path_buf).unwrap_or_else(|| folder.with_extension("gma"));
	pack::write(&plan, &addon, &out, |_, _| true).map_err(|e| e.to_string())?;
	println!("Packed {} files into {}", plan.files.len(), out.display());
	Ok(())
}

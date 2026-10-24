//! Building a GMA from an addon folder.

use std::{
	fs::{self, File},
	io::{BufWriter, Write},
	path::{Path, PathBuf},
	time::{SystemTime, UNIX_EPOCH},
};

use walkdir::WalkDir;

use crate::{
	addon_json::AddonJson,
	gma::MAGIC,
	whitelist::{self, DEFAULT_IGNORE},
	Error, Result,
};

/// What packing an addon folder would include and leave out.
#[derive(Debug, Default)]
pub struct Plan {
	/// (path inside the GMA, file on disk), sorted by GMA path.
	pub files: Vec<(String, PathBuf)>,
	/// Files that Garry's Mod wouldn't accept. Packing refuses to continue while there are any.
	pub not_allowed: Vec<String>,
	/// Files skipped because of the default ignore list or addon.json's `ignore`.
	pub ignored: Vec<String>,
	pub total_size: u64,
}

/// Works out which files of `addon_dir` go into the GMA.
pub fn plan(addon_dir: impl AsRef<Path>, addon: &AddonJson) -> Result<Plan> {
	let addon_dir = addon_dir.as_ref();
	let mut plan = Plan::default();

	for entry in WalkDir::new(addon_dir).follow_links(false).sort_by_file_name() {
		let entry = entry.map_err(|err| Error::Io(err.into()))?;
		if !entry.file_type().is_file() {
			continue;
		}
		let relative = entry.path().strip_prefix(addon_dir).expect("walkdir stays below its root");
		let Some(relative) = relative.to_str() else {
			plan.not_allowed.push(relative.to_string_lossy().into_owned());
			continue;
		};
		let gma_path = relative.to_lowercase();

		if whitelist::is_ignored(&gma_path, DEFAULT_IGNORE) || whitelist::is_ignored(&gma_path, &addon.ignore) {
			plan.ignored.push(gma_path);
		} else if whitelist::is_allowed(&gma_path) {
			plan.total_size += entry.metadata().map_err(|err| Error::Io(err.into()))?.len();
			plan.files.push((gma_path, entry.into_path()));
		} else {
			plan.not_allowed.push(gma_path);
		}
	}

	plan.files.sort_by(|a, b| a.0.cmp(&b.0));
	if let Some(window) = plan.files.windows(2).find(|w| w[0].0 == w[1].0) {
		return Err(Error::DuplicatePath(window[0].0.clone()));
	}
	Ok(plan)
}

/// Writes `plan` as a GMA to `out`. `progress` receives (files done, files total) and can return
/// `false` to cancel. The file is written next to `out` and renamed into place when complete.
pub fn write(plan: &Plan, addon: &AddonJson, out: impl AsRef<Path>, mut progress: impl FnMut(usize, usize) -> bool) -> Result<()> {
	if let Some(path) = plan.not_allowed.first() {
		return Err(Error::NotAllowed(path.clone()));
	}
	if plan.files.is_empty() {
		return Err(Error::Empty);
	}

	let out = out.as_ref();
	let partial = out.with_extension("gma.part");
	let result = (|| {
		let mut w = BufWriter::new(File::create(&partial)?);
		let mut crc_all = crc32fast::Hasher::new();
		let mut put = |w: &mut BufWriter<File>, bytes: &[u8]| -> Result<()> {
			crc_all.update(bytes);
			w.write_all(bytes)?;
			Ok(())
		};
		let string = |s: &str| [s.as_bytes(), &[0]].concat();

		let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
		let mut embedded = addon.clone();
		embedded.description.clear();

		put(&mut w, MAGIC)?;
		put(&mut w, &[3])?;
		put(&mut w, &0u64.to_le_bytes())?;
		put(&mut w, &timestamp.to_le_bytes())?;
		put(&mut w, &string(""))?;
		put(&mut w, &string(&addon.title))?;
		put(&mut w, &string(&serde_json::to_string(&embedded)?))?;
		put(&mut w, &string("Author Name"))?;
		put(&mut w, &1i32.to_le_bytes())?;

		// The file list needs every CRC up front, so hash first and copy afterwards.
		let total = plan.files.len();
		let mut sizes = Vec::with_capacity(total);
		for (i, (gma_path, disk_path)) in plan.files.iter().enumerate() {
			let contents = fs::read(disk_path)?;
			sizes.push(contents.len() as u64);
			put(&mut w, &(i as u32 + 1).to_le_bytes())?;
			put(&mut w, &string(gma_path))?;
			put(&mut w, &(contents.len() as i64).to_le_bytes())?;
			put(&mut w, &crc32fast::hash(&contents).to_le_bytes())?;
		}
		put(&mut w, &0u32.to_le_bytes())?;

		for (done, (_, disk_path)) in plan.files.iter().enumerate() {
			if !progress(done, total) {
				return Err(Error::Cancelled);
			}
			let contents = fs::read(disk_path)?;
			if contents.len() as u64 != sizes[done] {
				return Err(Error::ChangedWhilePacking(disk_path.clone()));
			}
			put(&mut w, &contents)?;
		}

		let crc = crc_all.finalize();
		w.write_all(&crc.to_le_bytes())?;
		w.into_inner().map_err(|err| err.into_error())?.sync_all()?;
		progress(total, total);
		Ok(())
	})();

	match result {
		Ok(()) => Ok(fs::rename(&partial, out)?),
		Err(err) => {
			let _ = fs::remove_file(&partial);
			Err(err)
		}
	}
}

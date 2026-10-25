//! Reading and extracting GMA archives.
//!
//! Layout (all integers little-endian, strings NUL-terminated):
//!
//! ```text
//! "GMAD" u8 version
//! u64 steamid, u64 timestamp
//! [version > 1] string required_content... ""   (list ends with an empty string)
//! string title, string description, string author, i32 addon_version
//! { u32 index (0 ends the list), string path, i64 size, u32 crc32 }...
//! file contents, back to back in list order
//! [optional] u32 crc32 of everything before it
//! ```

use std::{
	fs::{self, File},
	io::{self, BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write},
	path::{Path, PathBuf},
};

use crate::{addon_json::AddonJson, Error, Result};

pub const MAGIC: &[u8; 4] = b"GMAD";

/// One file stored in a GMA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
	pub path: String,
	pub size: u64,
	pub crc: u32,
	/// Offset of the contents from the start of the data section.
	pub offset: u64,
}

/// The header and file list of a GMA. File contents are read on demand.
#[derive(Debug, Clone)]
pub struct Gma {
	pub version: u8,
	pub steam_id: u64,
	pub timestamp: u64,
	pub title: String,
	/// The raw description field. Modern addons store their addon.json here.
	pub description: String,
	pub author: String,
	pub entries: Vec<Entry>,
	/// Paths that were in the archive but refused because they could escape the extraction folder.
	pub rejected: Vec<String>,
	data_start: u64,
	source: Source,
}

#[derive(Debug, Clone)]
enum Source {
	File(PathBuf),
	Memory(std::sync::Arc<[u8]>),
}

trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

impl Gma {
	/// Opens a `.gma`, or an LZMA-compressed Workshop download (`.bin`, `.gma.lzma`).
	pub fn open(path: impl AsRef<Path>) -> Result<Self> {
		let path = path.as_ref();
		let mut file = BufReader::new(File::open(path)?);

		let mut magic = [0u8; 4];
		file.read_exact(&mut magic)?;
		file.rewind()?;

		if &magic == MAGIC {
			Self::parse(&mut file, Source::File(path.to_owned()))
		} else {
			let mut decompressed = Vec::new();
			lzma_rs::lzma_decompress(&mut file, &mut decompressed).map_err(|_| Error::NotAGma)?;
			Self::from_bytes(decompressed)
		}
	}

	/// Parses a GMA held in memory.
	pub fn from_bytes(bytes: impl Into<std::sync::Arc<[u8]>>) -> Result<Self> {
		let bytes = bytes.into();
		Self::parse(&mut Cursor::new(bytes.clone()), Source::Memory(bytes))
	}

	fn parse<R: BufRead + Seek>(r: &mut R, source: Source) -> Result<Self> {
		let mut magic = [0u8; 4];
		r.read_exact(&mut magic)?;
		if &magic != MAGIC {
			return Err(Error::NotAGma);
		}

		let version = read_u8(r)?;
		let steam_id = read_u64(r)?;
		let timestamp = read_u64(r)?;
		if version > 1 {
			while !read_string(r)?.is_empty() {}
		}
		let title = read_string(r)?;
		let description = read_string(r)?;
		let author = read_string(r)?;
		let _addon_version = read_u32(r)?;

		let mut entries = Vec::new();
		let mut rejected = Vec::new();
		let mut offset: u64 = 0;
		while read_u32(r)? != 0 {
			let path = read_string(r)?;
			let size = u64::try_from(read_u64(r)? as i64).map_err(|_| Error::Corrupt("negative file size"))?;
			let crc = read_u32(r)?;

			if is_safe_path(&path) {
				entries.push(Entry { path, size, crc, offset });
			} else {
				rejected.push(path);
			}
			offset = offset.checked_add(size).ok_or(Error::Corrupt("file sizes overflow"))?;
		}

		Ok(Self {
			version,
			steam_id,
			timestamp,
			title,
			description,
			author,
			entries,
			rejected,
			data_start: r.stream_position()?,
			source,
		})
	}

	/// The addon.json embedded in the description, if the addon was packed with one.
	pub fn addon_json(&self) -> Option<AddonJson> {
		serde_json::from_str(&self.description).ok()
	}

	/// Total size of the stored files.
	pub fn content_size(&self) -> u64 {
		self.entries.iter().map(|e| e.size).sum()
	}

	fn reader(&self) -> Result<Box<dyn ReadSeek + '_>> {
		Ok(match &self.source {
			Source::File(path) => Box::new(BufReader::new(File::open(path)?)),
			Source::Memory(bytes) => Box::new(Cursor::new(&bytes[..])),
		})
	}

	/// Reads one file's contents.
	pub fn read_entry(&self, entry: &Entry) -> Result<Vec<u8>> {
		let mut r = self.reader()?;
		r.seek(SeekFrom::Start(self.data_start + entry.offset))?;
		let mut buf = Vec::with_capacity(entry.size as usize);
		r.take(entry.size).read_to_end(&mut buf)?;
		if buf.len() as u64 != entry.size {
			return Err(Error::Corrupt("file contents cut short"));
		}
		Ok(buf)
	}

	/// Extracts every file below `dest`. `progress` receives (files done, files total) and can return
	/// `false` to cancel. Returns the number of files written.
	pub fn extract(&self, dest: impl AsRef<Path>, mut progress: impl FnMut(usize, usize) -> bool) -> Result<usize> {
		let dest = dest.as_ref();
		let mut r = self.reader()?;
		let total = self.entries.len();

		for (done, entry) in self.entries.iter().enumerate() {
			if !progress(done, total) {
				return Err(Error::Cancelled);
			}
			let target = dest.join(&entry.path);
			if let Some(parent) = target.parent() {
				fs::create_dir_all(parent)?;
			}
			r.seek(SeekFrom::Start(self.data_start + entry.offset))?;
			let mut out = io::BufWriter::new(File::create(&target)?);
			let copied = io::copy(&mut (&mut r).take(entry.size), &mut out)?;
			out.flush()?;
			if copied != entry.size {
				return Err(Error::Corrupt("file contents cut short"));
			}
		}
		progress(total, total);
		Ok(total)
	}

	/// Writes the whole archive, uncompressed, to `dest`. Turns a compressed Workshop download into
	/// a plain `.gma` that Garry's Mod and other tools can open.
	pub fn save_as(&self, dest: impl AsRef<Path>) -> Result<()> {
		match &self.source {
			Source::File(path) => {
				fs::copy(path, dest)?;
			}
			Source::Memory(bytes) => fs::write(dest, &bytes[..])?,
		}
		Ok(())
	}

	/// Checks every file against its stored CRC32 and returns the paths that don't match.
	pub fn verify(&self) -> Result<Vec<String>> {
		let mut r = self.reader()?;
		let mut bad = Vec::new();
		for entry in &self.entries {
			r.seek(SeekFrom::Start(self.data_start + entry.offset))?;
			let mut hasher = crc32fast::Hasher::new();
			let mut chunk = (&mut r).take(entry.size);
			let mut buf = [0u8; 64 * 1024];
			loop {
				let n = chunk.read(&mut buf)?;
				if n == 0 {
					break;
				}
				hasher.update(&buf[..n]);
			}
			if hasher.finalize() != entry.crc {
				bad.push(entry.path.clone());
			}
		}
		Ok(bad)
	}
}

/// Entry paths come from untrusted files. Only plain relative paths are allowed, so nothing can be
/// written outside the extraction folder.
pub fn is_safe_path(path: &str) -> bool {
	!path.is_empty()
		&& !path.starts_with('/')
		&& !path.bytes().any(|b| b == 0 || b == b'\\' || b == b':')
		&& path.split('/').all(|seg| !seg.is_empty() && seg != "." && seg != ".." && seg == seg.trim())
}

fn read_u8(r: &mut impl Read) -> Result<u8> {
	let mut b = [0u8; 1];
	r.read_exact(&mut b)?;
	Ok(b[0])
}

fn read_u32(r: &mut impl Read) -> Result<u32> {
	let mut b = [0u8; 4];
	r.read_exact(&mut b)?;
	Ok(u32::from_le_bytes(b))
}

fn read_u64(r: &mut impl Read) -> Result<u64> {
	let mut b = [0u8; 8];
	r.read_exact(&mut b)?;
	Ok(u64::from_le_bytes(b))
}

fn read_string(r: &mut impl BufRead) -> Result<String> {
	let mut buf = Vec::new();
	r.read_until(0, &mut buf)?;
	if buf.pop() != Some(0) {
		return Err(Error::Corrupt("unterminated string"));
	}
	Ok(String::from_utf8_lossy(&buf).into_owned())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn safe_paths() {
		for ok in ["lua/autorun/a.lua", "addon.json", "foo..bar/baz"] {
			assert!(is_safe_path(ok), "{ok}");
		}
		for bad in [
			"",
			"/etc/passwd",
			"../x",
			"a/../../b",
			"./a",
			"a//b",
			"a\\b",
			"C:\\x",
			"c:x",
			"a\0b",
			" a/b",
			"a /b",
			"\\\\server\\share",
		] {
			assert!(!is_safe_path(bad), "{bad:?}");
		}
	}

	#[test]
	fn rejects_non_gma() {
		assert!(matches!(Gma::from_bytes(b"NOPE".to_vec()), Err(Error::NotAGma)));
	}
}

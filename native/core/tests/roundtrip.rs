use std::{fs, io::Write};

use gmpublisher_core::{pack, AddonJson, Error, Gma};

fn addon() -> AddonJson {
	AddonJson {
		title: "Roundtrip".into(),
		addon_type: "tool".into(),
		tags: vec!["fun".into()],
		ignore: vec!["lua/secret/*".into()],
		description: "Not stored in the GMA".into(),
	}
}

fn write(dir: &std::path::Path, path: &str, contents: &[u8]) {
	let path = dir.join(path);
	fs::create_dir_all(path.parent().unwrap()).unwrap();
	fs::write(path, contents).unwrap();
}

#[test]
fn pack_open_extract() {
	let src = tempfile::tempdir().unwrap();
	write(src.path(), "lua/autorun/init.lua", b"print('hi')");
	write(src.path(), "Materials/Logo.VMT", b"\"VertexLitGeneric\" {}");
	write(src.path(), "lua/secret/skip.lua", b"ignored by addon.json");
	write(src.path(), ".git/HEAD", b"ignored by default");
	write(src.path(), "README.md", b"ignored by default");
	addon().save(src.path()).unwrap();

	let plan = pack::plan(src.path(), &addon()).unwrap();
	let paths: Vec<_> = plan.files.iter().map(|(p, _)| p.as_str()).collect();
	assert_eq!(paths, ["lua/autorun/init.lua", "materials/logo.vmt"]);
	assert!(plan.not_allowed.is_empty());
	assert_eq!(plan.ignored.len(), 4);

	let out = tempfile::tempdir().unwrap();
	let gma_path = out.path().join("roundtrip.gma");
	pack::write(&plan, &addon(), &gma_path, |_, _| true).unwrap();
	assert!(!out.path().join("roundtrip.gma.part").exists());

	let gma = Gma::open(&gma_path).unwrap();
	assert_eq!(gma.version, 3);
	assert_eq!(gma.title, "Roundtrip");
	let embedded = gma.addon_json().unwrap();
	assert_eq!(embedded.addon_type, "tool");
	assert!(embedded.description.is_empty());
	assert_eq!(gma.entries.len(), 2);
	assert!(gma.verify().unwrap().is_empty());
	assert_eq!(gma.read_entry(&gma.entries[0]).unwrap(), b"print('hi')");

	let dest = tempfile::tempdir().unwrap();
	assert_eq!(gma.extract(dest.path(), |_, _| true).unwrap(), 2);
	assert_eq!(fs::read(dest.path().join("materials/logo.vmt")).unwrap(), b"\"VertexLitGeneric\" {}");
}

#[test]
fn refuses_files_gmod_would_reject() {
	let src = tempfile::tempdir().unwrap();
	write(src.path(), "lua/autorun/init.lua", b"");
	write(src.path(), "lua/bin/gmsv_evil_linux.dll", b"");

	let plan = pack::plan(src.path(), &addon()).unwrap();
	assert_eq!(plan.not_allowed, ["lua/bin/gmsv_evil_linux.dll"]);
	let out = tempfile::tempdir().unwrap();
	assert!(matches!(pack::write(&plan, &addon(), out.path().join("x.gma"), |_, _| true), Err(Error::NotAllowed(_))));
}

#[test]
fn refuses_case_collisions() {
	let src = tempfile::tempdir().unwrap();
	write(src.path(), "lua/a.lua", b"");
	write(src.path(), "lua/A.lua", b"");
	assert!(matches!(pack::plan(src.path(), &addon()), Err(Error::DuplicatePath(_))));
}

#[test]
fn skips_traversal_entries() {
	// A hand-built GMA whose second entry tries to escape the extraction folder.
	let mut bytes = Vec::new();
	bytes.extend_from_slice(b"GMAD\x03");
	bytes.extend_from_slice(&[0; 16]);
	bytes.extend_from_slice(b"\0Evil\0desc\0author\0");
	bytes.extend_from_slice(&1i32.to_le_bytes());
	for (i, path) in ["lua/ok.lua", "../../.bashrc"].iter().enumerate() {
		bytes.extend_from_slice(&(i as u32 + 1).to_le_bytes());
		bytes.extend_from_slice(path.as_bytes());
		bytes.push(0);
		bytes.extend_from_slice(&2i64.to_le_bytes());
		bytes.extend_from_slice(&crc32fast::hash(b"ok").to_le_bytes());
	}
	bytes.extend_from_slice(&0u32.to_le_bytes());
	bytes.extend_from_slice(b"okok");

	let gma = Gma::from_bytes(bytes).unwrap();
	assert_eq!(gma.entries.len(), 1);
	assert_eq!(gma.rejected, ["../../.bashrc"]);
	assert_eq!(gma.read_entry(&gma.entries[0]).unwrap(), b"ok");
}

#[test]
fn opens_lzma_workshop_downloads() {
	let src = tempfile::tempdir().unwrap();
	write(src.path(), "lua/autorun/init.lua", b"print('compressed')");
	let plan = pack::plan(src.path(), &addon()).unwrap();
	let out = tempfile::tempdir().unwrap();
	let gma_path = out.path().join("plain.gma");
	pack::write(&plan, &addon(), &gma_path, |_, _| true).unwrap();

	let mut compressed = Vec::new();
	lzma_rs::lzma_compress(&mut &fs::read(&gma_path).unwrap()[..], &mut compressed).unwrap();
	let bin = out.path().join("123_legacy.bin");
	fs::File::create(&bin).unwrap().write_all(&compressed).unwrap();

	let gma = Gma::open(&bin).unwrap();
	assert_eq!(gma.read_entry(&gma.entries[0]).unwrap(), b"print('compressed')");
}

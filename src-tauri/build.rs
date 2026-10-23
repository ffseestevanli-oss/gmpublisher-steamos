fn main() {
	if cfg!(target_os = "linux") {
		// Look for libsteam_api.so next to the executable (portable ZIP) or in ../lib (AppImage),
		// rather than in the current working directory.
		println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN:$ORIGIN/../lib");
	}

	tauri_build::build()
}

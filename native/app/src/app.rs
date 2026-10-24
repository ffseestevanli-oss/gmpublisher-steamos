#[cxx::bridge]
pub mod ffi {
	unsafe extern "C++" {
		include!("cpp/app.h");

		/// Implemented in cpp/app.cpp: creates the QApplication, loads the QML UI and runs the event loop.
		fn run_app() -> i32;
	}
}

use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
	// Ship libsteam_api.so next to the executable
	println!("cargo::rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");

	CxxQtBuilder::new()
		.qt_module("Widgets")
		.qt_module("Quick")
		.qt_module("QuickControls2")
		.file("src/app.rs")
		.qml_module(QmlModule {
			uri: "io.github.gmpublisher",
			rust_files: &["src/backend.rs"],
			qml_files: &["qml/Main.qml"],
			..Default::default()
		})
		.cc_builder(|cc| {
			cc.include(env!("CARGO_MANIFEST_DIR"));
			cc.file("cpp/app.cpp");
		})
		.build();

	println!("cargo::rerun-if-changed=cpp/app.cpp");
	println!("cargo::rerun-if-changed=cpp/app.h");
}

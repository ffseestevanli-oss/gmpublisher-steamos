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
			rust_files: &["src/backend.rs", "src/gma_view.rs", "src/project.rs", "src/workshop_task.rs"],
			qml_files: &[
				"qml/Main.qml",
				"qml/InstalledPage.qml",
				"qml/GmaPage.qml",
				"qml/ProjectsPage.qml",
				"qml/ProjectPage.qml",
				"qml/AboutPage.qml",
				"qml/PublishDialog.qml",
				"qml/MyWorkshopPage.qml",
				"qml/ItemEditorDialog.qml",
				"qml/DownloadDialog.qml",
				"qml/TaskStatus.qml",
				"qml/RainbowGradient.qml",
				"qml/RainbowButton.qml",
				"qml/RainbowProgressBar.qml",
				"qml/PreviewImage.qml",
			],
			qrc_files: &[
				"assets/SteventechnoRainbow.colors",
				"assets/brand/gmpublisher-icon.png",
				"assets/brand/steventechno-logo.png",
			],
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

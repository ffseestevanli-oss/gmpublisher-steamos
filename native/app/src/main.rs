mod app;
mod backend;
mod gmod;
mod steam;

fn main() {
	std::process::exit(app::ffi::run_app());
}

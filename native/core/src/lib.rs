//! The parts of gmpublisher that don't need a UI or Steam: GMA archives, addon.json and the
//! Garry's Mod file whitelist.

pub mod addon_json;
pub mod gma;
pub mod pack;
pub mod whitelist;

pub use addon_json::AddonJson;
pub use gma::{Entry, Gma};

use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error("invalid addon.json: {0}")]
	Json(#[from] serde_json::Error),
	#[error("this isn't a GMA file")]
	NotAGma,
	#[error("the GMA is damaged ({0})")]
	Corrupt(&'static str),
	#[error("Garry's Mod doesn't allow {0} in addons")]
	NotAllowed(String),
	#[error("two files would both be stored as {0} (GMA paths are lowercase)")]
	DuplicatePath(String),
	#[error("{0} changed while the addon was being packed")]
	ChangedWhilePacking(PathBuf),
	#[error("there are no files to pack")]
	Empty,
	#[error("cancelled")]
	Cancelled,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

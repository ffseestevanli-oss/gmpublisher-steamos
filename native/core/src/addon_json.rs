use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::Result;

/// The addon types the Workshop accepts.
pub const TYPES: &[&str] = &["gamemode", "map", "weapon", "vehicle", "npc", "entity", "tool", "effects", "model", "servercontent"];

/// The Workshop tags an addon can have (at most two).
pub const TAGS: &[&str] = &["fun", "roleplay", "scenic", "movie", "realism", "cartoon", "water", "comic", "build"];

/// An addon's `addon.json`. Garry's Mod also embeds this in the GMA's description field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddonJson {
	#[serde(default)]
	pub title: String,
	#[serde(default, rename = "type")]
	pub addon_type: String,
	#[serde(default)]
	pub tags: Vec<String>,
	#[serde(default)]
	pub ignore: Vec<String>,
	/// Workshop description shown on the item page. Not part of the GMA.
	#[serde(default, skip_serializing_if = "String::is_empty")]
	pub description: String,
}

impl AddonJson {
	pub fn load(addon_dir: impl AsRef<Path>) -> Result<Option<Self>> {
		let path = addon_dir.as_ref().join("addon.json");
		match fs::read_to_string(&path) {
			Ok(text) => Ok(Some(serde_json::from_str(&text)?)),
			Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
			Err(err) => Err(err.into()),
		}
	}

	pub fn save(&self, addon_dir: impl AsRef<Path>) -> Result<()> {
		fs::write(addon_dir.as_ref().join("addon.json"), serde_json::to_string_pretty(self)? + "\n")?;
		Ok(())
	}

	/// Problems that would stop Garry's Mod or the Workshop from accepting the addon.
	pub fn problems(&self) -> Vec<String> {
		let mut problems = Vec::new();
		if self.title.trim().is_empty() {
			problems.push("The addon needs a title.".to_owned());
		}
		if !TYPES.contains(&self.addon_type.as_str()) {
			problems.push(format!("\"{}\" isn't a Workshop addon type.", self.addon_type));
		}
		if self.tags.len() > 2 {
			problems.push("An addon can have at most two tags.".to_owned());
		}
		for tag in &self.tags {
			if !TAGS.contains(&tag.as_str()) {
				problems.push(format!("\"{tag}\" isn't a Workshop tag."));
			}
		}
		problems
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_and_validates() {
		let json: AddonJson = serde_json::from_str(r#"{"title":"Test","type":"tool","tags":["fun","build"],"ignore":["*.md"]}"#).unwrap();
		assert_eq!(json.addon_type, "tool");
		assert!(json.problems().is_empty());

		let bad = AddonJson { addon_type: "spaceship".into(), tags: vec!["a".into(), "fun".into(), "build".into()], ..Default::default() };
		assert_eq!(bad.problems().len(), 4);
	}
}

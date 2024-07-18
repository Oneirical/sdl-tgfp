use crate::{
	character::OrdDir,
	spell::{Range, Species},
};
use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug)]
pub struct Vault {
	pub width: usize,

	pub characters: Vec<(i32, i32, Species)>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
	symbols: HashMap<char, Species>,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
	#[error("vault is missing a layout section")]
	MissingLayout,
	#[error("failed to parse metadata: {0}")]
	Toml(#[from] toml::de::Error),
	#[error("unexpected symbol: {0}")]
	UnexpectedSymbol(char),
}

impl Vault {
	/// # Errors
	///
	/// Returns an error if the file could not be opened or parsed.
	pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
		let mut width = 0;

		let vault_text = fs::read_to_string(path).unwrap();

		// FIXME: Make this return 1 variable.
		let (symbols, layout) = vault_text
			.split_once("# Layout\n")
			.ok_or(Error::MissingLayout)?;

		// Before we can do anything, we need to know how wide this vault is.
		for line in layout.lines() {
			width = width.max(line.len());
		}

		let metadata: Metadata = toml::from_str(symbols)?;

		let mut characters = Vec::new();

		for (y, line) in layout.lines().enumerate() {
			for (x, c) in line.chars().enumerate() {
				if let Some(symbol) = metadata.symbols.get(&c) {
					characters.push((x as i32, y as i32, symbol.clone()))
				} else {
					if c != ' ' {
						return Err(Error::UnexpectedSymbol(c));
					}
				}
			}
		}

		Ok(Self { width, characters })
	}
}

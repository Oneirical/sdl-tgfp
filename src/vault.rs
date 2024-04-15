use sdl2::keyboard::Keycode;

use crate::{character::OrdDir, floor::Tile, spell::{Axiom, Range}};
use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug)]
pub struct Vault {
	pub tiles: Vec<Option<Tile>>,
	pub width: usize,

	pub characters: Vec<(i32, i32, String)>,
	pub axioms: Vec<(i32, i32, Axiom)>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum SymbolMeaning {
	Tile(Tile),
	Character(String),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
	symbols: HashMap<char, SymbolMeaning>,
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

		let (metadata, layout) = vault_text
			.split_once("# Layout\n")
			.ok_or(Error::MissingLayout)?;

		let metadata: Metadata = toml::from_str(metadata)?;

		// Before we can do anything, we need to know how wide this vault is.
		for line in layout.lines() {
			width = width.max(line.len());
		}

		let mut tiles = Vec::new();
		let mut characters = Vec::new();
		let mut axioms = Vec::new();

		let axiom_map = HashMap::from([
			('>', Axiom::Keypress(Keycode::Right)),
			('<', Axiom::Keypress(Keycode::Left)),
			('V', Axiom::Keypress(Keycode::Down)),
			('^', Axiom::Keypress(Keycode::Up)),
			('T', Axiom::Teleport),
			('X', Axiom::RadioBroadcaster(Range::Global("EON".to_string()))),
			('N', Axiom::CardinalTargeter(OrdDir::Up)),
			('S', Axiom::CardinalTargeter(OrdDir::Down)),
			('E', Axiom::CardinalTargeter(OrdDir::Right)),
			('O', Axiom::CardinalTargeter(OrdDir::Left)),
			('P', Axiom::SelectSpecies(crate::spell::Species::Terminal)),
		]);

		for (y, line) in layout.lines().enumerate() {
			for (x, c) in line.chars().enumerate() {
				if let Some(action) = metadata.symbols.get(&c) {
					match action {
						SymbolMeaning::Tile(t) => tiles.push(Some(*t)),
						SymbolMeaning::Character(sheet) => {
							characters.push((x as i32, y as i32, sheet.clone()));
							// TODO: What if you want a character standing on something else?
							tiles.push(Some(Tile::Floor));
						}
					}
				} else {
					tiles.push(match c {
						' ' => None,
						'.' => Some(Tile::Floor),
						'x' => Some(Tile::Wall),
						_ => {
							if let Some(axiom) = axiom_map.get(&c) {
								axioms.push((x as i32, y as i32, axiom.clone()));
								Some(Tile::Floor)
							} else {
								return Err(Error::UnexpectedSymbol(c))
							}
						},
					});
				}
			}
			for _ in 0..(width - line.len()) {
				tiles.push(None);
			}
		}

		Ok(Self {
			tiles,
			width,
			characters,
			axioms
		})
	}
}

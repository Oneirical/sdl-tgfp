use crate::floor::Tile;
use crate::prelude::*;
use std::{collections::HashMap, fs, path::Path};

pub struct Set {
	pub vaults: Vec<String>,
	/// Nodes per floor
	pub density: u32,
	/// ratio of halls to vaults.
	pub hall_ratio: i32,
}

#[derive(Clone, Debug)]
pub struct Vault {
	pub tiles: Vec<Option<Tile>>,
	pub width: usize,

	pub characters: Vec<(i32, i32, String)>,
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("vault is missing a layout section")]
	MissingLayout,
	#[error("unexpected symbol: {0}")]
	UnexpectedSymbol(char),
}

impl Vault {
	/// # Errors
	///
	/// Returns an error if the file could not be opened or parsed.
	pub fn open(path: impl AsRef<Path>) -> Result<Self> {
		let mut width = 0;

		let vault_text = fs::read_to_string(path)?;

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
						'>' => Some(Tile::Exit),
						_ => Err(Error::UnexpectedSymbol(c))?,
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
		})
	}
	pub fn cellular_automata() -> Result<Self> {
		let width = 36;
		let mut tiles = Vec::new();
		let characters = Vec::new();
		let mut rng = rand::thread_rng();

		fn xy_idx(x: i32, y: i32, width: i32) -> usize {
			(y * width + x) as usize
		}

		for _y in 1..width - 1 {
			for _x in 1..width - 1 {
				let roll = rng.gen_range(1..100);
				if roll > 55 {
					tiles.push(Some(Tile::Floor));
				} else {
					tiles.push(Some(Tile::Wall));
				}
			}
		}

		for _i in 0..15 {
			let mut newtiles = tiles.clone();

			for y in 1..width - 1 {
				for x in 1..width - 1 {
					let idx = xy_idx(x, y, width);
					let mut neighbors = 0;
					if tiles[idx - 1].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx + 1].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx - width as usize].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx + width as usize].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx - (width as usize - 1)].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx - (width as usize + 1)].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx + (width as usize - 1)].unwrap() == Tile::Wall {
						neighbors += 1;
					}
					if tiles[idx + (width as usize + 1)].unwrap() == Tile::Wall {
						neighbors += 1;
					}

					if neighbors > 4 || neighbors == 0 {
						newtiles[idx] = Some(Tile::Wall);
					} else {
						newtiles[idx] = Some(Tile::Floor);
					}
				}
			}

			tiles = newtiles.clone();
		}

		Ok(Self {
			tiles,
			width: width as usize,
			characters,
		})
	}
}

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
		let (_please_remove_this, layout) = vault_text
			.split_once("# Layout\n")
			.ok_or(Error::MissingLayout)?;

		// Before we can do anything, we need to know how wide this vault is.
		for line in layout.lines() {
			width = width.max(line.len());
		}

		let mut characters = Vec::new();

		let axiom_map = HashMap::from([
			('>', Species::Keypress("Right".to_owned())),
			('<', Species::Keypress("Left".to_owned())),
			('V', Species::Keypress("Down".to_owned())),
			('^', Species::Keypress("Up".to_owned())),
			('T', Species::Teleport),
			('?', Species::Twinning),
			('!', Species::EpsilonHead),
			(
				'(',
				Species::RadioReceiver(Range::Global("EON".to_string())),
			),
			('*', Species::SelectSpecies(Box::new(Species::EpsilonHead))),
			(')', Species::PathfindTargeter(Box::new(Species::Terminal))),
			(
				'X',
				Species::RadioBroadcaster(Range::Global("EON".to_string())),
			),
			('N', Species::CardinalTargeter(OrdDir::Up)),
			('S', Species::CardinalTargeter(OrdDir::Down)),
			('E', Species::CardinalTargeter(OrdDir::Right)),
			('O', Species::CardinalTargeter(OrdDir::Left)),
			('P', Species::SelectSpecies(Box::new(Species::Terminal))),
			('%', Species::SelectSpecies(Box::new(Species::Teleport))),
			('R', Species::Keypress("R".to_owned())),
			('C', Species::Keypress("C".to_owned())),
			('@', Species::SelfTargeter),
			('+', Species::PlusTargeter),
			('Z', Species::RealmShift(1)),
		]);

		for (y, line) in layout.lines().enumerate() {
			for (x, c) in line.chars().enumerate() {
				match c {
					' ' => continue,
					// FIXME: Remove the sheet argument.
					'#' => characters.push((x as i32, y as i32, Species::Wall)),
					_ => {
						if let Some(axiom) = axiom_map.get(&c) {
							characters.push((x as i32, y as i32, axiom.clone()));
						} else {
							return Err(Error::UnexpectedSymbol(c));
						}
					}
				}
			}
		}

		Ok(Self { width, characters })
	}
}

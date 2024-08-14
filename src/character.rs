use std::f64::consts::PI;

use crate::prelude::*;

use self::spell::Species;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Piece {
	// These are nice and serializable :)
	pub species: Species,
	pub x: i32,
	pub y: i32,
	pub z: i32,
	pub momentum: f64,
}

impl Piece {
	pub fn new(sheet: Sheet, resources: &ResourceManager) -> Self {
		Self {
			species: Species::Wall,
			x: 0,
			y: 0,
			z: 0,
			momentum: PI / 2.,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum OrdDir {
	Up,
	Right,
	Down,
	Left,
}

impl OrdDir {
	pub fn as_offset(self) -> (i32, i32) {
		let (x, y) = match self {
			OrdDir::Up => (0, -1),
			OrdDir::Right => (1, 0),
			OrdDir::Down => (0, 1),
			OrdDir::Left => (-1, 0),
		};
		(x, y)
	}
}

/// Anything a character piece can "do".
///
/// This is the only way that character logic or player input should communicate with pieces.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum Action {
	Move(OrdDir),
}

#[derive(Copy, PartialEq, Eq, Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum Alliance {
	Friendly,
	#[default]
	Enemy,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Sheet {
	/// Note that this includes the character's name.
	pub nouns: Nouns,

	pub level: u32,
	pub stats: Stats,
	pub speed: Aut,
	pub texture_id: i32,

	pub attacks: Vec<String>,
	pub spells: Vec<String>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Stats {
	/// Health, or HP; Heart Points
	pub heart: u32,
	/// Magic, or SP; Soul Points
	pub soul: u32,
	/// Bonus damage applied to physical attacks.
	pub power: u32,
	/// Damage reduction when recieving physical attacks.
	pub defense: u32,
	/// Bonus damage applied to magical attacks.
	pub magic: u32,
	/// Damage reduction when recieving magical attacks.
	/// Also makes harmful spells more likely to fail.
	pub resistance: u32,
}

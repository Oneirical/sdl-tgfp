use crate::prelude::*;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Piece {
	// These are nice and serializable :)
	pub id: Uuid,
	pub sheet: Sheet,
	pub x: i32,
	pub y: i32,
	pub next_action: Option<Action>,
	pub player_controlled: bool,
}

impl Piece {
	pub fn new(sheet: Sheet, resources: &ResourceManager) -> Self {

		Self {
			id: Uuid::new_v4(),
			sheet,
			x: 0,
			y: 0,
			next_action: None,
			player_controlled: false,
		}
	}
}

#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
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
	pub skillset: spell::Skillset,
	pub speed: Aut,

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

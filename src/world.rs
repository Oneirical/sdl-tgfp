use spell::{trigger_contingency, ContingencyPacket};

use crate::character::OrdDir;
use crate::prelude::*;
use std::cell::RefCell;

use self::animation::TileEffect;
use self::spell::Species;

pub const WORLD_ROWS: usize = 45;
pub const WORLD_COLS: usize = 45;

pub type CharacterRef = std::rc::Rc<RefCell<character::Piece>>;

/// This struct contains all information that is relevant during gameplay.
#[derive(Clone, Debug)]
pub struct Manager {
	// I know I'm going to have to change this in the future to add multiple worlds.
	/// Where in the world the characters are.
	pub location: Location,
	/// This is the level pointed to by `location.level`.
	pub current_level: Level,
	// It might be useful to sort this by remaining action delay to make selecting the next character easier.
	pub characters: Vec<CharacterRef>,
	// The current player on which the world focuses on
	pub reality_anchor: CharacterRef,
	/// Always point to the party's pieces, even across floors.
	/// When exiting a dungeon, these sheets will be saved to a party struct.
	pub console: Console,
	pub effects: RefCell<Vec<TileEffect>>,
	pub turn_count: RefCell<TurnCounter>,
}

/// Contains information about what should generate on each floor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Level {
	pub name: String,
}

/// Contains the data to dump to a toml save file.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SavePayload {
	pub characters: Vec<CharacterRef>,
	pub reality_anchor: CharacterRef,
	pub turn_count: usize,
}

/// The total number of turns elapsed, incremented with TurnIncrementer.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TurnCounter {
	pub turns: usize,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			name: String::from("New Level"),
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Location {
	/// Which level is currently loaded.
	///
	/// This is usually implicit (see Manager.current_level),
	/// But storing it is important for serialization.
	pub level: String,
	pub floor: usize,
}

impl Manager {
	pub fn dump_characters(&self) {
		let output = toml::to_string(&SavePayload {
			characters: self.characters.clone(),
			reality_anchor: self.reality_anchor.clone(),
			turn_count: self.turn_count.borrow().turns,
		})
		.unwrap();
		std::fs::write("save.toml", output).unwrap();
	}

	pub fn locate_player(&self, payload: &SavePayload) -> Option<&CharacterRef> {
		self.characters.iter().find(|p| {
			let p = p.borrow();
			let compare_anchor = payload.reality_anchor.borrow();
			let (x, y, z) = (compare_anchor.x, compare_anchor.y, compare_anchor.z);
			// Should it ever be possible for multiple creatures to have the same xyz, this will break.
			p.x == x && p.y == y && p.z == z
		})
	}

	pub fn next_character(&self) -> &CharacterRef {
		&self.characters[0]
	}

	// Returns none if no entity is at the specified coordinates.
	pub fn get_character_at(&self, x: i32, y: i32, z: i32) -> Option<&CharacterRef> {
		self.characters.iter().find(|p| {
			let p = p.borrow();
			p.x == x && p.y == y && p.z == z
		})
	}

	pub fn get_characters_of_species(
		&self,
		species: Species,
	) -> impl Iterator<Item = &CharacterRef> + Clone {
		self.characters.iter().filter(move |p| {
			let p = p.borrow();
			p.species == species
		})
	}

	pub fn apply_vault(
		&mut self,
		x: i32,
		y: i32,
		z: i32,
		vault: &Vault,
		resources: &ResourceManager,
	) {
		for (xoff, yoff, species) in &vault.characters {
			let piece = character::Piece {
				x: x + xoff,
				y: y + yoff,
				z,
				species: species.clone(),
				..character::Piece::new(resources.get_sheet("luvui").unwrap().clone(), resources)
			};
			self.characters.push(std::rc::Rc::new(RefCell::new(piece)));
		}
	}
}

#[derive(Clone, Debug)]
pub enum MovementResult {
	Move,
	Attack(AttackResult),
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum MovementError {
	#[error("hit a wall")]
	HitWall,
	#[error("hit the void")]
	HitVoid,
}

#[derive(Clone, Debug)]
pub enum AttackResult {
	Hit { message: String, weak: bool },
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum AttackError {
	#[error("attempted to attack an ally")]
	Ally,
	#[error("attacker has no attacks defined")]
	NoAttacks,
}

impl Manager {
	/// # Errors
	///
	/// Fails if a wall or void is in the way, or if an implicit attack failed.
	pub fn move_piece(
		&self,
		character_ref: &CharacterRef,
		dir: OrdDir,
	) -> Result<MovementResult, MovementError> {
		let (dest_x, dest_y, z) = {
			let (x, y) = dir.as_offset();
			let character = character_ref.borrow();
			(character.x + x, character.y + y, character.z)
		};
		self.teleport_piece(character_ref, dest_x, dest_y, z)
	}

	pub fn teleport_piece(
		&self,
		character_ref: &CharacterRef,
		x: i32,
		y: i32,
		z: i32,
	) -> Result<MovementResult, MovementError> {
		// TODO Preventing the momentum from being warped by the mapwrap
		// by keeping the original coords.
		let (x, y, z) = map_wrap(x, y, z);
		if let Some(collision) = self.get_character_at(x, y, z) {
			let mut character = character_ref.borrow_mut();
			if (character.x, character.y, character.z) == (x, y, z) {
				// Prevent entities from colliding with themselves.
				return Err(MovementError::HitWall);
			}
			let mut coll_character = collision.borrow_mut();
			let (ix, iy) = (character.x, character.y);
			let (dx, dy) = ((x - ix) as f64, (y - iy) as f64);
			// Both the character and the thing being pushed have their momentums changed.
			character.momentum = dy.atan2(dx);
			coll_character.momentum = dy.atan2(dx);
			let collided_species = coll_character.species.clone();
			drop(character);
			drop(coll_character);
			trigger_contingency(
				self,
				&Species::OnCollision(Box::new(collided_species)),
				Some(ContingencyPacket::Collision {
					collided: collision.clone(),
					collider: character_ref.clone(),
				}),
			);
			// TODO outsource all this collision logic to MovementError in Teleport?
			Err(MovementError::HitWall)
		} else {
			let mut character = character_ref.borrow_mut();
			let (ix, iy) = (character.x, character.y);
			character.x = x;
			character.y = y;
			character.z = z;
			let (dx, dy) = ((x - ix) as f64, (y - iy) as f64);
			character.momentum = dy.atan2(dx);
			Ok(MovementResult::Move)
		}
	}
}

pub fn map_wrap(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
	let (x, y) = if x < 0 || y < 0 || x >= WORLD_COLS as i32 || y >= WORLD_ROWS as i32 {
		let (width, height) = (WORLD_COLS as i32, WORLD_ROWS as i32);
		((x + width) % width, (y + height) % height)
	} else {
		(x, y)
	};
	(x, y, z)
}

use crate::character::OrdDir;
use crate::prelude::*;
use std::cell::RefCell;
use uuid::Uuid;

use self::spell::Species;

const DEFAULT_ATTACK_MESSAGE: &str = "{self_Address} attacked {target_indirect}";

pub type CharacterRef = RefCell<character::Piece>;

/// This struct contains all information that is relevant during gameplay.
#[derive(Clone, Debug)]
pub struct Manager {
	// I know I'm going to have to change this in the future to add multiple worlds.
	/// Where in the world the characters are.
	pub location: Location,
	/// This is the level pointed to by `location.level`.
	pub current_level: Level,
	pub current_floor: Floor,
	// It might be useful to sort this by remaining action delay to make selecting the next character easier.
	pub characters: Vec<CharacterRef>,
	pub items: Vec<item::Piece>,
	/// Always point to the party's pieces, even across floors.
	/// When exiting a dungeon, these sheets will be saved to a party struct.
	pub console: Console,
}

/// Contains information about what should generate on each floor.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Level {
	pub name: String,
}

impl Default for Level {
	fn default() -> Self {
		Self {
			name: String::from("New Level"),
		}
	}
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PartyReference {
	/// The piece that is being used by this party member.
	pub piece: Uuid,
	/// This party member's ID within the party.
	/// Used for saving data.
	pub member: Uuid,
}

impl PartyReference {
	pub fn new(piece: Uuid, member: Uuid) -> Self {
		Self { piece, member }
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
	// Returns none if no entity with the given uuid is currently loaded.
	// This either mean they no longer exist, or they're on a different floor;
	// either way they cannot be referenced.
	pub fn get_character(&self, id: Uuid) -> Option<&CharacterRef> {
		self.characters.iter().find(|x| x.borrow().id == id)
	}

	pub fn next_character(&self) -> &CharacterRef {
		&self.characters[0]
	}

	pub fn get_character_at(&self, x: i32, y: i32) -> Option<&CharacterRef> {
		self.characters.iter().find(|p| {
			let p = p.borrow();
			p.x == x && p.y == y
		})
	}

	pub fn get_characters_of_species(&self, species: Species) -> impl Iterator<Item = &CharacterRef> {
		self.characters.iter().filter(move |p| {
			let p = p.borrow();
			p.species == species
		})
	}

	pub fn apply_vault(&mut self, x: i32, y: i32, vault: &Vault, resources: &ResourceManager) {
		self.current_floor.blit_vault(y as usize, x as usize, vault); // Weird swapping. To check when making Pieces and Tiles the same thing.
		for (xoff, yoff, sheet_name) in &vault.characters {
			let piece = character::Piece {
				x: x + xoff,
				y: y + yoff,
				..character::Piece::new(resources.get_sheet(sheet_name).unwrap().clone(), resources)
			};
			self.characters.push(RefCell::new(piece));
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
	pub fn pop_action(&mut self) {
		let next_character = self.next_character();

		let Some(action) = next_character.borrow_mut().next_action.take() else {
			return;
		};
		match action {
			character::Action::Move(dir) => match self.move_piece(next_character, dir) {
				Ok(MovementResult::Attack(AttackResult::Hit { message, weak })) => {
					let colors = &self.console.colors;
					self.console.print_colored(
						message,
						if weak {
							colors.unimportant
						} else {
							colors.normal
						},
					)
				}
				Ok(_) => (),
				Err(MovementError::HitWall) => {
					let name = next_character.borrow().sheet.nouns.name.clone();
					self.console.say(name, "Ouch!");
				}
				Err(MovementError::HitVoid) => {
					self.console.print_system("You stare out into the void: an infinite expanse of nothingness enclosed within a single tile.");
				}
			},
		};
	}

	/// # Errors
	///
	/// Fails if a wall or void is in the way, or if an implicit attack failed.
	pub fn move_piece(
		&self,
		character_ref: &CharacterRef,
		dir: OrdDir,
	) -> Result<MovementResult, MovementError> {
		let (dest_x, dest_y) = {
			let (x, y) = dir.as_offset();
			let character = character_ref.borrow();
			(character.x + x, character.y + y)
		};
		self.teleport_piece(character_ref, dest_x, dest_y)
	}

	pub fn teleport_piece(
		&self,
		character_ref: &CharacterRef,
		x: i32,
		y: i32
	) -> Result<MovementResult, MovementError> {
		use crate::floor::Tile;

		if self.get_character_at(x, y).is_some() { // The Some can be unpacked here if we want to check for collisions.
			return Err(MovementError::HitWall);
		}

		let tile = self.current_floor.map.get(y, x);
		match tile {
			Some(Tile::Floor) => {
				let mut character = character_ref.borrow_mut();
				character.x = x;
				character.y = y;
				Ok(MovementResult::Move)
			}
			Some(Tile::Wall) => Err(MovementError::HitWall),
			None => {
				use crate::floor::WORLD_COLS;
				use crate::floor::WORLD_ROWS;
				let (width, height) = (WORLD_COLS as i32, WORLD_ROWS as i32);
				let (wrap_x, wrap_y) = ((x + width) % width, (y + height) % height);
				self.teleport_piece(character_ref, wrap_x, wrap_y)
			},
		}
	}
}

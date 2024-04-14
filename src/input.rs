use crate::{prelude::*, spell::PlantAxiom};
use sdl2::{
	event::Event,
	keyboard::Scancode,
};

use self::spell::process_axioms;

pub enum Mode {
	Normal,
}

pub struct Result {
	pub exit: bool,
}

pub fn world(
	event_pump: &mut sdl2::EventPump,
	world_manager: &mut world::Manager,
	mode: &mut Mode,
	options: &Options,
) -> Result {
	for event in event_pump.poll_iter() {
		match event {
			Event::Quit { .. }
			| Event::KeyDown {
				scancode: Some(Scancode::Escape),
				..
			} => return Result { exit: true },
			Event::KeyDown {
				keycode: Some(keycode),
				..
			} => {
				//let mut next_character = world_manager.next_character().borrow_mut();
				if true {//next_character.player_controlled {
					match *mode {
						Mode::Normal => {
							// This will need to be refactored.
							if options.controls.left.contains(&(keycode as i32)) {
								let forced_axioms: Vec<PlantAxiom> = vec![
									PlantAxiom {
										x: 0,
										axiom: spell::Axiom::SelectSpecies(spell::Species::Terminal),
										y: 0,
									},
									PlantAxiom {
										x: 1,
										axiom: spell::Axiom::CardinalTargeter(character::OrdDir::Left),
										y: 0,
									},
									PlantAxiom {
										x: 2,
										axiom: spell::Axiom::Teleport,
										y: 0,
									}
								];
								process_axioms(forced_axioms, (0,0), &world_manager);
								//next_character.next_action =
								//	Some(character::Action::Move(character::OrdDir::Left));
							}
							/*
							if options.controls.right.contains(&(keycode as i32)) {
								next_character.next_action =
									Some(character::Action::Move(character::OrdDir::Right));
							}
							if options.controls.up.contains(&(keycode as i32)) {
								next_character.next_action =
									Some(character::Action::Move(character::OrdDir::Up));
							}
							if options.controls.down.contains(&(keycode as i32)) {
								next_character.next_action =
									Some(character::Action::Move(character::OrdDir::Down));
							}
							*/
							//drop(next_character);
						}
					}
				}
			}
			_ => {}
		}
	}

	Result { exit: false }
}

use crate::prelude::*;
use sdl2::{
	event::Event,
	keyboard::Scancode,
};

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
				let mut next_character = world_manager.next_character().borrow_mut();
				if next_character.player_controlled {
					match *mode {
						Mode::Normal => {
							// This will need to be refactored.
							if options.controls.left.contains(&(keycode as i32)) {
								next_character.next_action =
									Some(character::Action::Move(character::OrdDir::Left));
							}
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
							drop(next_character);
						}
					}
				}
			}
			_ => {}
		}
	}

	Result { exit: false }
}

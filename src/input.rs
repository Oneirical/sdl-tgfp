use crate::prelude::*;
use sdl2::{
	event::Event,
	keyboard::{Keycode, Scancode},
};

use self::spell::{process_axioms, Species};

pub enum Mode {
	Normal,
}

pub struct Result {
	pub exit: bool,
}

pub fn world(
	event_pump: &mut sdl2::EventPump,
	world_manager: &world::Manager,
	mode: &Mode,
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
				for axiom in &world_manager.characters {
					if let Species::Keypress(key) = &axiom.borrow().species {
						if Keycode::from_name(key).unwrap() == keycode {
							process_axioms((axiom.borrow().x, axiom.borrow().y), world_manager);
						}
					}
				}
				//let mut next_character = world_manager.next_character().borrow_mut();
			}
			_ => {}
		}
	}

	Result { exit: false }
}

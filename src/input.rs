use crate::prelude::*;
use sdl2::{
	event::Event,
	keyboard::{Keycode, Scancode},
};

use self::spell::{process_axioms, Species, Synapse};

pub enum Mode {
	Normal,
}

pub struct Result {
	pub exit: bool,
}

pub fn world(event_pump: &mut sdl2::EventPump, world_manager: &world::Manager) -> Result {
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
					let axiom = axiom.borrow();
					let (x, y, z, species) = (axiom.x, axiom.y, axiom.z, &axiom.species);
					if let Species::Keypress(key) = species {
						if Keycode::from_name(key).unwrap() == keycode {
							drop(axiom);
							process_axioms(vec![Synapse::new(x, y, z)], world_manager);
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

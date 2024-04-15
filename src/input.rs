use crate::{prelude::*, spell::PlantAxiom};
use sdl2::{
	event::Event,
	keyboard::{Keycode, Scancode},
};

use self::spell::{process_axioms, Axiom};

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
				let forced_axioms: Vec<PlantAxiom> = vec![

				];
				for axiom in &forced_axioms {
					match axiom.axiom {
						Axiom::Keypress(key) => {
							if key == keycode {
								process_axioms(&forced_axioms, (axiom.x, axiom.y), &world_manager);
							}
						}
						_ => ()
					}
				}
				//let mut next_character = world_manager.next_character().borrow_mut();
			}
			_ => {}
		}
	}

	Result { exit: false }
}

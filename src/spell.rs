use std::{borrow::Borrow, cell::RefCell};

use sdl2::keyboard::Keycode;

use crate::{
	character::{OrdDir, Piece},
	world::Manager,
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Spell {
	pub name: String,
	pub icon: i32,
	pub lore: String,
	pub description: String,
}

pub fn match_axiom_with_codename(axiom: &Species) -> Option<&str> {
	let code = match axiom {
		Species::Keypress(_) => "keypress",
		Species::CardinalTargeter(_) => "cardinal_targeter",
		Species::SelectSpecies(_) => "select_species",
		Species::RadioBroadcaster(_) => "radio_broadcaster",
		Species::RadioReceiver(_) => "radio_receiver",
		Species::Teleport => "teleport",
		_ => "",
	};
	if code.is_empty() {
		None
	} else {
		Some(code)
	}
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Range {
	Targeted(String),
	Contained(String),
	Local(String),
	Global(String),
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Species {
	Wall,
	Terminal,
	WorldStem,

	// AXIOMS

	// Contingencies
	Keypress(String),
	RadioReceiver(Range),

	// Anointers
	SelectSpecies(Box<Species>),

	// Forms
	CardinalTargeter(OrdDir),

	// Functions
	Teleport,
	RadioBroadcaster(Range),
}

pub fn process_axioms(pulse: (i32, i32), manager: &Manager) {
	let mut casters = Vec::new();
	let mut pulse = vec![pulse];
	let mut visited = Vec::new();

	while let Some(current_pulse) = pulse.pop() {
		let curr_axiom = match manager.get_character_at(current_pulse.0, current_pulse.1) {
			Some(axiom) => axiom,
			None => continue,
		};
		match &curr_axiom.borrow().species {
			Species::Keypress(_) => (),
			// Anoint all creatures of a given Species.
			Species::SelectSpecies(species) => {
				let found = manager.get_characters_of_species(*species.clone());
				for creature in found {
					casters.push((creature, Vec::new()));
				}
			}
			// Target an adjacent tile to each Caster.
			Species::CardinalTargeter(dir) => {
				for (caster, ref mut targets) in casters.iter_mut() {
					let caster = caster.borrow_mut();
					let offset = dir.as_offset();
					targets.push((caster.x + offset.0, caster.y + offset.1));
				}
			}
			// Teleport each Caster to its closest Target.
			Species::Teleport => {
				for (caster, targets) in casters.iter() {
					let targets: Vec<_> = targets
						.iter()
						.filter_map(|(x, y)| {
							if manager.get_character_at(*x, *y).is_none() {
								Some((*x, *y))
							} else {
								None
							}
						})
						.collect();
					let b_caster = caster.borrow_mut();
					let (cx, cy) = (b_caster.x, b_caster.y);
					drop(b_caster);
					if let Some((x, y)) = find_closest_coordinate(&targets, (cx, cy)) {
						let _ = manager.teleport_piece(caster, x, y);
					}
				}
			}
			Species::RadioBroadcaster(output_range) => match output_range {
				Range::Global(output_message) => {
					for axiom in &manager.characters {
						let axiom = &axiom.borrow();
						if let Species::RadioReceiver(input_range) = &axiom.species {
							let input_message = match input_range {
								Range::Global(input_message) => input_message,
								_ => todo!(),
							};
							if *output_message == *input_message {
								process_axioms((axiom.x, axiom.y), manager);
							}
						}
					}
				}
				_ => (), // Any non-Axiom species causes the chain to break
			},
			_ => continue,
		}
		for adjacency in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
			let new_pulse = (current_pulse.0 + adjacency.0, current_pulse.1 + adjacency.1);
			if !visited.contains(&new_pulse) {
				pulse.push(new_pulse);
				visited.push(new_pulse);
			}
		}
	}
}

fn manhattan_distance(a: (i32, i32), b: (i32, i32)) -> i32 {
	(a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn find_closest_coordinate(coordinates: &[(i32, i32)], target: (i32, i32)) -> Option<(i32, i32)> {
	let mut min_distance = i32::MAX;
	let mut closest_coordinate = None;

	for &coordinate in coordinates {
		let distance = manhattan_distance(coordinate, target);
		if distance < min_distance {
			min_distance = distance;
			closest_coordinate = Some(coordinate);
		}
	}

	closest_coordinate
}

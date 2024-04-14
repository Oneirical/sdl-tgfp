use sdl2::keyboard::Scancode;

use crate::{character::{OrdDir, Piece}, world::Manager};
use tracing::error;


#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Spell {
	pub name: String,
	pub icon: String,
	pub lore: String,
	pub description: String,
}

pub struct PlantAxiom {
	pub axiom: Axiom,
	pub x: i32,
	pub y: i32,
}

pub enum Axiom {
	// Contingencies
	Keypress(Scancode),
	RadioReceiver(Range),

	// Anointers
	SelectSpecies(Species),

	// Forms
	CardinalTargeter(OrdDir),

	// Functions
	Teleport,
	RadioBroadcaster(Range),
}

pub enum Range {
	Targeted,
	Contained,
	Local,
	Global,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Species {
	Wall,
	Terminal,
	WorldStem,
}

fn process_axioms(
	axiom_grid: Vec<PlantAxiom>,
	pulse: (i32, i32),
	manager: &Manager,
) {
	let mut casters = Vec::new();
	let mut pulse = vec![pulse];
	let mut visited = Vec::new();
	
	loop {
		let current_pulse = match pulse.pop() {
			Some(coords) => coords,
			None => break,
		};
		let curr_axiom = match axiom_grid.iter().find(|plant_axiom| (plant_axiom.x, plant_axiom.y) == current_pulse) {
			Some(axiom) => axiom,
			None => continue,
		};
		match &curr_axiom.axiom {
			// Anoint all creatures of a given Species.
			Axiom::SelectSpecies (species) => {
				let found = manager.get_characters_of_species(species.clone());
				for creature in found {
					casters.push((creature, Vec::new()));
				}
				
			}
			// Target an adjacent tile to each Caster.
			Axiom::CardinalTargeter(dir) => {
				for (caster, ref mut targets) in casters.iter_mut() {
					let caster = caster.borrow();
					let offset = dir.as_offset();
					targets.push((caster.x + offset.0, caster.y + offset.1));
				}
			}
			// Teleport each Caster to its closest Target.
			Axiom::Teleport => {
				for (caster, targets) in casters.iter_mut() {
					let targets: Vec<_> = targets.iter()
					.filter_map(|(x, y)| {
						if manager.get_character_at(*x, *y).is_none() {
							Some((*x, *y))
						} else {
							None
						}
					}).collect();
					let b_caster = caster.borrow();
					let (cx, cy) = (b_caster.x, b_caster.y);
					if let Some((x,y)) = find_closest_coordinate(&targets, (cx, cy)) {
						let _ = manager.teleport_piece(caster, x, y);
					}
				}
			}
			_ => ()
		}
		for adjacency in [(0,-1),(-1,0), (1, 0), (0, 1)] {
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

    for &coordinate in coordinates.iter() {
        let distance = manhattan_distance(coordinate, target);
        if distance < min_distance {
            min_distance = distance;
            closest_coordinate = Some(coordinate);
        }
    }

    closest_coordinate
}
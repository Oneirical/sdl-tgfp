use crate::{
	character::OrdDir,
	world::{map_wrap, CharacterRef, Manager},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Spell {
	pub name: String,
	pub icon: i32,
	pub lore: String,
	pub description: String,
}

#[derive(Clone, Debug)]
struct CasterTarget {
	caster: CharacterRef,
	targets: Vec<(i32, i32, i32)>,
}

impl CasterTarget {
	pub fn new(caster: CharacterRef, targets: Vec<(i32, i32, i32)>) -> Self {
		CasterTarget { caster, targets }
	}
}

#[derive(Clone, Debug)]
pub struct Synapse {
	casters: Vec<CasterTarget>,
	momentum: OrdDir,
	pulse: (i32, i32, i32),
}

impl Synapse {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Synapse {
			casters: Vec::new(),
			momentum: OrdDir::Up,
			pulse: (x, y, z),
		}
	}
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
	EpsilonHead,
	EpsilonTail(usize),
	// AXIOMS

	// Contingencies
	Keypress(String),
	RadioReceiver(Range),

	// Anointers
	SelectSpecies(Box<Species>),
	AnointToTarget(Box<Species>),

	// Forms
	PathfindTargeter(Box<Species>),
	CardinalTargeter(OrdDir),
	PlusTargeter,
	SelfTargeter,

	// Mutators
	RealmShift(i32),
	// Functions
	Teleport,
	Twinning,
	RadioBroadcaster(Range),
}

pub fn process_axioms(mut synapses: Vec<Synapse>, manager: &Manager) {
	let mut visited = Vec::new();
	while !synapses.is_empty() {
		let mut syn_count = 0;
		for mut synapse in synapses.clone() {
			let (pulse_x, pulse_y, pulse_z) =
				map_wrap(synapse.pulse.0, synapse.pulse.1, synapse.pulse.2);
			let curr_axiom = match manager.get_character_at(pulse_x, pulse_y, pulse_z) {
				Some(axiom) => axiom,
				None => continue,
			};
			let curr_ax_species = curr_axiom.borrow().species.clone();
			dbg!(&curr_ax_species);
			match &curr_ax_species {
				Species::Keypress(_) => (),
				// Anoint all creatures of a given Species.
				Species::SelectSpecies(species) => {
					// should this be restricted to Z level?
					let found = manager.get_characters_of_species(*species.clone());
					for creature in found {
						synapse
							.casters
							.push(CasterTarget::new(creature.clone(), Vec::new()));
					}
				}
				Species::AnointToTarget(species) => {
					let mut new_targets = Vec::new();
					for CasterTarget { caster, targets: _ } in synapse.casters.iter_mut() {
						let caster = caster.borrow_mut();
						new_targets.push((caster.x, caster.y, caster.z)); // Grab the position of every caster
						drop(caster);
					}
					synapse.casters.clear(); // Remove all casters and their targets
						 // should this be restricted to Z level?
					let found = manager.get_characters_of_species(*species.clone());
					for creature in found {
						synapse.casters.push(CasterTarget {
							caster: creature.clone(),
							targets: new_targets.clone(),
						});
					}
				}
				// Target an adjacent tile to each Caster.
				Species::CardinalTargeter(dir) => {
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let caster = caster.borrow_mut();
						let offset = dir.as_offset();
						targets.push(map_wrap(caster.x + offset.0, caster.y + offset.1, caster.z));
						drop(caster);
					}
				}
				Species::PathfindTargeter(species) => {
					let found = manager.get_characters_of_species(*species.clone());
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let mut chosen = None;
						let mut distance = i32::MAX;
						for entity in found.clone() {
							let candidate = entity.borrow();
							let cand_coords = (candidate.x, candidate.y, candidate.z);
							let caster = caster.borrow();
							let new_dist = // Find the closest representative of Species.
								manhattan_distance(cand_coords, (caster.x, caster.y, caster.z));
							if new_dist < distance {
								chosen = Some(cand_coords);
								distance = new_dist;
							}
						}
						if let Some(chosen) = chosen {
							let caster = caster.borrow();
							let pot_targets = &[
								(caster.x + 1, caster.y, caster.z),
								(caster.x - 1, caster.y, caster.z),
								(caster.x, caster.y - 1, caster.z),
								(caster.x, caster.y + 1, caster.z),
							];
							// TODO: Pathfind through map wrapping too.
							let pot_targets = filter_targets_by_unoccupied(manager, pot_targets);
							let new_tar = find_closest_coordinate(&pot_targets, chosen);
							if let Some(new_tar) = new_tar {
								targets.push(new_tar);
							}
						}
					}
				}
				// Target all orthogonal tiles to each Caster.
				Species::PlusTargeter => {
					let offsets = [(-1, 0), (1, 0), (0, 1), (0, -1)];
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let caster = caster.borrow_mut();
						for i in 0..3 {
							let offset = offsets[i];
							targets.push(map_wrap(
								caster.x + offset.0,
								caster.y + offset.1,
								caster.z,
							));
						}
						drop(caster);
					}
				}
				// Target the tiles on which the Casters stand on.
				Species::SelfTargeter => {
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let caster = caster.borrow_mut();
						targets.push((caster.x, caster.y, caster.z)); // No need for map_wrap, this always stays inbounds
						drop(caster);
					}
				}
				// All Targets's Z coordinate get shifted to `realm`.
				Species::RealmShift(realm) => {
					for CasterTarget { caster: _, targets } in synapse.casters.iter_mut() {
						for tar in targets {
							tar.2 = *realm;
						}
					}
				}
				// Transform each Target into a clone of the Caster.
				Species::Twinning => {
					for CasterTarget { caster, targets } in synapse.casters.iter() {
						let cas_species = caster.borrow().species.clone();
						for (x, y, z) in targets {
							if let Some(victim) = manager.get_character_at(*x, *y, *z) {
								victim.borrow_mut().species = cas_species.clone();
							}
						}
					}
				}
				// Teleport each Caster to its closest Target.
				Species::Teleport => {
					for CasterTarget { caster, targets } in synapse.casters.iter() {
						let targets = filter_targets_by_unoccupied(manager, &targets);
						let b_caster = caster.borrow_mut();
						let (cx, cy, cz) = (b_caster.x, b_caster.y, b_caster.z);
						drop(b_caster);
						if let Some((x, y, z)) = find_closest_coordinate(&targets, (cx, cy, cz)) {
							let _ = manager.teleport_piece(&caster, x, y, z);
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
									process_axioms(
										vec![Synapse::new(axiom.x, axiom.y, axiom.z)],
										manager,
									);
								}
							}
						}
					}
					_ => (),
				},
				_ => (), // Any non-Axiom species
			}
			let mut potential_new_axioms = Vec::new();
			let search_order = generate_clockwise_rotation(synapse.momentum); // Starting from the direction we come from, rotate clockwise.
			let search_order_ints = search_order.iter().map(|x| x.as_offset());
			for (i, adjacency) in search_order_ints.enumerate() {
				let (new_pulse_x, new_pulse_y, new_pulse_z) =
					map_wrap(pulse_x + adjacency.0, pulse_y + adjacency.1, pulse_z);
				dbg!(map_wrap(
					pulse_x + adjacency.0,
					pulse_y + adjacency.1,
					pulse_z
				));
				dbg!(manager
					.get_character_at(new_pulse_x, new_pulse_y, new_pulse_z)
					.is_some());
				dbg!(&visited);
				if manager // Must contain an entity and not have been visited before.
					.get_character_at(new_pulse_x, new_pulse_y, new_pulse_z)
					.is_some() && !visited.contains(&(new_pulse_x, new_pulse_y, new_pulse_z))
				{
					dbg!("ha");
					visited.push((new_pulse_x, new_pulse_y, new_pulse_z));
					potential_new_axioms.push((
						search_order.get(i).unwrap(),
						(new_pulse_x, new_pulse_y, new_pulse_z),
					));
				}
			}
			if potential_new_axioms.is_empty() {
				synapses.remove(syn_count);
			} else {
				synapse.momentum = *potential_new_axioms[0].0;
				synapse.pulse = potential_new_axioms[0].1;
				for new_synapse in 1..potential_new_axioms.len() - 1 {
					synapses.push(Synapse {
						casters: synapse.casters.clone(),
						momentum: *potential_new_axioms[new_synapse].0,
						pulse: potential_new_axioms[new_synapse].1,
					})
				}
			}
			syn_count += 1;
		}
	}
}

fn manhattan_distance(a: (i32, i32, i32), b: (i32, i32, i32)) -> i32 {
	(a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs()
}

fn generate_clockwise_rotation(start: OrdDir) -> [OrdDir; 4] {
	fn rotate(dir: OrdDir) -> OrdDir {
		match dir {
			OrdDir::Up => OrdDir::Right,
			OrdDir::Right => OrdDir::Down,
			OrdDir::Down => OrdDir::Left,
			OrdDir::Left => OrdDir::Up,
		}
	}
	[
		start,
		rotate(start),
		rotate(rotate(start)),
		rotate(rotate(rotate(start))),
	]
}

fn find_closest_coordinate(
	coordinates: &[(i32, i32, i32)],
	target: (i32, i32, i32),
) -> Option<(i32, i32, i32)> {
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

fn filter_targets_by_unoccupied(
	manager: &Manager,
	targets: &[(i32, i32, i32)],
) -> Vec<(i32, i32, i32)> {
	targets
		.iter()
		.filter_map(|(x, y, z)| {
			if manager.get_character_at(*x, *y, *z).is_none() {
				Some((*x, *y, *z))
			} else {
				None
			}
		})
		.collect()
}

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
	Synaptic(String),
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
	SelectRealityAnchor,

	// Forms
	PathfindTargeter(Box<Species>),
	CardinalTargeter(OrdDir),
	PlusTargeter,
	SelfTargeter,
	MomentumBeam,

	// Mutators
	RealmShift(i32),
	ClearThisCaster(Box<Species>),
	// Functions
	Teleport,
	Twinning,
	SwapAnchor,
	RadioBroadcaster(Range),
}

// Suggestion: store casters as Uuids instead of the actual Piece structs.
pub fn process_axioms(mut synapses: Vec<Synapse>, manager: &Manager) {
	let mut visited = Vec::new();
	while !synapses.is_empty() {
		let mut syn_count = 0;
		// Create a temporary vector to hold new synapses
		let mut new_synapses = Vec::new();
		let mut synapses_to_remove = Vec::new();
		for synapse in &mut synapses {
			let (pulse_x, pulse_y, pulse_z) =
				map_wrap(synapse.pulse.0, synapse.pulse.1, synapse.pulse.2);
			visited.push((pulse_x, pulse_y, pulse_z));
			let curr_axiom = match manager.get_character_at(pulse_x, pulse_y, pulse_z) {
				Some(axiom) => axiom,
				None => continue,
			};
			let curr_ax_species = curr_axiom.borrow().species.clone();
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
				Species::SelectRealityAnchor => {
					let player = manager
						.get_player_character()
						.expect("The player does not exist");
					synapse
						.casters
						.push(CasterTarget::new(player.clone(), Vec::new()));
				}
				// All casters in the synapse turn into targets for the `species`.
				Species::AnointToTarget(species) => {
					let mut new_targets = Vec::new();
					for CasterTarget { caster, targets: _ } in synapse.casters.iter() {
						let caster = caster.borrow();
						new_targets.push((caster.x, caster.y, caster.z)); // Grab the position of every caster
						drop(caster);
					}
					// synapse.casters.clear(); // Remove all casters and their targets
					// should this be restricted to Z level?
					let found = manager.get_characters_of_species(*species.clone());
					for creature in found {
						synapse.casters.push(CasterTarget {
							caster: creature.clone(),
							targets: new_targets.clone(),
						});
					}
				}
				// Remove all caster/targets pairs where the caster is `species`.
				Species::ClearThisCaster(species) => {
					let mut remove_indices = Vec::new();
					let mut count = 0;
					for CasterTarget { caster, targets: _ } in synapse.casters.iter() {
						let caster = caster.borrow();
						if caster.species == *species.clone() {
							remove_indices.push(count);
						}
						count += 1;
					}
					for i in remove_indices {
						synapse.casters.remove(i);
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
				// All Targets's Z coordinates get shifted to `realm`.
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
							// This will return an intentional error if a collision happens.
							// This collision could be used for a cool Contingency, like starting
							// dialogue.
							let _ = manager.teleport_piece(&caster, x, y, z);
						}
					}
				}
				// Swap the reality-anchor state of the Caster with its closest Target.
				Species::SwapAnchor => {
					for CasterTarget { caster, targets } in synapse.casters.iter() {
						// Only targets with an entity should be candidates.
						let targets = filter_targets_by_occupied(manager, &targets);
						let b_caster = caster.borrow_mut();
						let (cx, cy, cz) = (b_caster.x, b_caster.y, b_caster.z);
						// Find the closest entity that's on a target.
						if let Some((x, y, z)) = find_closest_coordinate(&targets, (cx, cy, cz)) {
							let mut reality_anchor = manager.reality_anchor.borrow_mut();
							let caster_id = b_caster.id.clone();
							drop(b_caster);
							// If the caster is the anchor, give the anchor to the target.
							if caster_id.eq(&reality_anchor) {
								*reality_anchor =
									manager.get_character_at(x, y, z).unwrap().borrow().id;
							} else if manager
								// But if the target is the anchor, steal their anchor for the caster.
								.get_character_at(x, y, z)
								.unwrap()
								.borrow()
								.id
								.eq(&reality_anchor)
							{
								*reality_anchor = caster_id;
							}
						}
					}
				}
				Species::RadioBroadcaster(output_range) => match output_range {
					Range::Global(output_message) => {
						for axiom in &manager.characters {
							let axiom = axiom.borrow();
							if let Species::RadioReceiver(input_range) = &axiom.species {
								let (synapse_transmission, input_message) = match input_range {
									Range::Global(input_message) => (
										vec![Synapse::new(axiom.x, axiom.y, axiom.z)],
										input_message,
									),
									Range::Synaptic(input_message) => (
										// Continues the synapse to the new destination.
										vec![Synapse {
											casters: synapse.casters.clone(),
											momentum: synapse.momentum,
											pulse: (axiom.x, axiom.y, axiom.z),
										}],
										input_message,
									),
									_ => todo!(),
								};
								let current_z = manager.get_player_character().unwrap().borrow().z;
								if *output_message == *input_message && axiom.z <= current_z {
									// It can only broadcast to local or upper layers

									// Important to get this axiom out of scope as the new synapse
									// could use it
									drop(axiom);
									process_axioms(synapse_transmission, manager);
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
				if manager // Must contain an entity and not have been visited before.
					.get_character_at(new_pulse_x, new_pulse_y, new_pulse_z)
					.is_some() && !visited.contains(&(new_pulse_x, new_pulse_y, new_pulse_z))
				{
					potential_new_axioms.push((
						search_order.get(i).unwrap(),
						(new_pulse_x, new_pulse_y, new_pulse_z),
					));
				}
			}
			if potential_new_axioms.is_empty() {
				synapses_to_remove.push(syn_count);
			} else {
				synapse.momentum = *potential_new_axioms[0].0;
				synapse.pulse = potential_new_axioms[0].1;
				for new_synapse in 1..potential_new_axioms.len() {
					new_synapses.push(Synapse {
						casters: synapse.casters.clone(),
						momentum: *potential_new_axioms[new_synapse].0,
						pulse: potential_new_axioms[new_synapse].1,
					})
				}
			}
			syn_count += 1;
		}

		// Remove marked synapses from the original vector
		for count in synapses_to_remove.into_iter().rev() {
			synapses.remove(count);
		}

		// Add new synapses to the original vector
		for synapse in new_synapses.drain(..) {
			synapses.push(synapse);
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

fn filter_targets_by_occupied(
	manager: &Manager,
	targets: &[(i32, i32, i32)],
) -> Vec<(i32, i32, i32)> {
	targets
		.iter()
		.filter_map(|(x, y, z)| {
			if manager.get_character_at(*x, *y, *z).is_some() {
				Some((*x, *y, *z))
			} else {
				None
			}
		})
		.collect()
}

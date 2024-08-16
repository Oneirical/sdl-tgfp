use std::cell::RefCell;

use crate::{
	animation::TileEffect,
	character::OrdDir,
	world::{map_wrap, CharacterRef, Manager, SavePayload},
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
	visited: Vec<(i32, i32, i32)>,
}

impl Synapse {
	pub fn new(x: i32, y: i32, z: i32) -> Self {
		Synapse {
			casters: Vec::new(),
			momentum: OrdDir::Up,
			pulse: (x, y, z),
			visited: Vec::new(),
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
	WatchBot,
	// AXIOMS

	// Contingencies
	Keypress(String),
	RadioReceiver(Range),
	OnTurn,

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
	SpecificCoord((i32, i32, i32)),

	// Mutators
	RealmShift(i32),
	ClearThisCaster(Box<Species>),
	Orbit(usize),
	Halo(usize),

	// Functions
	Teleport,
	Twinning,
	SwapAnchor,
	RadioBroadcaster(Range),
	Fireworks,
	SaveGame,
	LoadGame,
	TurnIncrementer,
}

pub struct Result {
	pub new_manager: Option<crate::world::Manager>,
}

pub fn trigger_contingency(world_manager: &Manager, contingency: &Species) -> Result {
	let mut new_manager = None;
	for axiom in &world_manager.characters {
		let axiom = axiom.borrow();
		let (x, y, z, species) = (axiom.x, axiom.y, axiom.z, &axiom.species);
		if species == contingency {
			match contingency {
				Species::OnTurn => {
					drop(axiom);
					new_manager =
						process_axioms(vec![Synapse::new(x, y, z)], world_manager).new_manager;
				}
				_ => (),
			}
		}
	}
	Result { new_manager }
}

pub fn process_axioms(mut synapses: Vec<Synapse>, manager: &Manager) -> Result {
	let mut new_manager = None;
	let mut loop_danger_count = 0;
	while !synapses.is_empty() {
		loop_danger_count += 1;
		if loop_danger_count > 500 {
			panic!("Infinite loop in axioms (use this for something cool later!)");
		}
		let mut syn_count = 0;
		// Create a temporary vector to hold new synapses
		let mut new_synapses = Vec::new();
		let mut synapses_to_remove = Vec::new();
		for synapse in &mut synapses {
			let (pulse_x, pulse_y, pulse_z) =
				map_wrap(synapse.pulse.0, synapse.pulse.1, synapse.pulse.2);
			synapse.visited.push((pulse_x, pulse_y, pulse_z));
			let curr_axiom = match manager.get_character_at(pulse_x, pulse_y, pulse_z) {
				Some(axiom) => axiom,
				None => {
					// Would this happen if a location was spread to, then made unavailable?
					// FIXME This is very suspicious.
					synapses_to_remove.push(syn_count);
					syn_count += 1;
					continue;
				}
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
					let player = &manager.reality_anchor;
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
					for (count, CasterTarget { caster, targets: _ }) in
						synapse.casters.iter().enumerate()
					{
						let caster = caster.borrow();
						if caster.species == *species.clone() {
							remove_indices.push(count);
						}
					}
					for i in remove_indices {
						synapse.casters.remove(i);
					}
				}
				// Target this specific coordinate.
				Species::SpecificCoord((x, y, z)) => {
					for CasterTarget { caster: _, targets } in synapse.casters.iter_mut() {
						targets.push(map_wrap(*x, *y, *z));
					}
				}
				// Target an adjacent tile to each Caster.
				Species::CardinalTargeter(dir) => {
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let caster = caster.borrow();
						let offset = dir.as_offset();
						targets.push(map_wrap(caster.x + offset.0, caster.y + offset.1, caster.z));
						drop(caster);
					}
				}
				// Target the adjacent tile closest to the nearest representative of `species`.
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
						for offset in offsets {
							targets.push(map_wrap(
								caster.x + offset.0,
								caster.y + offset.1,
								caster.z,
							));
						}
					}
				}
				// Target the tiles on which the Casters stand on.
				Species::SelfTargeter => {
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let caster = caster.borrow();
						targets.push((caster.x, caster.y, caster.z)); // No need for map_wrap, this always stays inbounds
					}
				}
				// Target tiles with a beam shooting from the Caster in the direction of their momentum.
				Species::MomentumBeam => {
					for CasterTarget { caster, targets } in synapse.casters.iter_mut() {
						let caster = caster.borrow();
						let mut beam = beam_from_point(
							manager,
							caster.momentum,
							(caster.x, caster.y, caster.z),
						);
						targets.append(&mut beam);
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
				// Each target becomes the centre of a circle of `radius`, and is replaced
				// by a new target on that circle corresponding to the turn count.
				// NOTE: Could be cool to add an `arc_length` for slashes.
				Species::Orbit(radius) => {
					for CasterTarget { caster: _, targets } in synapse.casters.iter_mut() {
						for tar in targets {
							let mut circle = circle_around(tar, *radius as i32);
							// Sort by clockwise rotation.
							circle.sort_by(|a, b| {
								let angle_a = angle_from_center(tar, a);
								let angle_b = angle_from_center(tar, b);
								angle_a.partial_cmp(&angle_b).unwrap()
							});
							let circle: Vec<(i32, i32, i32)> =
								circle.iter().map(|p| map_wrap(p.0, p.1, p.2)).collect();
							// "% circle.len()" so that bigger circles are slower to traverse. May need adaptation.
							let offset = manager.turn_count.borrow().turns % circle.len();
							let orbit_point = circle
								.get(offset)
								.expect("The measured offset was out of bounds");
							(tar.0, tar.1, tar.2) = (orbit_point.0, orbit_point.1, orbit_point.2);
						}
					}
				}
				// Each target becomes the centre of a circle of `radius`, and is replaced
				// by new targets all around that circle's outline.
				Species::Halo(radius) => {
					let mut halo = Vec::new();
					for CasterTarget { caster: _, targets } in synapse.casters.iter_mut() {
						for tar in &*targets {
							let mut circle = circle_around(tar, *radius as i32);
							// Sort by clockwise rotation.
							circle.sort_by(|a, b| {
								let angle_a = angle_from_center(tar, a);
								let angle_b = angle_from_center(tar, b);
								angle_a.partial_cmp(&angle_b).unwrap()
							});
							halo.append(&mut circle);
						}
						targets.append(&mut halo);
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
						let targets = filter_targets_by_unoccupied(manager, targets);
						let b_caster = caster.borrow_mut();
						let (cx, cy, cz) = (b_caster.x, b_caster.y, b_caster.z);
						drop(b_caster);
						if let Some((x, y, z)) = find_closest_coordinate(&targets, (cx, cy, cz)) {
							// This will return an intentional error if a collision happens.
							// This collision could be used for a cool Contingency, like starting
							// dialogue.
							let _ = manager.teleport_piece(caster, x, y, z);
						}
					}
				}
				// Dump the world state to save.toml.
				Species::SaveGame => {
					manager.dump_characters();
				}
				// Rewind the world state as it is stored in save.toml.
				Species::LoadGame => {
					if std::path::Path::new("save.toml").exists() {
						let saved_chars = std::fs::read_to_string("save.toml").unwrap();
						let saved_manager: SavePayload = toml::from_str(&saved_chars).unwrap();
						let new_characters = saved_manager.characters.clone();
						// Find the player among the cloned characters.
						// We need to do this because the main.rs loop uses as_ptr.
						let new_anchor = new_characters
							.iter()
							.find(|p| {
								let p = p.borrow();
								let compare_anchor = saved_manager.reality_anchor.borrow();
								let (x, y, z) =
									(compare_anchor.x, compare_anchor.y, compare_anchor.z);
								// Should it ever be possible for multiple creatures to have the same xyz, this will break.
								p.x == x && p.y == y && p.z == z
							})
							.expect("The player did not exist in the save file")
							.clone();
						new_manager = Some(Manager {
							current_level: manager.current_level.clone(),
							characters: new_characters,
							reality_anchor: new_anchor,
							console: manager.console.clone(),
							effects: manager.effects.clone(),
							location: manager.location.clone(),
							turn_count: RefCell::new(crate::world::TurnCounter {
								turns: saved_manager.turn_count,
							}),
						});
					}
				}
				// Add a fading tile effect to each Target.
				Species::Fireworks => {
					for CasterTarget { caster: _, targets } in synapse.casters.iter() {
						for tar in targets {
							manager.effects.borrow_mut().push(TileEffect {
								x: tar.0,
								y: tar.1,
								z: tar.2,
								alpha: 255,
								texture: crate::animation::EffectType::Red,
							});
						}
					}
				}
				// Swap the reality-anchor state of the Caster with its closest Target.
				Species::SwapAnchor => {
					for CasterTarget { caster, targets } in synapse.casters.iter() {
						// Only targets with an entity should be candidates.
						let targets = filter_targets_by_occupied(manager, targets);
						let b_caster = caster.borrow();
						let (cx, cy, cz) = (b_caster.x, b_caster.y, b_caster.z);
						// Find the closest entity that's on a target.
						if let Some((x, y, z)) = find_closest_coordinate(&targets, (cx, cy, cz)) {
							let anchor_ptr = manager.reality_anchor.as_ptr();
							drop(b_caster);
							// If the caster is the anchor, give the anchor to the target.
							if caster.as_ptr().eq(&anchor_ptr) {
								let new_anchor = manager.get_character_at(x, y, z).unwrap();
								if new_anchor.as_ptr() == caster.as_ptr() {
									// Do not swap the caster with itself - this will crash the game.
									continue;
								}
								manager.reality_anchor.swap(new_anchor);
							} else if manager
								// But if the target is the anchor, steal their anchor for the caster.
								.get_character_at(x, y, z)
								.unwrap()
								.as_ptr()
								.eq(&anchor_ptr)
							{
								manager.reality_anchor.swap(caster);
							}
						}
					}
				}
				Species::TurnIncrementer => {
					let mut turn_counter = manager.turn_count.borrow_mut();
					turn_counter.turns += 1;
					drop(turn_counter);
					new_manager = trigger_contingency(manager, &Species::OnTurn).new_manager;
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
											visited: synapse.visited.clone(),
										}],
										input_message,
									),
									_ => todo!(),
								};
								let current_z = manager.reality_anchor.borrow().z;
								if *output_message == *input_message && axiom.z <= current_z {
									// It can only broadcast to local or upper layers

									// Important to get this axiom out of scope as the new synapse
									// could use it
									drop(axiom);
									new_manager =
										process_axioms(synapse_transmission, manager).new_manager;
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
					.is_some() && !synapse
					.visited
					.contains(&(new_pulse_x, new_pulse_y, new_pulse_z))
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
						visited: synapse.visited.clone(),
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
	Result { new_manager }
}

fn manhattan_distance(a: (i32, i32, i32), b: (i32, i32, i32)) -> i32 {
	(a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs()
}

/// From `start`, find the four rotations in a clockwise direction.
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

/// Find the tile with the shortest Manhattan distance to `target`.
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

/// Generate the points across the outline of a circle.
fn circle_around(center: &(i32, i32, i32), radius: i32) -> Vec<(i32, i32, i32)> {
	let mut circle = Vec::new();
	for r in 0..=(radius as f32 * (0.5f32).sqrt()).floor() as i32 {
		let d = (((radius * radius - r * r) as f32).sqrt()).floor() as i32;
		let adds = [
			((center.0 - d, center.1 + r, center.2)),
			((center.0 + d, center.1 + r, center.2)),
			((center.0 - d, center.1 - r, center.2)),
			((center.0 + d, center.1 - r, center.2)),
			((center.0 + r, center.1 - d, center.2)),
			((center.0 + r, center.1 + d, center.2)),
			((center.0 - r, center.1 - d, center.2)),
			((center.0 - r, center.1 + d, center.2)),
		];
		for new_add in adds {
			if !circle.contains(&new_add) {
				circle.push(new_add);
			}
		}
	}
	circle
}

/// Find the angle of a point on a circle relative to its center.
fn angle_from_center(center: &(i32, i32, i32), point: &(i32, i32, i32)) -> f64 {
	let delta_x = point.0 - center.0;
	let delta_y = point.1 - center.1;
	(delta_y as f64).atan2(delta_x as f64)
}

/// Remove all targets containing a creature.
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

/// Remove all targets NOT containing a creature.
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

/// Return all tiles in the path of a beam that stops at the first encountered creature.
fn beam_from_point(manager: &Manager, angle: f64, origin: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
	let increments = (angle.cos(), angle.sin());
	let mut scale = 1.;
	let mut out = Vec::new();
	loop {
		let new_point = (
			origin.0 as f64 + increments.0 * scale,
			origin.1 as f64 + increments.1 * scale,
		);
		let selected_tile = (
			new_point.0.round() as i32,
			new_point.1.round() as i32,
			origin.2,
		);
		// This wrapping could cause an infinite loop... if there was absolutely no entity in the loop path.
		let selected_tile = map_wrap(selected_tile.0, selected_tile.1, selected_tile.2);
		out.push(selected_tile);
		if manager
			.get_character_at(selected_tile.0, selected_tile.1, selected_tile.2)
			.is_some()
		{
			break;
		}
		scale += 1.;

		if scale > 100. {
			// Hardcoded maximum range, to avoid an infinite loop.
			break;
		}
	}
	out
}

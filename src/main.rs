use character::Piece;
use sdl2::{pixels::Color, rect::Rect, rwops::RWops};
use sdltgfp::options::{RESOURCE_DIRECTORY, USER_DIRECTORY};
use sdltgfp::prelude::*;
use sdltgfp::spell::match_axiom_with_codename;
use sdltgfp::world::{WORLD_COLS, WORLD_ROWS};
use std::cell::RefCell;
use std::f32::consts::PI;
use std::process::exit;
use tracing::*;

fn update_delta(
	last_time: &mut f64,
	current_time: &mut f64,
	timer_subsystem: &sdl2::TimerSubsystem,
) -> f64 {
	*last_time = *current_time;
	*current_time = timer_subsystem.performance_counter() as f64;
	((*current_time - *last_time) * 1000.0
				    / (timer_subsystem.performance_frequency() as f64))
				    // Convert milliseconds to seconds.
				    / 1000.0
}

pub fn main() {
	// SDL initialization.
	let sdl_context = sdl2::init().unwrap();
	let ttf_context = sdl2::ttf::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let timer_subsystem = sdl_context.timer().unwrap();
	let window = video_subsystem
		.window("SDL TGFP", 1280, 720)
		.resizable()
		.position_centered()
		.build()
		.unwrap();

	let mut canvas = window
		.into_canvas()
		.accelerated()
		.present_vsync()
		.build()
		.unwrap();
	let texture_creator = canvas.texture_creator();
	let mut event_pump = sdl_context.event_pump().unwrap();

	let mut current_time = timer_subsystem.performance_counter() as f64;
	let mut last_time = current_time;

	// Logging initialization.
	tracing_subscriber::fmt::init();

	// Game initialization.
	let resources = match ResourceManager::open(&*RESOURCE_DIRECTORY, &texture_creator) {
		Ok(resources) => resources,
		Err(msg) => {
			error!("Failed to open resource directory: {msg}");
			exit(1);
		}
	};
	let options = Options::open(USER_DIRECTORY.join("options.toml")).unwrap_or_else(|msg| {
		error!("failed to open options.toml: {msg}");
		Options::default()
	});
	// Create a piece for the player, and register it with the world manager.
	let player = character::Piece {
		species: spell::Species::Terminal,
		..character::Piece::new(resources.get_sheet("luvui").unwrap().clone(), &resources)
	};
	let player_piece = std::rc::Rc::new(RefCell::new(player));
	let mut world_manager = world::Manager {
		location: world::Location {
			level: String::from("New Level"),
			floor: 0,
		},
		console: Console::default(),
		reality_anchor: player_piece.clone(),

		current_level: world::Level::default(),
		characters: Vec::new(),
		effects: RefCell::new(Vec::new()),
		turn_count: RefCell::new(world::TurnCounter { turns: 0 }),
	};

	if std::path::Path::new("save.toml").exists() {
		let saved_chars = std::fs::read_to_string("save.toml").unwrap();
		let saved_manager: world::SavePayload = toml::from_str(&saved_chars).unwrap();
		world_manager.characters = saved_manager.characters.clone();
		// Locate the player among the saved characters.
		world_manager.reality_anchor = world_manager
			.locate_player(&saved_manager)
			.expect("The player did not exist in the save file")
			.clone();
		world_manager.turn_count = RefCell::new(world::TurnCounter {
			turns: saved_manager.turn_count,
		});
	} else {
		world_manager.characters.push(player_piece);
		//world_manager.characters.push(CharacterRef::new(ally));
		world_manager.apply_vault(
			0,
			0,
			0,
			resources.get_vault("world_roots").unwrap(),
			&resources,
		);
		world_manager.apply_vault(0, 0, 1, resources.get_vault("lower").unwrap(), &resources);
	}

	let mut spritesheet = resources.get_owned_texture("spritesheet").unwrap();
	let font = ttf_context
		.load_font_from_rwops(
			RWops::from_bytes(include_bytes!(
				"res/FantasqueSansMNerdFontPropo-Regular.ttf"
			))
			.unwrap(),
			options.ui.font_size,
		)
		.unwrap();

	// Print some debug messages to test the console.
	world_manager.console.print("Hello, world!");

	// TODO: Display this on-screen.
	let mut input_mode = input::Mode::Normal;
	let mut global_time = 0;
	let mut zoom_amount = 0;
	loop {
		// Input processing
		let input_result = input::world(&mut event_pump, &mut world_manager);
		let (exit, new_manager) = (input_result.exit, input_result.new_manager);
		if exit {
			break;
		};
		if let Some(new_manager) = new_manager {
			world_manager = new_manager;
		}

		// Logic
		// This is the only place where delta time should be used.
		{
			let delta = update_delta(&mut last_time, &mut current_time, &timer_subsystem);
			world_manager.console.update(delta);
		}

		// Rendering
		// Clear the screen.
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();

		// Configure world viewport.
		let window_size = canvas.window().size();
		let side_dim = window_size.0
			- options.ui.pamphlet_width
			- options.ui.left_pamphlet_width
			- options.ui.padding * 2;
		// zoom_amount += if global_time % 1 == 0 { 1 } else { 0 };
		let tiles_in_viewport = side_dim / (options.ui.tile_size);
		canvas.set_viewport(Rect::new(
			options.ui.left_pamphlet_width as i32 + options.ui.padding as i32,
			options.ui.padding as i32,
			side_dim,
			side_dim,
		));
		global_time += 1;
		canvas.set_draw_color(Color::BLACK);
		canvas
			.fill_rect(Rect::new(0, 0, window_size.0, window_size.1))
			.unwrap();

		// Draw tilemap
		canvas.set_draw_color(Color::WHITE);

		let wi_width = window_size.0
			- options.ui.pamphlet_width
			- options.ui.left_pamphlet_width
			- options.ui.padding * 2;
		let wi_height = window_size.1 - options.ui.console_height - options.ui.padding * 2;
		let mut curr_xy = (0, 0);
		let mut curr_z = 0;
		let (world_width, world_height) = (
			(WORLD_COLS * options.ui.tile_size as usize) as i32,
			(WORLD_ROWS * options.ui.tile_size as usize) as i32,
		);
		let areas = [
			(0, 0),
			(world_width, 0),
			(-world_width, 0),
			(0, world_height),
			(0, -world_height),
			(world_width, world_height),
			(-world_width, world_height),
			(world_width, -world_height),
			(-world_width, -world_height),
		];

		for character in world_manager.characters.iter() {
			let character_ptr = character.as_ptr();
			let character = character.borrow();
			if character_ptr == world_manager.reality_anchor.as_ptr() {
				curr_xy = (character.x, character.y);
				curr_z = character.z;
			}
		}
		// for character in world_manager.characters.iter().map(|x| x.borrow()) {
		// 	let (x, y) = (
		// 		(character.x - curr_xy.0 + wi_width as i32 / 2 / 8) * (zoom_amount + 8),
		// 		(character.y - curr_xy.1 + wi_height as i32 / 2 / 8) * (zoom_amount + 8),
		// 	);
		// 	let mut texture_y = 0;
		// 	let texture_x = match character.species {
		// 		spell::Species::Wall => 3,
		// 		spell::Species::Terminal => 0,
		// 		_ => {
		// 			// It could be an axiom.
		// 			let axiom_name = match_axiom_with_codename(&character.species);
		// 			if let Some(axiom_name) = axiom_name {
		// 				texture_y = 16;
		// 				resources.get_spell(axiom_name).unwrap().icon
		// 			} else {
		// 				// Fallback "missing texture" for unknown species.
		// 				1
		// 			}
		// 		}
		// 	} * 16;
		// 	let source_rect = Rect::new(texture_x, texture_y, 16, 16);
		// 	for (off_x, off_y) in areas {
		// 		canvas
		// 			.copy(
		// 				spritesheet,
		// 				Some(source_rect),
		// 				Some(Rect::new(
		// 					off_x + x - zoom_amount * 8,
		// 					off_y + y - zoom_amount * 8,
		// 					8 + zoom_amount as u32,
		// 					8 + zoom_amount as u32,
		// 				)),
		// 			)
		// 			.unwrap();
		// 	}
		// }
		// Draw characters (normal)
		for character in world_manager.characters.iter() {
			let character_ptr = character.as_ptr();
			let character = character.borrow();
			if curr_z as i32 != character.z {
				continue;
			}
			let (x, y) = if character_ptr == world_manager.reality_anchor.as_ptr() {
				curr_xy = (character.x, character.y);
				(
					((tiles_in_viewport / 2) * (options.ui.tile_size)) as i32,
					((tiles_in_viewport / 2 - 1) * (options.ui.tile_size)) as i32,
				)
			} else {
				(
					(character.x - curr_xy.0 + wi_width as i32 / 2 / options.ui.tile_size as i32)
						* (zoom_amount + options.ui.tile_size as i32),
					(character.y - curr_xy.1 + wi_height as i32 / 2 / options.ui.tile_size as i32)
						* (zoom_amount + options.ui.tile_size as i32),
				)
			};
			let mut texture_y = 0;
			let texture_x = match character.species {
				spell::Species::Wall => 3,
				spell::Species::Terminal => 0,
				spell::Species::WatchBot => 8,
				spell::Species::EpsilonHead => 67,
				spell::Species::PushCrate => 7,
				spell::Species::EpsilonTail(_) => 68,
				_ => {
					// It could be an axiom.
					let axiom_name = match_axiom_with_codename(&character.species);
					if let Some(axiom_name) = axiom_name {
						texture_y = 16;
						resources.get_spell(axiom_name).unwrap().icon
					} else {
						// Fallback "missing texture" for unknown species.
						1
					}
				}
			} * 16;
			let source_rect = Rect::new(texture_x, texture_y, 16, 16);
			for (off_x, off_y) in areas {
				if character_ptr == world_manager.reality_anchor.as_ptr() {
					// Prevent the main character from being drawn multiple times for the "looping world" effect.
					if (off_x, off_y) != (0, 0) {
						continue;
					}
					canvas
						.copy(
							&spritesheet,
							Some(source_rect),
							Some(Rect::new(x, y, options.ui.tile_size, options.ui.tile_size)),
						)
						.unwrap();
					continue;
				}
				canvas
					.copy(
						&spritesheet,
						Some(source_rect),
						Some(Rect::new(
							off_x + x - zoom_amount * 16,
							off_y + y - zoom_amount * 16,
							options.ui.tile_size + zoom_amount as u32,
							options.ui.tile_size + zoom_amount as u32,
						)),
					)
					.unwrap();
			}
		}
		// Draw fading tile effects
		for effect in world_manager.effects.borrow_mut().iter_mut() {
			let (x, y) = (
				(effect.x - curr_xy.0 + wi_width as i32 / 2 / options.ui.tile_size as i32)
					* (zoom_amount + options.ui.tile_size as i32),
				(effect.y - curr_xy.1 + wi_height as i32 / 2 / options.ui.tile_size as i32)
					* (zoom_amount + options.ui.tile_size as i32),
			);
			let texture_x = match effect.texture {
				animation::EffectType::Red => 14,
				animation::EffectType::Lime => 13,
			} * 16;
			let source_rect = Rect::new(texture_x, 0, 16, 16);
			spritesheet.set_alpha_mod(effect.alpha);
			for (off_x, off_y) in areas {
				canvas
					.copy(
						&spritesheet,
						Some(source_rect),
						Some(Rect::new(
							off_x + x - zoom_amount * 16,
							off_y + y - zoom_amount * 16,
							options.ui.tile_size + zoom_amount as u32,
							options.ui.tile_size + zoom_amount as u32,
						)),
					)
					.unwrap();
			}
			effect.alpha = effect.alpha.checked_sub(10).unwrap_or_default();
		}
		// Restore opacity after effects loop.
		spritesheet.set_alpha_mod(255);
		// Render User Interface
		canvas.set_viewport(None);

		// Draw Console
		world_manager.console.draw(
			&mut canvas,
			Rect::new(
				(window_size.0 - options.ui.pamphlet_width) as i32,
				(window_size.1 - options.ui.console_height) as i32,
				options.ui.pamphlet_width,
				options.ui.console_height,
			),
			&font,
		);

		// Draw pamphlet
		pamphlet(
			&mut canvas,
			window_size,
			&options,
			&font,
			&world_manager,
			&resources,
		);

		canvas.present();
	}
}

fn pamphlet(
	canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
	window_size: (u32, u32),
	options: &Options,
	font: &sdl2::ttf::Font<'_, '_>,
	world_manager: &world::Manager,
	resources: &ResourceManager<'_>,
) {
	let mut left_pamphlet = gui::Context::new(
		canvas,
		Rect::new(0, 0, options.ui.left_pamphlet_width, window_size.1),
	);
	let mut minimap_fn = |left_pamphlet: &mut gui::Context| {
		let chains = get_chain_border(6, window_size.1 as usize / 16 - 1);
		let mut chains = chains.iter().peekable();
		left_pamphlet.horizontal();
		while chains.peek().is_some() {
			if let Some(chain) = chains.next() {
				left_pamphlet.set(
					(chain.position.0 * 64.) as i32,
					(chain.position.1 * 32.) as i32,
				);
				left_pamphlet.htexture_ex(
					resources.get_texture(chain.sprite.clone()),
					32,
					chain.rotation as f64 * 57.3,
				);
			}
		}
	};
	left_pamphlet.hsplit(&mut [Some((&mut minimap_fn) as &mut dyn FnMut(&mut gui::Context))]);
	let (px, py) = ((window_size.0 - options.ui.pamphlet_width) as i32, 0);
	let mut pamphlet = gui::Context::new(
		canvas,
		Rect::new(
			(window_size.0 - options.ui.pamphlet_width) as i32,
			0,
			options.ui.pamphlet_width,
			window_size.1,
		),
	);

	let mut inventory_fn = |pamphlet: &mut gui::Context| {
		let chains = get_chain_border(12, 24);
		let mut chains = chains.iter().peekable();
		pamphlet.horizontal();
		while chains.peek().is_some() {
			if let Some(chain) = chains.next() {
				pamphlet.set(
					(chain.position.0 * 64.) as i32 + px,
					(chain.position.1 * 32.) as i32 + py,
				);
				pamphlet.htexture_ex(
					resources.get_texture(chain.sprite.clone()),
					32,
					chain.rotation as f64 * 57.3,
				);
			}
		}
	};
	let mut log_fn = |pamphlet: &mut gui::Context| {
		let chains = get_chain_border(12, window_size.1 as usize / 16 - 27);
		let mut chains = chains.iter().peekable();
		pamphlet.horizontal();
		while chains.peek().is_some() {
			if let Some(chain) = chains.next() {
				pamphlet.set(
					(chain.position.0 * 64.) as i32 + px,
					(chain.position.1 * 32.) as i32 + 13 * 32,
				);
				pamphlet.htexture_ex(
					resources.get_texture(chain.sprite.clone()),
					32,
					chain.rotation as f64 * 57.3,
				);
			}
		}
	};
	pamphlet.hsplit(&mut [
		Some((&mut inventory_fn) as &mut dyn FnMut(&mut gui::Context)),
		Some((&mut log_fn) as &mut dyn FnMut(&mut gui::Context)),
	]);
}

fn get_chain_border(width: usize, height: usize) -> Vec<ChainIcon> {
	let offset = (width as f32 / 2., height as f32 / 2.);
	let points = (0..width).flat_map(|x| (0..height).map(move |y| (x, y)));
	let border_points =
		points.filter(|&(x, y)| x == 0 || y == 0 || x == width - 1 || y == height - 1);
	let chain_icons = border_points.map(|(x, y)| {
		let chain = match (x, y) {
			(0, 0) => ChainType::TopLeft,
			(0, y) if y == height - 1 => ChainType::BotLeft,
			(x, 0) if x == width - 1 => ChainType::TopRight,
			(x, y) if x == width - 1 && y == height - 1 => ChainType::BotRight,
			_ => match (x, y) {
				(0, _) => ChainType::Left,
				(x, _) if x == width - 1 => ChainType::Right,
				(_, 0) => ChainType::Top,
				_ => ChainType::Bot,
			},
		};

		let sprite = if [
			ChainType::TopLeft,
			ChainType::TopRight,
			ChainType::BotLeft,
			ChainType::BotRight,
		]
		.contains(&chain)
		{
			"corner_chain".into()
		} else {
			"lateral_chain".into()
		};

		let rotation = match chain {
			ChainType::TopLeft => 0.,
			ChainType::BotLeft => 3. * PI / 2.,
			ChainType::TopRight => PI / 2.,
			ChainType::BotRight => PI,
			ChainType::Left => 0.,
			ChainType::Right => PI,
			ChainType::Top => PI / 2.,
			ChainType::Bot => 3. * PI / 2.,
			_ => panic!("Wrong chain type!"),
		};

		ChainIcon {
			sprite,
			rotation,
			position: (
				(x as f32 - width as f32 / 2. + offset.0) / 2.,
				(y as f32 - height as f32 / 2. + offset.1) / 2.,
			),
		}
	});
	chain_icons.collect()
}

#[derive(PartialEq)]
enum ChainType {
	TopLeft,
	TopRight,
	BotLeft,
	BotRight,
	Top,
	Right,
	Left,
	Bot,
	EndLeft,
	EndRight,
}

#[derive(Debug)]
struct ChainIcon {
	sprite: String,
	rotation: f32,
	position: (f32, f32),
}

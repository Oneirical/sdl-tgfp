use esprit2::floor::{WORLD_COLS, WORLD_ROWS};
use esprit2::options::{RESOURCE_DIRECTORY, USER_DIRECTORY};
use esprit2::prelude::*;
use esprit2::world::CharacterRef;
use sdl2::render::Texture;
use sdl2::{pixels::Color, rect::Rect, rwops::RWops};
use std::f32::consts::PI;
use std::process::exit;
use tracing::*;
use uuid::Uuid;

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

struct SoulJar<'texture> {
	souls: Vec<Soul>,
	light_texture: Texture<'texture>,
}

impl SoulJar<'_> {
	fn tick(&mut self, delta: f32) {
		for i in &mut self.souls {
			i.tick(delta);
		}
	}
}

pub fn main() {
	// SDL initialization.
	let sdl_context = sdl2::init().unwrap();
	let ttf_context = sdl2::ttf::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let timer_subsystem = sdl_context.timer().unwrap();
	let window = video_subsystem
		.window("Esprit 2", 1280, 720)
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
	let party = [
		(
			Uuid::new_v4(),
			resources.get_sheet("luvui").unwrap().clone(),
		),
		(Uuid::new_v4(), resources.get_sheet("aris").unwrap().clone()),
	];
	let player = character::Piece {
		player_controlled: true,
		..character::Piece::new(party[0].1.clone(), &resources)
	};
	let mut world_manager = world::Manager {
		location: world::Location {
			level: String::from("New Level"),
			floor: 0,
		},
		console: Console::default(),

		current_level: world::Level::default(),
		current_floor: Floor::default(),
		characters: Vec::new(),
		items: Vec::new(),
	};
	world_manager.characters.push(CharacterRef::new(player));
	//world_manager.characters.push(CharacterRef::new(ally));
	world_manager.apply_vault(0, 11, resources.get_vault("example").unwrap(), &resources);
	let sleep_texture = resources.get_texture("luvui_sleep");
	let spritesheet = resources.get_texture("spritesheet");
	let font = ttf_context
		.load_font_from_rwops(
			RWops::from_bytes(include_bytes!(
				"res/FantasqueSansMNerdFontPropo-Regular.ttf"
			))
			.unwrap(),
			options.ui.font_size,
		)
		.unwrap();

	let mut soul_jar = SoulJar {
		souls: vec![
			Soul::new(Color::RED),
			Soul::new(Color::YELLOW),
			Soul::new(Color::BLUE),
			Soul::new(Color::GREEN),
			Soul::new(Color::CYAN),
			Soul::new(Color::RGB(255, 128, 0)),
			Soul::new(Color::WHITE),
			Soul::new(Color::RGB(255, 0, 255)),
			Soul::new(Color::RGB(255, 128, 128)),
		],
		light_texture: resources.get_owned_texture("light").unwrap(),
	};

	// Print some debug messages to test the console.
	world_manager.console.print("Hello, world!");
	world_manager.console.print("Luvui scratches the cat.");
	world_manager.console.print_defeat("The cat ran away.");
	world_manager.console.print("Luvui casts Magic Missile.");
	world_manager
		.console
		.print("Her magic missile strikes the cat!");
	world_manager.console.print("The cat scratches Aris");
	world_manager.console.print("Aris bites the cat");
	world_manager.console.print_defeat("The cat scampered off.");
	world_manager
		.console
		.print_special("Luvui's level increased to 2!");

	// TODO: Display this on-screen.
	let mut input_mode = input::Mode::Normal;
	let mut global_time = 0;
	loop {
		// Input processing
		if input::world(
			&mut event_pump,
			&mut world_manager,
			&mut input_mode,
			&options,
		)
		.exit
		{
			break;
		};

		// Logic
		// This is the only place where delta time should be used.
		{
			let delta = update_delta(&mut last_time, &mut current_time, &timer_subsystem);

			world_manager.pop_action();
			world_manager.console.update(delta);
			soul_jar.tick(delta as f32);
		}

		// Rendering
		// Clear the screen.
		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();

		// Configure world viewport.
		let window_size = canvas.window().size();
		canvas.set_viewport(Rect::new(
			options.ui.left_pamphlet_width as i32 + options.ui.padding as i32,
			options.ui.padding as i32,
			window_size.0 - options.ui.pamphlet_width - options.ui.left_pamphlet_width - options.ui.padding * 2,
			window_size.1 - options.ui.console_height - options.ui.padding * 2,
		));
		global_time = (global_time + 1) % 255;
		canvas.set_draw_color(Color::BLUE);
		canvas
			.fill_rect(Rect::new(0, 0, window_size.0, window_size.1))
			.unwrap();

		// Draw tilemap
		canvas.set_draw_color(Color::WHITE);

		let wi_width = window_size.0 - options.ui.pamphlet_width - options.ui.left_pamphlet_width - options.ui.padding * 2;
		let wi_height = window_size.1 - options.ui.console_height - options.ui.padding * 2;
		let mut curr_xy = (0,0);
		// Draw characters
		for character in world_manager.characters.iter().map(|x| x.borrow()) {
			let (x, y) = if character.player_controlled {
				curr_xy = (character.x, character.y);
				(wi_width as i32 / 2, wi_height as i32 / 2)
			} else {
				(0, 0)
			};
			let texture_x = character.sheet.texture_id * 16;
			let source_rect = Rect::new(texture_x, 0, 16, 16);
			canvas
				.copy(
					&spritesheet,
					Some(source_rect),
					Some(Rect::new(x, y, options.ui.tile_size, options.ui.tile_size)),
				)
				.unwrap();
		}

		for (x, col) in world_manager.current_floor.map.iter_cols().enumerate() {
			for (y, tile) in col.enumerate() {
				if *tile == floor::Tile::Wall {
					let (world_width, world_height) = (WORLD_COLS as i32, WORLD_ROWS as i32);
					let areas = [(0,0),(world_width, 0), (-world_width, 0), (0, world_height), (0, -world_height), (world_width, world_height), (-world_width, world_height), (world_width, -world_height), (-world_width, -world_height)];
					for (off_x, off_y) in areas {
						let source_rect = Rect::new(48, 0, 16, 16);
						canvas.copy(&spritesheet, Some(source_rect), Some(
							Rect::new(
								(off_x + x as i32 - curr_xy.0 + wi_width as i32 / 2 / options.ui.tile_size as i32) * options.ui.tile_size as i32, 
								(off_y + y as i32 - curr_xy.1 + wi_height as i32 / 2 / options.ui.tile_size as i32) * options.ui.tile_size as i32, 
								options.ui.tile_size,
								options.ui.tile_size
							)
						)).unwrap();
					}
				}
			}
		}

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
			&mut soul_jar,
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
	soul_jar: &mut SoulJar<'_>,
) {
	let mut left_pamphlet = gui::Context::new(
		canvas,
		Rect::new(
			0,
			0,
			options.ui.left_pamphlet_width,
			window_size.1,
		),
	);
	let mut minimap_fn = |left_pamphlet: &mut gui::Context| {
		let chains = get_chain_border(6,window_size.1 as usize / 16 - 1);
  		let mut chains = chains.iter().peekable();
		left_pamphlet.horizontal();
		while chains.peek().is_some() {
			if let Some(chain) = chains.next() {
				left_pamphlet.set((chain.position.0 * 64.) as i32, (chain.position.1 * 32.) as i32);
				left_pamphlet.htexture_ex(resources.get_texture(chain.sprite.clone()), 32, chain.rotation as f64 * 57.3);
			}
		}
	};
	left_pamphlet.hsplit(&mut [
		Some((&mut minimap_fn) as &mut dyn FnMut(&mut gui::Context)),
	]);
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
		let chains = get_chain_border(12,24);
  		let mut chains = chains.iter().peekable();
		pamphlet.horizontal();
		while chains.peek().is_some() {
			if let Some(chain) = chains.next() {
				pamphlet.set((chain.position.0 * 64.) as i32 + px, (chain.position.1 * 32.) as i32 + py);
				pamphlet.htexture_ex(resources.get_texture(chain.sprite.clone()), 32, chain.rotation as f64 * 57.3);
			}
		}
	};
	let mut log_fn = |pamphlet: &mut gui::Context| {
		let chains = get_chain_border(12,window_size.1 as usize / 16 - 27);
  		let mut chains = chains.iter().peekable();
		pamphlet.horizontal();
		while chains.peek().is_some() {
			if let Some(chain) = chains.next() {
				pamphlet.set((chain.position.0 * 64.) as i32 + px, (chain.position.1 * 32.) as i32 + 13 * 32);
				pamphlet.htexture_ex(resources.get_texture(chain.sprite.clone()), 32, chain.rotation as f64 * 57.3);
			}
		}
	};
	pamphlet.hsplit(&mut [
		Some((&mut inventory_fn) as &mut dyn FnMut(&mut gui::Context)),
		Some((&mut log_fn) as &mut dyn FnMut(&mut gui::Context)),
	]);
}

fn get_chain_border(
    width: usize,
    height: usize,
) -> Vec<ChainIcon> {
	let offset = (width as f32 / 2., height as f32 / 2.);
    let points = (0..width).flat_map(|x| (0..height).map(move |y| (x, y)));
    let border_points = points.filter(|&(x, y)| x == 0 || y == 0 || x == width - 1 || y == height - 1);
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

        let sprite = if [ChainType::TopLeft, ChainType::TopRight, ChainType::BotLeft, ChainType::BotRight].contains(&chain) {
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
            position: ((x as f32 - width as f32 / 2. + offset.0) / 2., (y as f32 - height as f32 / 2. + offset.1) / 2.),
        }
    });
    chain_icons.collect()
}



#[derive(PartialEq)]
enum ChainType{
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
struct ChainIcon{
    sprite: String,
    rotation: f32,
    position: (f32, f32)
}
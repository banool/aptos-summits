use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_prototype_lyon::prelude::*;
use clap::Parser;
use rand::SeedableRng;
use rand::{rngs::SmallRng, Rng};
use sha2::{Digest, Sha256};

const WIDTH: f32 = 1600.0;

// TODO: Make the clap stuff conditional behind a feature.
#[derive(Clone, Debug, Parser)]
pub struct AppConfig {
    #[clap(long, default_value_t = 1600.)]
    pub width: f32,

    // TODO: If we can make the Rust SDK less massive, use AccountAddress instead.
    #[clap(long)]
    pub token_address: String,
}

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub html_canvas_id: String,
}

#[derive(Resource)]
pub struct AppSeed {
    pub token_address: String,
}

impl AppConfig {
    pub fn run(self, web_config: Option<WebConfig>) {
        let mut app = App::new();

        let resolution = WindowResolution::new(self.width, self.width);
        let window = match web_config {
            Some(web_config) => Window {
                resolution,
                canvas: Some(web_config.html_canvas_id),
                fit_canvas_to_parent: true,
                ..default()
            },
            None => Window {
                resolution,
                ..default()
            },
        };

        // From https://bevy-cheatbook.github.io/platforms/wasm/webpage.html#custom-canvas.
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }))
        .insert_resource(AppSeed {
            token_address: self.token_address,
        })
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup_system);

        app.run();
    }
}

fn setup_system(mut commands: Commands, app_seed: Res<AppSeed>) {
    // Convert the token address into a u64 for the seed.
    let mut hasher = Sha256::new();
    hasher.update(&app_seed.token_address);
    let result = hasher.finalize();
    let first_eight_bytes = &result[0..8];
    let seed = u64::from_be_bytes(first_eight_bytes.try_into().unwrap());

    // Build deterministic rng with seed.
    let mut rng = SmallRng::seed_from_u64(seed);

    // Generate mountains.
    let mountains = vec![Mountain::new(&mut rng, WIDTH as u32, 50., 300.)];

    // Draw mountains.
    for mountain in mountains {
        draw_mountain(&mut commands, &mountain);
    }
}

fn draw_mountain(commands: &mut Commands, mountain: &Mountain) {
    let mut path_builder = PathBuilder::new();

    for (i, y) in mountain.heights().iter().enumerate() {
        let x = (i as i32 - WIDTH as i32) as f32;
        let point = Vec2::new(x, *y);
        path_builder.line_to(point);
    }

    path_builder.close();
    let path = path_builder.build();

    commands.spawn(Camera2dBundle::default());
    commands.spawn((ShapeBundle { path, ..default() }, Fill::color(Color::RED)));
}

#[derive(Component)]
struct Mountain {
    heights: Vec<f32>,
}

impl Mountain {
    pub fn new(rng: &mut SmallRng, width: u32, min_height: f64, max_height: f64) -> Self {
        let step_max = rng.gen_range(0.9..1.1);
        let step_change = rng.gen_range(0.15..0.35);
        let mut height = rng.gen_range(0.0..max_height);
        let mut slope = rng.gen_range(0.0..step_max) * 2.0 - step_max;
        let mut heights: Vec<f32> = Vec::new();

        for _ in 0..(width * 2) {
            height = height + slope;
            slope = slope + (rng.gen_range(0.0..step_change) * 2.0 - step_change);

            if slope > step_max {
                slope = step_max;
            } else if slope < -step_max {
                slope = -step_max;
            }

            if height > max_height {
                height = max_height;
                slope = slope * -1.0;
            } else if height < min_height {
                height = min_height;
                slope = slope * -1.0;
            }
            heights.push(height as f32);
        }
        Mountain { heights }
    }

    pub fn heights(&self) -> &[f32] {
        &self.heights
    }
}

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_prototype_lyon::prelude::*;
use clap::Parser;
use rand::SeedableRng;
use rand::{rngs::SmallRng, Rng};
use sha2::{Digest, Sha256};
use std::ops::Range;

const WIDTH: f32 = 1200.0;

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

fn rand_color(rng: &mut SmallRng, r: Range<u8>, g: Range<u8>, b: Range<u8>) -> Color {
    Color::rgb_u8(rng.gen_range(r), rng.gen_range(g), rng.gen_range(b))
}

fn interpolate(left: Color, right: Color, left_weight: f32) -> Color {
    let right_weight = 1.0 - left_weight;

    let red = left.r() * left_weight + right.r() * right_weight;
    let green = left.g() * left_weight + right.g() * right_weight;
    let blue = left.b() * left_weight + right.b() * right_weight;
    let alpha = left.a() * left_weight + right.a() * right_weight;

    Color::rgba(red, green, blue, alpha)
}

fn setup_system(mut commands: Commands, window: Query<&Window>, app_seed: Res<AppSeed>) {
    // Convert the token address into a u64 for the seed.
    let mut hasher = Sha256::new();
    hasher.update(&app_seed.token_address);
    let result = hasher.finalize();
    let first_eight_bytes = &result[0..8];
    let seed = u64::from_be_bytes(first_eight_bytes.try_into().unwrap());

    // Build deterministic rng with seed.
    let mut rng = SmallRng::seed_from_u64(seed);

    // Generate sky color.
    let sky_color = match rng.gen_range(1..4) {
        1 => rand_color(&mut rng, 1..40, 1..40, 1..40),
        2 => rand_color(&mut rng, 215..225, 215..225, 230..255),
        _ => rand_color(&mut rng, 200..255, 200..255, 200..255),
    };

    // Generate fog color.
    // let fog_color = rand_color(&mut rng, 1..255, 1..255, 1..255);

    // TODO: Add stuff in sky.

    // Spawn the camera with our sky color as the background
    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(sky_color),
        },
        ..default()
    });

    let window = window.single();
    let height = window.resolution.height() as f64;

    // Generate mountains.
    let mut mountains = Vec::new();
    let num_mountains: u64 = rng.gen_range(3..7);
    let mountain_base_color = rand_color(&mut rng, 1..255, 1..255, 1..255);
    for i in 0..num_mountains {
        let color = interpolate(
            mountain_base_color,
            sky_color,
            (i + 1) as f32 / num_mountains as f32,
        );
        let min_height = -height / 3. / (num_mountains * (num_mountains - i)) as f64;
        let max_height = height * 0.72;
        let mountain = Mountain::new(&mut rng, WIDTH as u32, min_height, max_height, color);
        mountains.push(mountain);
    }

    // Draw mountains.
    let mut z = 1.;
    for mountain in mountains {
        mountain.draw(&mut commands, z, &window.resolution);
        z += 1.;
    }
}

#[derive(Component)]
struct Mountain {
    heights: Vec<f32>,
    color: Color,
}

impl Mountain {
    pub fn new(
        rng: &mut SmallRng,
        width: u32,
        min_height: f64,
        max_height: f64,
        color: Color,
    ) -> Self {
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
        Mountain { heights, color }
    }

    pub fn draw(&self, commands: &mut Commands, z: f32, resolution: &WindowResolution) {
        let mut path_builder = PathBuilder::new();

        // Start in the bottom left corner.
        path_builder.move_to(Vec2::new(
            -resolution.width() / 2.,
            -resolution.height() / 2.,
        ));

        for (i, y) in self.heights.iter().enumerate() {
            let x = (i as i32 - WIDTH as i32) as f32;
            let point = Vec2::new(x, *y);
            path_builder.line_to(point);
        }

        // End in the bottom right corner.
        path_builder.line_to(Vec2::new(
            resolution.width() / 2.,
            -resolution.height() / 2.,
        ));

        path_builder.close();
        let path = path_builder.build();

        // Apply z transformation so the shapes are layered properly and move
        // everything down.
        let transform = Transform::from_xyz(0.0, -resolution.height() / 2., z);

        commands.spawn((
            ShapeBundle {
                path,
                spatial: SpatialBundle::from_transform(transform),
                ..default()
            },
            Fill::color(self.color),
        ));
    }
}

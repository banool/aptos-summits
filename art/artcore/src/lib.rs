// https://github.com/bevyengine/bevy/issues/11493
// https://github.com/bevyengine/bevy/issues/11494

#[cfg(feature = "api")]
mod api;

#[cfg(feature = "api")]
pub use api::*;
use bevy::{
    core_pipeline::clear_color::ClearColorConfig, ecs::system::RunSystemOnce, prelude::*,
    window::WindowResolution,
};
use bevy_prototype_lyon::prelude::*;
use clap::Parser;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};
use std::ops::Range;

// TODO: Make the clap stuff conditional behind a feature.
#[derive(Clone, Debug, Parser)]
pub struct AppConfig {
    // TODO: Make the generation of the mountains unaffected by the width.
    #[clap(long, default_value_t = 2000.)]
    pub width: f32,

    // TODO: If we can make the Rust SDK less massive, use AccountAddress instead.
    #[clap(long)]
    pub initial_token_address: String,
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
    pub fn build(self, web_config: Option<WebConfig>) -> App {
        let mut app = App::new();

        let resolution = WindowResolution::new(self.width, self.width);
        let window = match web_config {
            Some(web_config) => Window {
                resolution,
                // From https://bevy-cheatbook.github.io/platforms/wasm/webpage.html#custom-canvas.
                canvas: Some(web_config.html_canvas_id),
                fit_canvas_to_parent: true,
                ..default()
            },
            None => Window {
                resolution,
                ..default()
            },
        };
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(window),
            ..default()
        }));

        app.insert_resource(AppSeed {
            token_address: self.initial_token_address,
        })
        .add_plugins(ShapePlugin)
        .add_systems(Startup, initial_spawn)
        .add_systems(Update, update_mountains);

        app
    }

    #[cfg(feature = "api")]
    pub fn build_for_api(self, web_config: Option<WebConfig>, api_channels: ApiChannels) -> App {
        let mut app = self.build(web_config);
        app.insert_resource(api_channels.image_channel)
            .insert_resource(api_channels.token_address_receiver)
            .add_systems(Update, token_address_listener);
        app
    }
}

fn rand_color(rng: &mut ChaCha8Rng, r: Range<u8>, g: Range<u8>, b: Range<u8>) -> Color {
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

fn get_rng(token_address: &str) -> ChaCha8Rng {
    // Convert the token address into a u64 for the seed.
    let mut hasher = Sha256::new();
    hasher.update(&token_address);
    let result = hasher.finalize();
    let first_eight_bytes = &result[0..8];
    let seed = u64::from_be_bytes(first_eight_bytes.try_into().unwrap());

    println!("Token address: {} // Seed {}", token_address, seed);

    // Build deterministic rng with seed.
    ChaCha8Rng::seed_from_u64(seed)
}

// This is not Clone on purpose, we only want to use one randomness.
#[derive(Resource)]
struct Randomness {
    rng: ChaCha8Rng,
}

impl Randomness {
    pub fn from_token_address(token_address: &str) -> Self {
        Randomness {
            rng: get_rng(token_address),
        }
    }
}

fn initial_spawn(mut commands: Commands, app_seed: Res<AppSeed>) {
    commands.insert_resource(Randomness::from_token_address(&app_seed.token_address));
    commands.add(move |world: &mut World| {
        world.run_system_once(spawn_mountains);
    });
}

// TODO: Move this to to an update system and scroll each mountain layer.
fn spawn_mountains(
    mut commands: Commands,
    window: Query<&Window>,
    mut randomness: ResMut<Randomness>,
) {
    let mut rng = &mut randomness.rng;

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

    // Generate mountains back to front.
    let mut mountains = Vec::new();
    let num_mountains: u64 = rng.gen_range(4..7);
    let mountain_base_color = rand_color(&mut rng, 1..255, 1..255, 1..255);
    let base_max_height = height * 0.7;
    // If this is close to 0, the heights of the mountains will be more similar.
    let height_diff_multiplier = 0.7;
    for i in 0..num_mountains {
        let color = interpolate(
            mountain_base_color,
            sky_color,
            (i + 1) as f32 / num_mountains as f32,
        );
        let min_height = -height * 2.0 / (num_mountains * (num_mountains - i)) as f64;

        // Scale max_height based on z-order.
        let max_height =
            base_max_height * (1.0 - (i as f64 / num_mountains as f64 * height_diff_multiplier));
        println!(
            "Mountain {} min height {} max height: {}",
            i, min_height, max_height
        );

        let mountain = Mountain::new(
            window.width() as u32,
            min_height,
            max_height,
            color,
            (i + 1) as f32,
            &mut randomness,
        );
        mountains.push(mountain);
    }

    // Spawn and draw mountains.
    for mountain in mountains {
        mountain.spawn(&mut commands, &window.resolution);
    }
}

fn update_mountains(
    time: Res<Time>,
    window: Query<&Window>,
    mut randomness: ResMut<Randomness>,
    mut query: Query<(&mut Mountain, &mut Path)>,
) {
    let window = window.single();
    let resolution = &window.resolution;

    for (mut mountain, mut path) in query.iter_mut() {
        // Scroll the heights of the mountain.
        mountain.scroll(time.delta_seconds(), &mut randomness);

        // Update the path.
        *path = mountain.build_path(&resolution);
    }
}

struct MountainHeightGenerator {
    height: f32,
    slope: f32,
    step_max: f32,
    step_change: f32,
    min_height: f32,
    max_height: f32,
}

impl MountainHeightGenerator {
    fn new(min_height: f32, max_height: f32, randomness: &mut ResMut<Randomness>) -> Self {
        let rng = &mut randomness.rng;
        let step_max = rng.gen_range(0.9..1.1);
        let step_change = rng.gen_range(0.15..0.35);
        let height = rng.gen_range(0.0..max_height);
        let slope = rng.gen_range(0.0..step_max) * 2.0 - step_max;

        MountainHeightGenerator {
            height,
            slope,
            step_max,
            step_change,
            min_height,
            max_height,
        }
    }
}

impl MountainHeightGenerator {
    fn next(&mut self, randomness: &mut ResMut<Randomness>) -> Option<f32> {
        self.height += self.slope;
        self.slope += randomness.rng.gen_range(0.0..self.step_change) * 2.0 - self.step_change;

        if self.slope > self.step_max {
            self.slope = self.step_max;
        } else if self.slope < -self.step_max {
            self.slope = -self.step_max;
        }

        if self.height > self.max_height {
            self.height = self.max_height;
            self.slope *= -1.0;
        } else if self.height < self.min_height {
            self.height = self.min_height;
            self.slope *= -1.0;
        }

        Some(self.height)
    }
}

#[derive(Component)]
struct Mountain {
    heights: Vec<f32>,
    color: Color,
    z: f32,
    height_generator: MountainHeightGenerator,
    // To ensure we can scroll smoothly we need to keep track of what fraction of the
    // pixel we have scrolled through.
    pub sub_pixel_offset: f32,
}

impl Mountain {
    pub fn new(
        width: u32,
        min_height: f64,
        max_height: f64,
        color: Color,
        z: f32,
        randomness: &mut ResMut<Randomness>,
    ) -> Self {
        // Initialize the height generator
        let mut height_generator =
            MountainHeightGenerator::new(min_height as f32, max_height as f32, randomness);

        // Generate initial heights
        let mut heights: Vec<f32> = Vec::new();
        for _ in 0..width * 2 {
            heights.push(height_generator.next(randomness).unwrap());
        }

        Mountain {
            heights,
            color,
            z,
            height_generator,
            sub_pixel_offset: 0.0,
        }
    }

    fn build_path(self: &Mountain, resolution: &WindowResolution) -> Path {
        let mut path_builder = PathBuilder::new();

        // Start in the bottom left corner with the sub_pixel_offset.
        let start_x = -resolution.width() / 2. - self.sub_pixel_offset;

        path_builder.move_to(Vec2::new(
            start_x,
            -resolution.height() / 2.,
        ));

        for (i, y) in self.heights.iter().enumerate() {
            let x = start_x + i as f32;
            let point = Vec2::new(x, *y);
            path_builder.line_to(point);
        }

        // End in the bottom right corner.
        let end_x = start_x + self.heights.len() as f32;
        path_builder.line_to(Vec2::new(end_x, -resolution.height() / 2.0));

        path_builder.close();
        path_builder.build()
    }

    pub fn spawn(self, commands: &mut Commands, resolution: &WindowResolution) {
        let path = self.build_path(&resolution);

        // Apply z transformation so the shapes are layered properly and move
        // everything down a bit.
        let transform = Transform::from_xyz(0.0, -resolution.height() / 3.0, self.z);

        let color = self.color;
        commands.spawn(MountainBundle {
            mountain: self,
            shape_bundle: ShapeBundle {
                path,
                spatial: SpatialBundle::from_transform(transform),
                ..default()
            },
            fill: Fill::color(color),
        });
    }

    pub fn scroll(&mut self, delta_seconds: f32, randomness: &mut ResMut<Randomness>) {
        let movement = self.speed() * delta_seconds;
        self.sub_pixel_offset += movement;

        let whole_pixels = self.sub_pixel_offset.floor() as usize;
        self.sub_pixel_offset -= whole_pixels as f32;

        // Remove points from the left.
        self.heights.drain(0..whole_pixels);

        // Add points to the right.
        for _ in 0..whole_pixels {
            self.heights.push(self.height_generator.next(randomness).unwrap());
        }
    }

    pub fn speed(&self) -> f32 {
        let exponent = 2.0;
        let base_speed = 0.8;
        base_speed * self.z.powf(exponent)
    }
}

#[derive(Bundle)]
struct MountainBundle {
    mountain: Mountain,
    shape_bundle: ShapeBundle,
    fill: Fill,
}

/*
fn exiter(channel: Res<ImageChannel>, mut exit: EventWriter<AppExit>) {
    if channel.receiver.is_full() {
        eprintln!("Channel has image data, exiting Bevy app");
        // TODO: This doesn't work properly.
        // https://github.com/bevyengine/bevy/discussions/11496
        exit.send(AppExit);
        // Might need to use process exit from the API as the nuclear option.
    }
}
*/

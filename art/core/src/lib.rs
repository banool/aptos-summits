// https://github.com/bevyengine/bevy/issues/11493
// https://github.com/bevyengine/bevy/issues/11494

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    ecs::system::RunSystemOnce,
    prelude::*,
    render::view::screenshot::ScreenshotManager,
    window::{PrimaryWindow, WindowResolution},
    winit::WinitSettings,
};
use bevy_prototype_lyon::prelude::*;
use clap::Parser;
#[cfg(feature = "api")]
use crossbeam_channel::{Receiver, Sender};
#[cfg(feature = "api")]
use image::ImageOutputFormat;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};
use std::{io::Cursor, ops::Range};

const WIDTH: f32 = 1200.0;

// TODO: Make the clap stuff conditional behind a feature.
#[derive(Clone, Debug, Parser)]
pub struct AppConfig {
    #[clap(long, default_value_t = 1600.)]
    pub width: f32,

    // TODO: If we can make the Rust SDK less massive, use AccountAddress instead.
    #[clap(long)]
    pub initial_token_address: String,
}

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub html_canvas_id: String,
}

#[cfg(feature = "api")]
pub struct ApiChannels {
    pub image_channel: ImageChannel,
    pub token_address_receiver: TokenAddressReceiver,
}

#[cfg(not(feature = "api"))]
pub struct ApiChannels;

#[cfg(feature = "api")]
#[derive(Debug, Resource)]
pub struct ImageChannel {
    // Sender so we can send the image data back to the caller.
    pub sender: Sender<Vec<u8>>,
    // Receiver so bevy itself can know when the data was sent and then exit.
    // pub receiver: Receiver<Vec<u8>>,
}

#[cfg(feature = "api")]
#[derive(Debug, Resource)]
pub struct TokenAddressReceiver {
    // Receiver so we can modify the mountains.
    pub receiver: Receiver<String>,
}

#[derive(Resource)]
pub struct AppSeed {
    pub token_address: String,
}

impl AppConfig {
    pub fn build(self, web_config: Option<WebConfig>, api_channels: Option<ApiChannels>) -> App {
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
        match api_channels {
            Some(api_channels) => {
                app.insert_resource(WinitSettings {
                    // https://github.com/bevyengine/bevy/issues/11494
                    // return_from_run: true,
                    ..default()
                });
                // TODO: For now we can't do this headlessly, see this issue:
                // https://github.com/bevyengine/bevy/issues/11493
                // This means the API must be run in a place with windowing support.
                app.add_plugins(DefaultPlugins.build() /*.disable::<WinitPlugin>()*/);
                #[cfg(feature = "api")]
                app.insert_resource(api_channels.image_channel);
                #[cfg(feature = "api")]
                app.insert_resource(api_channels.token_address_receiver);
                #[cfg(feature = "api")]
                app.add_systems(Update, token_address_listener);
            },
            None => {
                app.add_plugins(DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                }));
            },
        }

        app.insert_resource(AppSeed {
            token_address: self.initial_token_address,
        })
        .add_plugins(ShapePlugin)
        .add_systems(Startup, initial_spawn);

        app
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

fn initial_spawn(mut commands: Commands, window: Query<&Window>, app_seed: Res<AppSeed>) {
    spawn_mountains(&mut commands, window, app_seed.token_address.clone());
}

// TODO: Move this to to an update system and scroll each mountain layer.
fn spawn_mountains(mut commands: &mut Commands, window: Query<&Window>, token_address: String) {
    // Convert the token address into a u64 for the seed.
    let mut hasher = Sha256::new();
    hasher.update(&token_address);
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
    let mut z = 1.;
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
        let mountain = Mountain::new(&mut rng, WIDTH as u32, min_height, max_height, color, z);
        mountains.push(mountain);
        z += 1.;
    }

    // Spawn and draw mountains.
    for mountain in mountains {
        mountain.spawn(&mut commands, &window.resolution);
    }
}

// This despawns the shapes drawn too.
fn despawn_mountains(mut commands: Commands, mountains: Query<Entity, With<Mountain>>) {
    for mountain in mountains.iter() {
        commands.entity(mountain).despawn_recursive();
    }
}

fn despawn_camera(mut commands: Commands, cameras: Query<Entity, With<Camera2d>>) {
    for camera in cameras.iter() {
        commands.entity(camera).despawn_recursive();
    }
}

#[derive(Component)]
struct Mountain {
    heights: Vec<f32>,
    color: Color,
    z: f32,
}

#[derive(Bundle)]
struct MountainBundle {
    mountain: Mountain,
    shape_bundle: ShapeBundle,
    fill: Fill,
}

impl Mountain {
    pub fn new(
        rng: &mut SmallRng,
        width: u32,
        min_height: f64,
        max_height: f64,
        color: Color,
        z: f32,
    ) -> Self {
        let step_max = rng.gen_range(0.9..1.1);
        let step_change = rng.gen_range(0.15..0.35);
        let mut height = rng.gen_range(0.0..max_height);
        let mut slope = rng.gen_range(0.0..step_max) * 2.0 - step_max;
        let mut heights: Vec<f32> = Vec::new();

        for _ in 0..(width * 2) {
            height += slope;
            slope += rng.gen_range(0.0..step_change) * 2.0 - step_change;

            if slope > step_max {
                slope = step_max;
            } else if slope < -step_max {
                slope = -step_max;
            }

            if height > max_height {
                height = max_height;
                slope *= -1.0;
            } else if height < min_height {
                height = min_height;
                slope *= -1.0;
            }
            heights.push(height as f32);
        }
        Mountain { heights, color, z }
    }

    pub fn spawn(self, commands: &mut Commands, resolution: &WindowResolution) {
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
        let transform = Transform::from_xyz(0.0, -resolution.height() / 2., self.z);

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

#[cfg(feature = "api")]
fn token_address_listener(
    channel: Res<TokenAddressReceiver>,
    mut commands: Commands,
    window: Query<&Window>,
) {
    if let Ok(token_address) = channel.receiver.try_recv() {
        eprintln!("New token address: {}", token_address);
        commands.add(move |world: &mut World| {
            world.run_system_once(despawn_mountains);
            world.run_system_once(despawn_camera);
        });
        spawn_mountains(&mut commands, window, token_address);
        commands.add(move |world: &mut World| {
            world.run_system_once(capture_frame);
        });
    }
}

#[cfg(feature = "api")]
fn capture_frame(
    channel: Res<ImageChannel>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
) {
    let sender = channel.sender.clone();
    if sender.is_full() {
        // eprintln!("Sender is full, not capturing frame");
        return;
    }
    screenshot_manager
        .take_screenshot(main_window.single(), move |image| {
            let image = image
                .try_into_dynamic()
                .expect("Failed to convert image to dynamic");
            let mut buffer = Cursor::new(Vec::new());
            image
                .write_to(&mut buffer, ImageOutputFormat::Png)
                .expect("Failed to write image as png");
            let png_data: Vec<u8> = buffer.into_inner();

            let result = sender.send(png_data);
            match result {
                Ok(_) => eprintln!("Sent image data"),
                Err(e) => eprintln!("Failed to send image data {}", e),
            }
        })
        .unwrap();
}

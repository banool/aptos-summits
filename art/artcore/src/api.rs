use super::{spawn_mountains, Mountain, Randomness};
use bevy::{
    ecs::system::RunSystemOnce, prelude::*, render::view::screenshot::ScreenshotManager,
    window::PrimaryWindow,
};
use crossbeam_channel::{Receiver, Sender};
use image::{ImageOutputFormat, RgbaImage};
use once_cell::sync::Lazy;
use std::io::Cursor;

pub struct ApiChannels {
    pub image_channel: ImageChannel,
    pub token_address_receiver: TokenAddressReceiver,
}

#[derive(Debug, Resource)]
pub struct ImageChannel {
    // Sender so we can send the image data back to the caller.
    pub sender: Sender<Vec<u8>>,
}

#[derive(Debug, Resource)]
pub struct TokenAddressReceiver {
    // Receiver so we can modify the mountains.
    pub receiver: Receiver<String>,
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

pub fn token_address_listener(channel: Res<TokenAddressReceiver>, mut commands: Commands) {
    if let Ok(token_address) = channel.receiver.try_recv() {
        eprintln!("New token address: {}", token_address);
        commands.insert_resource(Randomness::from_token_address(&token_address));
        commands.add(move |world: &mut World| {
            world.run_system_once(despawn_mountains);
            world.run_system_once(despawn_camera);
            world.run_system_once(spawn_mountains);
            world.run_system_once(capture_frame);
        });
    }
}

// Because we do this at the screenshot layer, it means the texture and logo are
// only visible in screenshots. This is fine for now since we're not going to use
// the site at the moment, just drop the tokens.

const NFT_TEXTURE: &'static [u8] =
    include_bytes!("../../assets/aptos-ecosummit-2024_nft_texture_v2.png");
const NFT_LOCKUP: &'static [u8] =
    include_bytes!("../../assets/aptos-ecosummit-2024_nft_lockup.png");

static NFT_TEXTURE_RGBA8: Lazy<RgbaImage> = Lazy::new(|| {
    image::load_from_memory(NFT_TEXTURE)
        .expect("Failed to load NFT texture")
        .to_rgba8()
});

static NFT_LOCKUP_RGBA8: Lazy<RgbaImage> = Lazy::new(|| {
    image::load_from_memory(NFT_LOCKUP)
        .expect("Failed to load NFT lockup")
        .to_rgba8()
});

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

            let mut image = image.to_rgba8();

            // This requires that the output width be the same size as the overlays.
            blend_images_multiply(&mut image, vec![&NFT_TEXTURE_RGBA8]);
            blend_images_replace(&mut image, vec![&NFT_LOCKUP_RGBA8]);

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

fn blend_images_multiply(base_image: &mut RgbaImage, images: Vec<&RgbaImage>) {
    for (x, y, pixel) in base_image.enumerate_pixels_mut() {
        for image in &images {
            let image_pixel = image.get_pixel(x, y);
            // Multiply blend mode formula: (Base * Overlay) / 255
            // Apply this formula to each channel (R, G, B)
            for i in 0..3 {
                let base_val = pixel.0[i] as u16;
                let overlay_val = image_pixel.0[i] as u16;

                // Perform the multiply blend operation
                pixel.0[i] = ((base_val * overlay_val) / 255) as u8;
            }
        }
    }
}

// Replace the pixels with the overlay images based on transparency.
fn blend_images_replace(base_image: &mut RgbaImage, images: Vec<&RgbaImage>) {
    for (x, y, pixel) in base_image.enumerate_pixels_mut() {
        for image in &images {
            let image_pixel = image.get_pixel(x, y);
            // Overlay the pixels based on the alpha of image_pixel.
            for i in 0..3 {
                let base_val = pixel.0[i] as u16;
                let overlay_val = image_pixel.0[i] as u16;
                let alpha = image_pixel.0[3] as u16;

                // Perform the multiply blend operation
                pixel.0[i] = ((base_val * (255 - alpha) + overlay_val * alpha) / 255) as u8;
            }
        }
    }
}

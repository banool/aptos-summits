use artcore::{ApiChannels, AppConfig, ImageChannel, TokenAddressReceiver};

#[derive(Debug)]
pub struct BevyChannels {
    pub token_address_sender: crossbeam_channel::Sender<String>,
    pub img_data_receiver: crossbeam_channel::Receiver<Vec<u8>>,
}

pub fn run_bevy_app(
    width: u32,
    img_data_sender: crossbeam_channel::Sender<Vec<u8>>,
    token_address_receiver: crossbeam_channel::Receiver<String>,
) {
    // TODO: Accept this from configuration.
    let app_config = AppConfig {
        width: width as f32,
        initial_token_address: "0x5".to_string(),
        paused: true,
    };

    let mut bevy_app = app_config.build_for_api(None, ApiChannels {
        image_channel: ImageChannel {
            sender: img_data_sender,
        },
        token_address_receiver: TokenAddressReceiver {
            receiver: token_address_receiver,
        },
    });

    // Run the app.
    bevy_app.run();
}

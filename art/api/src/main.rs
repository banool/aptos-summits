use axum::{
    extract::{Path, State},
    http::header,
    response::{AppendHeaders, IntoResponse},
    routing::get,
    Router,
};
use core::{ApiChannels, AppConfig, ImageChannel, TokenAddressReceiver};
use serde::Deserialize;
use tower_http::trace::TraceLayer;

#[derive(Clone, Debug)]
struct MyState {
    token_address_sender: crossbeam_channel::Sender<String>,
    img_data_receiver: crossbeam_channel::Receiver<Vec<u8>>,
}

#[tokio::main]
async fn main() {
    let width: f32 = std::env::var("WIDTH")
        .unwrap_or_else(|_| "1200.0".to_string())
        .parse()
        .expect("WIDTH must be a float");

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3131".to_string())
        .parse()
        .expect("PORT must be a u16");

    let (img_data_sender, img_data_receiver) = crossbeam_channel::bounded(1);
    let (token_address_sender, token_address_receiver) = crossbeam_channel::bounded(1);

    let state = MyState {
        token_address_sender,
        img_data_receiver,
    };

    // Run the API in the background.
    tokio::spawn(async move {
        let app = Router::new()
            .route("/", get(|| async { "Hello!" }))
            .route("/media/:network/:address", get(handler))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
            .await
            .unwrap();

        eprintln!("Running server on port {}", port);

        axum::serve(listener, app).await.unwrap();
    });

    let app_config = AppConfig {
        width,
        initial_token_address: "0x5".to_string(),
    };

    let mut bevy_app = app_config.build(
        None,
        Some(ApiChannels {
            image_channel: ImageChannel {
                sender: img_data_sender,
                // receiver: img_data_receiver,
            },
            token_address_receiver: TokenAddressReceiver {
                receiver: token_address_receiver,
            },
        }),
    );

    // Run the app. This call should end because we have a system that ends the app
    // after the image is captured and we use return_from_run. This will run in
    // the background.
    bevy_app.run();
}

#[derive(Deserialize)]
struct PathParams {
    #[allow(dead_code)]
    network: String,
    address: String,
}

async fn handler(
    Path(params): Path<PathParams>,
    State(state): State<MyState>,
) -> impl IntoResponse {
    // TODO: Single sempahore. Maybe not necessary.

    // Send the token address to the app.
    state
        .token_address_sender
        .send(params.address.clone())
        .unwrap();

    // This needs to actually take the data.
    let image = state.img_data_receiver.recv().unwrap();

    let headers = AppendHeaders([(header::CONTENT_TYPE, "image/png")]);

    println!("Returning image via API");

    (headers, image)
}
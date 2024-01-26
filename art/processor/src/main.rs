mod bevyapp;
mod blob_store;
mod config;
mod health_server;
mod processor;
mod run;
mod storage;

use crate::config::{Args, Config};
use anyhow::{Context as AnyhowContext, Result};
use bevyapp::{run_bevy_app, BevyChannels};
use blob_store::LocalBlobStore;
use clap::Parser;
use run::run;
use std::sync::Arc;
use storage::PostgresStorage;
use tokio::runtime::Builder;
use tracing::{error, Level};
use tracing_subscriber::FmtSubscriber;

fn main() -> Result<()> {
    Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to build tokio runtime")?
        .block_on(main_inner())
}

async fn main_inner() -> Result<()> {
    let args = Args::parse();
    let config = Config::try_from(args)?;

    let subscriber = FmtSubscriber::builder()
        // All spans of this level or more severe will be written to stdout.
        .with_max_level(Level::INFO)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Setting default tracing subscriber failed")?;

    let storage = PostgresStorage::new(config.storage_config.clone())
        .await
        .context("Failed to initialize Postgres storage")?;

    let blob_store = Arc::new(
        LocalBlobStore::new(config.blob_store_config.clone())
            .await
            .context("Failed to initialize Postgres storage")?,
    );

    // Create channels for communication with the Bevy app.
    let (img_data_sender, img_data_receiver) = crossbeam_channel::bounded::<Vec<u8>>(1);
    let (token_address_sender, token_address_receiver) = crossbeam_channel::bounded::<String>(1);

    let bevy_channels = BevyChannels {
        token_address_sender,
        img_data_receiver,
    };

    let mut tasks = run(config.processor_config, blob_store, storage, bevy_channels).await?;

    // Start the health server.
    let health_server = tokio::spawn(async {
        let result = health_server::run(config.health_server_config).await;
        error!("Health server ended unexpectedly: {:?}", result);
    });

    tasks.push(health_server);

    run_bevy_app(config.bevy_width, img_data_sender, token_address_receiver);

    // Wait for all the tasks. We don't actually get here, the bevy app blocks.
    let result = futures::future::select_all(tasks).await;

    Err(anyhow::anyhow!(
        "One of the futures finished unexpectedly: {:#?}",
        result
    ))
}

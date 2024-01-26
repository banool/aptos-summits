//! Using these functions the dev can build and run all the components required to run
//! the processor. The dev could glue all these pieces together themselves, this file
//! doesn't use anything private, so this is all just for dev convenience / dedupe.

use super::storage::{CommonStorageConfig, PostgresStorage};
use crate::{
    bevyapp::BevyChannels,
    blob_store::BlobStoreTrait,
    processor::{SummitsProcessor, SummitsProcessorConfig},
};
use anyhow::{Context, Result};
use aptos_processor_sdk::{
    dispatcher::{Dispatcher, DispatcherConfig},
    processor::ProcessorTrait,
    progress_storage::ProgressStorageTrait,
    stream_subscriber::{GrpcStreamSubscriber, GrpcStreamSubscriberConfig, StreamSubscriberTrait},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// This contains all the configs necessary to build the components required to run the
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfig {
    pub stream_subscriber_config: GrpcStreamSubscriberConfig,
    pub dispatcher_config: DispatcherConfig,
    pub common_storage_config: CommonStorageConfig,
    pub processor_config: SummitsProcessorConfig,
}

/// Build all the relevant pieces required to run the processor, and the processor
/// itself, and spawn tokio tasks for them. This returns handles to those tasks.
pub async fn run(
    config: RunConfig,
    blob_store: Arc<dyn BlobStoreTrait>,
    storage: PostgresStorage,
    bevy_channels: BevyChannels,
) -> Result<Vec<JoinHandle<()>>> {
    // Build the question processor, which is what processes transactions and updates the
    // question storage and the DB.
    let processor = Arc::new(
        SummitsProcessor::new(config.processor_config.clone(), blob_store, bevy_channels)
            .context("Failed to build processor")?,
    );

    // From the DB, read the last version we processed.
    let starting_version_from_db = storage
        .read_last_processed_version(processor.name())
        .await?;

    // Determine the actual version we'll start from based on the data in the DB and
    // the values in the config.
    let starting_version = config
        .common_storage_config
        .determine_starting_version(starting_version_from_db);

    // Build the stream subscriber, which subscribes to txn stream service and pushes
    // the txns to an internal channel.
    let stream_subscriber = GrpcStreamSubscriber::new(
        config.stream_subscriber_config.clone(),
        processor.name().to_string(),
        starting_version,
    );

    // Start the stream subscriber.
    let channel_handle = stream_subscriber.start()?;

    // Forcibly set the number of concurrent workers to 1. This processor depends on
    // txns being processed in order because of how we only write files when we
    // process the create question txns.
    let mut dispatcher_config = config.dispatcher_config.clone();
    dispatcher_config.number_concurrent_processing_tasks = 1;

    // Build the dispatcher, which is what reads from the channel and dispatches txns
    // to the processor.
    let storage_clone = storage.clone();
    let dispatcher_task = tokio::spawn(async move {
        let mut dispatcher = Dispatcher {
            config: dispatcher_config,
            progress_storage: storage_clone,
            processor: processor.clone(),
            receiver: channel_handle.receiver,
            starting_version,
            indexer_grpc_data_service_address: config
                .stream_subscriber_config
                .grpc_data_service_address
                .clone(),
            auth_token: config.stream_subscriber_config.auth_token.clone(),
        };
        let result = dispatcher.dispatch().await;
        eprintln!("Dispatcher finished unexpectedly: {:?}", result);
    });

    let task_handles = vec![dispatcher_task, channel_handle.join_handle];

    Ok(task_handles)
}

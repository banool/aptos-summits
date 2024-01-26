use crate::{bevyapp::BevyChannels, blob_store::BlobStoreTrait};
use anyhow::{Context as AnyhowContext, Result};
use aptos_processor_sdk::{
    aptos_protos::transaction::v1::{
        transaction::TxnData, transaction_payload::Payload, write_set_change::Change,
        EntryFunctionId, MoveModuleId, Transaction,
    },
    processor::{ProcessingResult, ProcessorTrait},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

const MODULE_NAME: &str = "summits_token";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SummitsProcessorConfig {
    // TODO: This should be an Address instead
    pub contract_address: String,
}

#[derive(Debug)]
pub struct SummitsProcessor {
    config: SummitsProcessorConfig,
    blob_store: Arc<dyn BlobStoreTrait>,
    bevy_channels: BevyChannels,
}

impl SummitsProcessor {
    pub fn new(
        config: SummitsProcessorConfig,
        blob_store: Arc<dyn BlobStoreTrait>,
        bevy_channels: BevyChannels,
    ) -> Result<Self> {
        Ok(Self {
            config,
            blob_store,
            bevy_channels,
        })
    }
}

/// A processor that just prints the txn version.
#[async_trait::async_trait]
impl ProcessorTrait for SummitsProcessor {
    fn name(&self) -> &'static str {
        "SummitsProcessor"
    }

    async fn process_transactions(
        &self,
        transactions: Vec<Transaction>,
        start_version: u64,
        end_version: u64,
        _db_chain_id: Option<u8>,
    ) -> Result<ProcessingResult> {
        let mut token_addresses_to_write = Vec::new();
        for transaction in transactions {
            // Skip failed transactions.
            if let Some(info) = &transaction.info {
                if !info.success {
                    continue;
                }
            }
            if let Some(txn_has) = self.get_tokens_to_write(&transaction)? {
                token_addresses_to_write.push(txn_has);
            }
        }

        // Generate images and write them to storage.
        for txn_hash in token_addresses_to_write {
            info!("Writing image for txn {}", txn_hash);

            // Send the token address to the Bevy app.
            self.bevy_channels
                .token_address_sender
                .send(txn_hash.clone())?;

            // Pull the image data the app eventually writes.
            let image = self.bevy_channels.img_data_receiver.recv().unwrap();

            self.blob_store.write_image(image, &txn_hash).await?;

            info!("Wrote image for txn {}", txn_hash);
        }

        Ok(ProcessingResult {
            start_version,
            end_version,
            // These aren't correct of course. Just placeholders for now.
            processing_duration_in_secs: 0.0,
            db_insertion_duration_in_secs: 0.0,
        })
    }
}

impl SummitsProcessor {
    /// Returns txn hashes to generate and write to file storage.
    fn get_tokens_to_write(&self, transaction: &Transaction) -> Result<Option<String>> {
        // TODO: This check doesn't handle account addresses with leading zeroes.
        // Skip this transaction if this wasn't a create transaction.
        let entry_function_id = EntryFunctionId {
            module: Some(MoveModuleId {
                address: self.config.contract_address.clone(),
                name: MODULE_NAME.to_string(),
            }),
            name: "mint".to_string(),
        };

        // Filter out non mints.
        if !entry_function_id_matches(transaction, &entry_function_id) {
            return Ok(None);
        }

        let info = match transaction.info {
            Some(ref info) => info,
            None => return Ok(None),
        };

        for change in &info.changes {
            if let Some(change) = &change.change {
                match change {
                    Change::WriteResource(resource) => {
                        if resource.type_str == "0x4::token::Token" {
                            return Ok(Some(standardize_address(&resource.address)));
                        }
                    },
                    _ => {},
                }
            }
        }

        Ok(None)
    }
}

fn entry_function_id_matches(
    transaction: &Transaction,
    entry_function_id: &EntryFunctionId,
) -> bool {
    let txn_data = transaction
        .txn_data
        .as_ref()
        .context("No txn_data")
        .unwrap();
    let user_transaction = match txn_data {
        TxnData::User(user_transaction) => user_transaction,
        _ => return false,
    };
    let request = user_transaction
        .request
        .as_ref()
        .context("No request")
        .unwrap();
    let payload = request.payload.as_ref().unwrap();
    let entry_function_payload = match payload.payload.as_ref().context("No payload").unwrap() {
        Payload::EntryFunctionPayload(payload) => payload,
        _ => return false,
    };

    let function_id = entry_function_payload
        .function
        .as_ref()
        .context("No function")
        .unwrap();

    function_id == entry_function_id
}

// Not in the SDK right now.
pub fn standardize_address(handle: &str) -> String {
    if let Some(handle) = handle.strip_prefix("0x") {
        format!("0x{:0>64}", handle)
    } else {
        format!("0x{:0>64}", handle)
    }
}

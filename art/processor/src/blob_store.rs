use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};

/// A BlobStore is ... todo
#[async_trait::async_trait]
pub trait BlobStoreTrait: Debug + Send + Sync + 'static {
    /// Write the image to storage. The image should be in png format.
    async fn write_image(&self, png_data: Vec<u8>, token_address: &str) -> Result<()>;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalBlobStoreConfig {
    out_dir: PathBuf,
}

/// This is only intended for use in testing, it is not required in any of the main
/// deployment configurations, including for any approach related to serving images.
#[derive(Clone, Debug)]
pub struct LocalBlobStore {
    config: LocalBlobStoreConfig,
}

impl LocalBlobStore {
    pub async fn new(config: LocalBlobStoreConfig) -> Result<Self> {
        Ok(Self { config })
    }
}

#[async_trait]
impl BlobStoreTrait for LocalBlobStore {
    async fn write_image(&self, png_data: Vec<u8>, token_address: &str) -> Result<()> {
        let extension = "png";
        let filename = format!("{}.{}", token_address, extension);

        std::fs::write(self.config.out_dir.join(filename), png_data).context(format!(
            "Failed to write image for {} to disk",
            token_address
        ))?;

        Ok(())
    }
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::objects::{
        upload::{UploadObjectRequest, UploadType},
        Object,
    },
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf, sync::Arc};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GcsBlobStoreConfig {
    bucket_name: String,
}

/// This assumes that we're running inside GCP. If we're not then this won't work
/// because we use ClientConfig::default().with_auth() to create the client.
#[derive(Clone)]
pub struct GcsBlobStore {
    config: GcsBlobStoreConfig,
    client: Client,
}

impl Debug for GcsBlobStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GcsBlobStore")
            .field("config", &self.config)
            .finish()
    }
}

impl GcsBlobStore {
    pub async fn new(config: GcsBlobStoreConfig) -> Result<Self> {
        let client_config = ClientConfig::default()
            .with_auth()
            .await
            .context("Failed to create GCP GCS client config")?;
        let client = Client::new(client_config);
        Ok(Self { config, client })
    }
}

#[async_trait]
impl BlobStoreTrait for GcsBlobStore {
    async fn write_image(&self, png_data: Vec<u8>, token_address: &str) -> Result<()> {
        let extension = "png";

        let filename = format!("images/{}.{}", token_address, extension);
        // We can't use uploadType::Simple because it doesn't allow us to set the cache
        // control parameters.
        let upload_type = UploadType::Multipart(Box::new(Object {
            name: filename.clone(),
            content_type: format!("image/{}", extension).into(),
            size: png_data.len() as i64,
            // cache_control: Some("no-cache, no-store, max-age=0".to_string()),
            ..Default::default()
        }));
        self.client
            .upload_object(
                &UploadObjectRequest {
                    bucket: self.config.bucket_name.clone(),
                    ..Default::default()
                },
                png_data.clone(),
                &upload_type,
            )
            .await
            .with_context(|| {
                format!("Failed to write image for address {} to GCS", token_address)
            })?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum BlobStoreConfig {
    Local(LocalBlobStoreConfig),
    Gcs(GcsBlobStoreConfig),
}

impl BlobStoreConfig {
    pub async fn build(self) -> Result<Arc<dyn BlobStoreTrait>> {
        match self {
            Self::Local(config) => Ok(Arc::new(LocalBlobStore::new(config).await?)),
            Self::Gcs(config) => Ok(Arc::new(GcsBlobStore::new(config).await?)),
        }
    }
}

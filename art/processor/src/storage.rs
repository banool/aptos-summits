use anyhow::{Context, Result};
use aptos_processor_sdk::progress_storage::ProgressStorageTrait;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use tracing::info;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresStorageConfig {
    pub connection_string: String,
}

#[derive(Clone, Debug)]
pub struct PostgresStorage {
    pub pool: Pool<Postgres>,
}

impl PostgresStorage {
    pub async fn new(config: PostgresStorageConfig) -> Result<Self> {
        // Build the DB connection.
        let pool = PgPoolOptions::new()
            .connect(&config.connection_string)
            .await
            .context("Failed to connect to DB")?;

        // Apply migrations if necessary.
        sqlx::query("CREATE TABLE IF NOT EXISTS chain_id ( chain_id INTEGER )")
            .execute(&pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS processor_status ( processor VARCHAR NOT NULL UNIQUE, last_success_version BIGINT NOT NULL, PRIMARY KEY (processor))").execute(&pool).await?;

        info!("Built postgresql storage");

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl ProgressStorageTrait for PostgresStorage {
    async fn read_chain_id(&self) -> Result<Option<u8>> {
        Ok(None)
    }

    async fn write_chain_id(&self, _chain_id: u8) -> Result<()> {
        Ok(())
    }

    async fn read_last_processed_version(&self, processor_name: &str) -> Result<Option<u64>> {
        let last_processed_version_result =
            sqlx::query("SELECT last_success_version FROM processor_status WHERE processor = $1")
                .bind(processor_name)
                .fetch_optional(&self.pool)
                .await
                .context("Failed to read last processed last_success_version")?;

        match last_processed_version_result {
            Some(row) => {
                let version: i64 = row.try_get("last_success_version")?;
                Ok(Some(version as u64))
            },
            None => Ok(None),
        }
    }

    async fn write_last_processed_version(
        &self,
        processor_name: &str,
        version: u64,
        _last_transaction_timestamp: Option<
            aptos_processor_sdk::aptos_protos::util::timestamp::Timestamp,
        >,
    ) -> Result<()> {
        sqlx::query("INSERT INTO processor_status (processor, last_success_version) VALUES ($1, $2) ON CONFLICT (processor) DO UPDATE SET last_success_version = $2")
            .bind(processor_name)
            .bind(version as i64)
            .execute(&self.pool)
            .await
            .context("Failed to write last processed version")?;
        Ok(())
    }
}

// Originally from https://github.com/aptos-labs/aptos-indexer-processors/pull/202 but
// this is a bit too opinionated for the processor SDK.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CommonStorageConfig {
    /// If there is no minimum version in the DB, this is the version from which we'll
    /// start streaming txns.
    pub initial_starting_version: Option<u64>,

    /// Even if there is a minimum version in the DB, we'll start streaming txns from
    /// this version.
    pub starting_version_override: Option<u64>,
}

impl CommonStorageConfig {
    pub fn determine_starting_version(&self, starting_version_from_db: Option<u64>) -> u64 {
        let (starting_version, source) = match self.starting_version_override {
            Some(version) => {
                info!("Starting from starting_version_override: {}", version);
                (version, "starting_version_override")
            },
            None => match starting_version_from_db {
                Some(version) => {
                    info!("Starting from version found in DB: {}", version);
                    (version, "starting_version_from_db")
                },
                None => match self.initial_starting_version {
                    Some(version) => {
                        info!("Starting from initial_starting_version: {}", version);
                        (version, "initial_starting_version")
                    },
                    None => {
                        info!("No starting_version_override, starting_version_from_db, or initial_starting_version. Starting from version 0");
                        (0, "default")
                    },
                },
            },
        };
        info!(
            start_version = starting_version,
            source = source,
            "Starting from version {} (source: {})",
            starting_version,
            source
        );
        starting_version
    }
}

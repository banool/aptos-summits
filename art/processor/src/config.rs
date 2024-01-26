use super::{health_server::HealthServerConfig, run::RunConfig, storage::PostgresStorageConfig};
use crate::blob_store::LocalBlobStoreConfig;
use anyhow::Context as AnyhowContext;
use clap::Parser;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub config_path: PathBuf,
}

impl TryFrom<Args> for Config {
    type Error = anyhow::Error;

    /// This function uses Figment to read the config. In short, it reads the config
    /// first from the file and then environment variables.
    ///
    /// You can override (or set, if it wasn't set in the first place), a value
    /// in the config field with by setting an environment variable. First, to do this
    /// you need to set the appropriate prefix, so you must prefix your env vars with
    /// AURACLE___. Now, to override a nested config field you need to "nest" the env
    /// var key with `___`. For example, if you wanted to set connection_string in
    /// PostgresStorageConfig you would have to use the following key:
    ///
    /// AURACLE___METADATA_STORAGE_CONFIG___CONNECTION_STRING
    ///
    /// So if you had a config file that completely leaves out this section:
    ///
    /// metadata_storage_config:
    ///   connection_string: "postgres://dport@127.0.0.1:5432/postgres"
    ///
    /// You could set that whole "path" with this env var:
    ///
    /// AURACLE___METADATA_STORAGE_CONFIG___CONNECTION_STRING=postgres://dport@localhost:5432/postgres
    fn try_from(args: Args) -> Result<Self, Self::Error> {
        Figment::new()
            .merge(Yaml::file(args.config_path))
            .merge(Env::prefixed("SUMMITS___").split("___"))
            .extract()
            .context("Failed to load config")
    }
}

/// Config for running just the processor.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub processor_config: RunConfig,

    pub storage_config: PostgresStorageConfig,

    pub blob_store_config: LocalBlobStoreConfig,

    #[serde(default)]
    pub health_server_config: HealthServerConfig,

    pub bevy_width: u32,
}

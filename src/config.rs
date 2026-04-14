/// Configuration types for the Hyperion Swap Processor.
///
/// The `config.yaml` file is parsed into `IndexerProcessorConfig` by the
/// Aptos Indexer SDK server framework. A minimal `config.yaml` looks like:
///
/// ```yaml
/// health_check_port: 8085
/// server_config:
///   processor_config:
///     type: "hyperion_swap_processor"
///   transaction_stream_config:
///     indexer_grpc_data_service_address: "https://grpc.mainnet.aptoslabs.com:443"
///     starting_version: 0
///     request_ending_version: 1000000   # optional
///     auth_token: "aptoslabs_YOUR_TOKEN"
///     request_name_header: "hyperion-swap-processor"
///   db_config:
///     postgres_connection_string: "postgresql://user:pass@localhost:5432/hyperion"
/// ```
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

// ─── Top-level config (matches SDK ServerArgs expectations) ───────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IndexerProcessorConfig {
    /// Which processor to run (must equal `PROCESSOR_NAME`)
    pub processor_config: ProcessorConfig,

    /// gRPC Transaction Stream configuration
    pub transaction_stream_config: TransactionStreamConfig,

    /// Database configuration
    pub db_config: DbConfig,
}

impl IndexerProcessorConfig {
    pub fn validate(&self) -> Result<()> {
        if self.db_config.postgres_connection_string.is_empty() {
            bail!("postgres_connection_string must not be empty");
        }
        if self.transaction_stream_config.auth_token.is_empty() {
            bail!("auth_token must not be empty");
        }
        if self
            .transaction_stream_config
            .indexer_grpc_data_service_address
            .is_empty()
        {
            bail!("indexer_grpc_data_service_address must not be empty");
        }
        Ok(())
    }
}

// ─── Processor config ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProcessorConfig {
    /// Processor type identifier. Must be `"hyperion_swap_processor"`.
    #[serde(rename = "type")]
    pub processor_type: String,
}

// ─── Transaction Stream config ────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionStreamConfig {
    /// e.g. `"https://grpc.mainnet.aptoslabs.com:443"`
    pub indexer_grpc_data_service_address: String,

    /// The ledger version to start indexing from.
    /// The processor will resume from `processor_status.last_success_version + 1`
    /// if that is higher than this value.
    #[serde(default)]
    pub starting_version: u64,

    /// Optional: stop after reaching this version.
    /// Useful for backfilling a specific block range.
    pub request_ending_version: Option<u64>,

    /// API key obtained from https://developers.aptoslabs.com/
    pub auth_token: String,

    /// Sent as the `x-aptos-request-name` gRPC header; used for observability.
    pub request_name_header: Option<String>,
}

// ─── Database config ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbConfig {
    /// Full PostgreSQL connection string.
    /// Example: `"postgresql://postgres:secret@localhost:5432/hyperion_indexer"`
    pub postgres_connection_string: String,
}
/// `HyperionSwapProcessor` – the top-level processor struct.
///
/// Wires together:
///   1. `TransactionStreamStep`  – pulls transactions from the Aptos gRPC stream
///   2. `HyperionSwapExtractor`  – filters & parses Hyperion SwapEvent events
///   3. `HyperionSwapStorer`     – bulk-upserts events into PostgreSQL
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_indexer_transaction_stream::{TransactionStream, TransactionStreamConfig},
    builder::ProcessorBuilder,
    common_steps::TransactionStreamStep,
    traits::IntoRunnableStep,
};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::info;

use crate::{
    config::IndexerProcessorConfig,
    db::postgres::{new_pool, DbPool},
};
use super::{extractor::HyperionSwapExtractor, storer::HyperionSwapStorer};

pub const PROCESSOR_NAME: &str = "hyperion_swap_processor";

/// Embed all migrations at compile time so the binary is self-contained.
const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("src/db/migrations");

// ─── Processor ────────────────────────────────────────────────────────────────

pub struct HyperionSwapProcessor {
    pub config: IndexerProcessorConfig,
}

impl HyperionSwapProcessor {
    pub fn new(config: IndexerProcessorConfig) -> Self {
        Self { config }
    }

    pub async fn run_processor(self) -> Result<()> {
        // ── 1. Connect to the database ───────────────────────────────────────
        let pool = new_pool(&self.config.db_config.postgres_connection_string).await?;

        // ── 2. Run Diesel migrations (sync, via blocking task) ───────────────
        run_migrations(&self.config.db_config.postgres_connection_string).await?;

        // ── 3. Determine effective starting version ──────────────────────────
        let starting_version =
            get_starting_version(&pool, self.config.transaction_stream_config.starting_version)
                .await?;

        let ending_version = self.config.transaction_stream_config.request_ending_version;

        info!(
            starting_version,
            ending_version = ?ending_version,
            "Starting HyperionSwapProcessor pipeline"
        );

        // ── 4. Build gRPC transaction stream ─────────────────────────────────
        let txn_stream_config = TransactionStreamConfig {
            indexer_grpc_data_service_address: self
                .config
                .transaction_stream_config
                .indexer_grpc_data_service_address
                .parse()
                .expect("Invalid gRPC address URL"),
            starting_version: Some(starting_version),
            request_ending_version: ending_version,
            auth_token: self.config.transaction_stream_config.auth_token.clone(),
            request_name_header: self
                .config
                .transaction_stream_config
                .request_name_header
                .clone()
                .unwrap_or_else(|| PROCESSOR_NAME.to_string()),
            ..Default::default()
        };

        let transaction_stream = TransactionStream::new(txn_stream_config).await?;
        let stream_step = TransactionStreamStep::new(transaction_stream).await?;

        // ── 5. Instantiate pipeline steps ────────────────────────────────────
        let extractor = HyperionSwapExtractor;
        let storer = HyperionSwapStorer::new(pool);

        // ── 6. Connect steps into a pipeline using ProcessorBuilder ──────────
        //
        // Data flow:
        //   [gRPC Stream]
        //       │  Vec<Transaction>
        //       ▼
        //   HyperionSwapExtractor
        //       │  Vec<HyperionSwapEvent>
        //       ▼
        //   HyperionSwapStorer
        //       │  Vec<HyperionSwapEvent>  (passed through for observability)
        //       ▼
        //   [output channel – drained below]
        //
        let (_, output_receiver) =
            ProcessorBuilder::new_with_inputless_first_step(stream_step.into_runnable_step())
                .connect_to(extractor.into_runnable_step(), 10)
                .connect_to(storer.into_runnable_step(), 10)
                .end_and_return_output_receiver(10);

        // ── 7. Drain the output channel until the stream ends ─────────────────
        loop {
            match output_receiver.recv_async().await {
                Ok(txn_context) => {
                    info!(
                        start_version = txn_context.metadata.start_version,
                        end_version   = txn_context.metadata.end_version,
                        num_swaps     = txn_context.data.len(),
                        "Batch committed"
                    );
                }
                Err(_) => {
                    info!("Output channel closed – pipeline finished");
                    break;
                }
            }
        }

        Ok(())
    }
}

// ─── Migration helper ─────────────────────────────────────────────────────────

/// Run all pending Diesel migrations using a *synchronous* PgConnection.
/// Diesel migrations do not expose an async interface, so we run them
/// inside `spawn_blocking`.
async fn run_migrations(connection_string: &str) -> Result<()> {
    let conn_str = connection_string.to_owned();

    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut conn = diesel::PgConnection::establish(&conn_str)
            .map_err(|e| anyhow::anyhow!("Migration DB connect failed: {}", e))?;

        info!("Running pending database migrations…");
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| anyhow::anyhow!("Migration failed: {}", e))?;
        info!("Database migrations complete");
        Ok(())
    })
    .await??;

    Ok(())
}

// ─── Version tracking helper ──────────────────────────────────────────────────

/// Returns the effective starting version:
/// - Resume from `last_success_version + 1` if prior state exists.
/// - Otherwise use `config_starting_version`.
async fn get_starting_version(pool: &DbPool, config_starting_version: u64) -> Result<u64> {
    use crate::db::postgres::schema::processor_status::dsl::*;

    let mut conn = pool
        .get()
        .await
        .map_err(|e| anyhow::anyhow!("DB pool error: {}", e))?;

    let maybe_last: Option<i64> = processor_status
        .filter(processor.eq(PROCESSOR_NAME))
        .select(last_success_version)
        .first(&mut conn)
        .await
        .optional()
        .map_err(|e| anyhow::anyhow!("Failed to query processor_status: {}", e))?;

    let version = match maybe_last {
        Some(v) if v >= 0 => {
            let resumed = (v as u64).saturating_add(1);
            info!(
                last_success_version = v,
                resuming_from = resumed,
                "Resuming from last successful version"
            );
            resumed
        }
        _ => {
            info!(
                config_starting_version,
                "No prior progress found; using config starting_version"
            );
            config_starting_version
        }
    };

    Ok(version)
}
/// Entry point for the Hyperion Swap Processor.
///
/// Usage:
///   ```
///   hyperion-swap-processor --config config.yaml
///   ```
///
/// The binary reads the YAML config, validates it, then runs the processor
/// pipeline until the stream ends (if `request_ending_version` is set) or
/// until interrupted.
use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};


mod config;
mod db;
mod processors;

use config::IndexerProcessorConfig;
use processors::hyperion_swap::processor::HyperionSwapProcessor;

// ─── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "hyperion-swap-processor",
    about = "Aptos Indexer SDK processor for Hyperion DEX swap events",
    version
)]
struct Cli {
    /// Path to the YAML configuration file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // ── Logging ───────────────────────────────────────────────────────────────
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,hyperion_swap_processor=debug")),
        )
        .json()
        .init();

    // ── Parse CLI ─────────────────────────────────────────────────────────────
    let cli = Cli::parse();

    info!(config_path = %cli.config, "Loading configuration");

    // ── Load config ───────────────────────────────────────────────────────────
    let config_str = std::fs::read_to_string(&cli.config)
        .map_err(|e| anyhow::anyhow!("Cannot read config file '{}': {}", cli.config, e))?;

    let config: IndexerProcessorConfig = serde_yaml::from_str(&config_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse config YAML: {}", e))?;

    config.validate()?;

    info!(
        grpc_address = %config.transaction_stream_config.indexer_grpc_data_service_address,
        starting_version = config.transaction_stream_config.starting_version,
        ending_version = ?config.transaction_stream_config.request_ending_version,
        "Configuration loaded"
    );

    // ── Run processor ─────────────────────────────────────────────────────────
    let processor = HyperionSwapProcessor::new(config);
    processor.run_processor().await?;

    info!("Processor finished successfully");
    Ok(())
}
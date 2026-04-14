/// `HyperionSwapExtractor` – Step 1 of the processor pipeline.
///
/// Receives a batch of raw Aptos transactions from `TransactionStreamStep`,
/// iterates over every event in each user transaction, and emits only the
/// `SwapEvent`s emitted by the Hyperion contract.
///
/// The SDK models each processor as a DAG of *Steps* connected by channels.
/// An extractor step implements `AsyncStep` and transforms
/// `Vec<Transaction>` → `Vec<HyperionSwapEvent>`.

use anyhow::Result;
use aptos_indexer_processor_sdk::{
    aptos_protos::transaction::v1::{
        transaction::TxnData, Event, Transaction,
    },
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use tracing::{debug, warn};

use super::model::HyperionSwapEvent;

// ─── Hyperion contract constants ─────────────────────────────────────────────

/// The Hyperion contract address on Aptos mainnet.
/// This is the module publisher / deployer address.
pub const HYPERION_CONTRACT_ADDRESS: &str =
    "0xd3894aca06d5f42b27c89e6f448114b3ed6a1ba07f992a58b2126c71dd83c127";

/// The Move module name within the Hyperion contract that emits swap events.
/// Adjust if Hyperion uses a different module name (e.g. `clmm_pool`, `pool_v2`).
pub const HYPERION_POOL_MODULE: &str = "pool";

/// The Move event struct name for swap events.
pub const HYPERION_SWAP_EVENT_STRUCT: &str = "SwapEvent";

/// Full Move event type tag:  `<address>::<module>::<struct>`
pub fn hyperion_swap_event_type() -> String {
    format!(
        "{}::{}::{}",
        HYPERION_CONTRACT_ADDRESS, HYPERION_POOL_MODULE, HYPERION_SWAP_EVENT_STRUCT
    )
}

// ─── Extractor step ───────────────────────────────────────────────────────────

pub struct HyperionSwapExtractor;

impl NamedStep for HyperionSwapExtractor {
    fn name(&self) -> String {
        "HyperionSwapExtractor".to_string()
    }
}

#[async_trait]
impl Processable for HyperionSwapExtractor {
    type Input = Vec<Transaction>;
    type Output = Vec<HyperionSwapEvent>;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        transactions: TransactionContext<Vec<Transaction>>,
    ) -> Result<Option<TransactionContext<Vec<HyperionSwapEvent>>>, ProcessorError> {
        let swap_event_type = hyperion_swap_event_type();
        let mut swap_events: Vec<HyperionSwapEvent> = Vec::new();

        for txn in &transactions.data {
            let txn_version = txn.version as i64;
            let block_height = txn.block_height as i64;

            // Convert protobuf timestamp → NaiveDateTime
            let txn_timestamp = txn
                .timestamp
                .as_ref()
                .map(|ts| {
                    NaiveDateTime::from_timestamp_opt(
                        ts.seconds,
                        ts.nanos as u32,
                    )
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc())
                })
                .unwrap_or_else(|| chrono::Utc::now().naive_utc());

            // Only user transactions can contain contract events
            let user_txn = match &txn.txn_data {
                Some(TxnData::User(u)) => u,
                _ => continue,
            };

            let events: &[Event] = user_txn
                .events
                .as_slice();

            for (event_idx, event) in events.iter().enumerate() {
                // Filter by event type string
                if !is_hyperion_swap_event(&event.type_str, &swap_event_type) {
                    continue;
                }

                // Parse the raw JSON data
                let data_value: serde_json::Value =
                    serde_json::from_str(&event.data).unwrap_or(serde_json::Value::Null);

                match HyperionSwapEvent::from_raw_event(
                    txn_version,
                    event_idx as i64,
                    block_height,
                    txn_timestamp,
                    &event.type_str,
                    &data_value,
                ) {
                    Ok(swap) => {
                        debug!(
                            txn_version,
                            event_index = event_idx,
                            pool = %swap.pool_address,
                            "Extracted Hyperion swap event"
                        );
                        swap_events.push(swap);
                    }
                    Err(e) => {
                        warn!(
                            txn_version,
                            event_index = event_idx,
                            error = %e,
                            "Failed to parse Hyperion swap event – skipping"
                        );
                    }
                }
            }
        }

        Ok(Some(TransactionContext {
            data: swap_events,
            metadata: transactions.metadata,
        }))
    }
}

#[async_trait]
impl AsyncStep for HyperionSwapExtractor {}

// ─── Helper ───────────────────────────────────────────────────────────────────

/// Returns `true` if the event type string matches the Hyperion swap event.
///
/// We accept both the exact canonical type and any generic instantiation,
/// e.g. `0x..::pool::SwapEvent` as well as typed variants.
fn is_hyperion_swap_event(event_type: &str, canonical: &str) -> bool {
    // Exact match
    if event_type == canonical {
        return true;
    }
    // Handle generics: `0x..::pool::SwapEvent<0x1::..., 0x1::...>`
    if event_type.starts_with(canonical) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_event_type_filter() {
        let canonical = hyperion_swap_event_type();
        assert!(is_hyperion_swap_event(&canonical, &canonical));
        assert!(is_hyperion_swap_event(
            &format!("{}<0x1::a::A,0x1::b::B>", canonical),
            &canonical
        ));
        assert!(!is_hyperion_swap_event("0xdeadbeef::pool::SwapEvent", &canonical));
    }
}
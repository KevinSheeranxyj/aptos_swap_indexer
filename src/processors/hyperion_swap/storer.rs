/// `HyperionSwapStorer` – Step 2 of the processor pipeline.
///
/// Receives the extracted `Vec<HyperionSwapEvent>` from the extractor step
/// and bulk-inserts them into the `hyperion_swap_events` PostgreSQL table.
/// Also updates `processor_status` so the processor can resume correctly
/// after a restart.
use anyhow::Result;
use aptos_indexer_processor_sdk::{
    traits::{AsyncRunType, AsyncStep, NamedStep, Processable},
    types::transaction_context::TransactionContext,
    utils::errors::ProcessorError,
};
use async_trait::async_trait;
use diesel::{upsert::excluded, ExpressionMethods};
use diesel_async::RunQueryDsl;
use tracing::{debug, info};

use crate::db::postgres::{schema, DbPool};
use super::model::HyperionSwapEvent;

// ─── Storer step ─────────────────────────────────────────────────────────────

pub struct HyperionSwapStorer {
    pool: DbPool,
}

impl HyperionSwapStorer {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl NamedStep for HyperionSwapStorer {
    fn name(&self) -> String {
        "HyperionSwapStorer".to_string()
    }
}

#[async_trait]
impl Processable for HyperionSwapStorer {
    type Input = Vec<HyperionSwapEvent>;
    type Output = Vec<HyperionSwapEvent>;
    type RunType = AsyncRunType;

    async fn process(
        &mut self,
        context: TransactionContext<Vec<HyperionSwapEvent>>,
    ) -> Result<Option<TransactionContext<Vec<HyperionSwapEvent>>>, ProcessorError> {
        let swaps = &context.data;

        if swaps.is_empty() {
            debug!("No Hyperion swap events in this batch – skipping DB write");
            // Still need to update processor_status even when there are no swaps
            self.update_processor_status(&context).await?;
            return Ok(Some(context));
        }

        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ProcessorError::DBStoreError {
                message: format!("Failed to get DB connection: {}", e),
                query: None,
            })?;

        // ── Bulk upsert swap events ──────────────────────────────────────────
        // ON CONFLICT DO UPDATE so re-processing is idempotent.
        let inserted = diesel::insert_into(schema::hyperion_swap_events::table)
            .values(swaps)
            .on_conflict((
                schema::hyperion_swap_events::transaction_version,
                schema::hyperion_swap_events::event_index,
            ))
            .do_update()
            .set((
                schema::hyperion_swap_events::transaction_block_height
                    .eq(excluded(schema::hyperion_swap_events::transaction_block_height)),
                schema::hyperion_swap_events::transaction_timestamp
                    .eq(excluded(schema::hyperion_swap_events::transaction_timestamp)),
                schema::hyperion_swap_events::sender
                    .eq(excluded(schema::hyperion_swap_events::sender)),
                schema::hyperion_swap_events::pool_address
                    .eq(excluded(schema::hyperion_swap_events::pool_address)),
                schema::hyperion_swap_events::token_x_type
                    .eq(excluded(schema::hyperion_swap_events::token_x_type)),
                schema::hyperion_swap_events::token_y_type
                    .eq(excluded(schema::hyperion_swap_events::token_y_type)),
                schema::hyperion_swap_events::amount_x_in
                    .eq(excluded(schema::hyperion_swap_events::amount_x_in)),
                schema::hyperion_swap_events::amount_y_in
                    .eq(excluded(schema::hyperion_swap_events::amount_y_in)),
                schema::hyperion_swap_events::amount_x_out
                    .eq(excluded(schema::hyperion_swap_events::amount_x_out)),
                schema::hyperion_swap_events::amount_y_out
                    .eq(excluded(schema::hyperion_swap_events::amount_y_out)),
                schema::hyperion_swap_events::fee_amount
                    .eq(excluded(schema::hyperion_swap_events::fee_amount)),
                schema::hyperion_swap_events::sqrt_price_after
                    .eq(excluded(schema::hyperion_swap_events::sqrt_price_after)),
                schema::hyperion_swap_events::current_tick_after
                    .eq(excluded(schema::hyperion_swap_events::current_tick_after)),
                schema::hyperion_swap_events::liquidity
                    .eq(excluded(schema::hyperion_swap_events::liquidity)),
                schema::hyperion_swap_events::event_data
                    .eq(excluded(schema::hyperion_swap_events::event_data)),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| ProcessorError::DBStoreError {
                message: format!("Failed to upsert hyperion_swap_events: {}", e),
                query: Some("INSERT INTO hyperion_swap_events ... ON CONFLICT DO UPDATE".into()),
            })?;

        info!(
            start_version = context.metadata.start_version,
            end_version = context.metadata.end_version,
            inserted,
            "Stored Hyperion swap events"
        );

        // ── Update processor_status ──────────────────────────────────────────
        self.update_processor_status(&context).await?;

        Ok(Some(context))
    }
}

#[async_trait]
impl AsyncStep for HyperionSwapStorer {}

// ─── Internal helpers ─────────────────────────────────────────────────────────

impl HyperionSwapStorer {
    async fn update_processor_status(
        &self,
        context: &TransactionContext<Vec<HyperionSwapEvent>>,
    ) -> Result<(), ProcessorError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| ProcessorError::DBStoreError {
                message: format!("Failed to get DB connection for status update: {}", e),
                query: None,
            })?;

        let last_txn_timestamp = context
            .metadata
            .end_transaction_timestamp
            .as_ref()
            .map(|ts| {
                chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32)
                    .unwrap_or_else(|| chrono::Utc::now().naive_utc())
            });

        diesel::insert_into(schema::processor_status::table)
            .values((
                schema::processor_status::processor
                    .eq(super::processor::PROCESSOR_NAME),
                schema::processor_status::last_success_version
                    .eq(context.metadata.end_version as i64),
                schema::processor_status::last_transaction_timestamp
                    .eq(last_txn_timestamp),
                schema::processor_status::last_updated
                    .eq(chrono::Utc::now().naive_utc()),
            ))
            .on_conflict(schema::processor_status::processor)
            .do_update()
            .set((
                schema::processor_status::last_success_version
                    .eq(context.metadata.end_version as i64),
                schema::processor_status::last_transaction_timestamp
                    .eq(last_txn_timestamp),
                schema::processor_status::last_updated
                    .eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| ProcessorError::DBStoreError {
                message: format!("Failed to update processor_status: {}", e),
                query: Some("INSERT INTO processor_status ... ON CONFLICT DO UPDATE".into()),
            })?;

        Ok(())
    }
}
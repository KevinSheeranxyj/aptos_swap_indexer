// @generated automatically by Diesel CLI.
// Run `diesel migration run` to auto-generate this file.
// This file is committed for reference; regenerate after schema changes.

diesel::table! {
    ledger_infos (chain_id) {
        chain_id -> Int8,
    }
}

diesel::table! {
    processor_status (processor) {
        #[max_length = 100]
        processor -> Varchar,
        last_success_version -> Int8,
        last_transaction_timestamp -> Nullable<Timestamp>,
        last_updated -> Timestamp,
    }
}

diesel::table! {
    hyperion_swap_events (transaction_version, event_index) {
        transaction_version -> Int8,
        event_index -> Int8,
        transaction_block_height -> Int8,
        transaction_timestamp -> Timestamp,
        #[max_length = 66]
        sender -> Varchar,
        #[max_length = 66]
        pool_address -> Varchar,
        token_x_type -> Text,
        token_y_type -> Text,
        amount_x_in -> Numeric,
        amount_y_in -> Numeric,
        amount_x_out -> Numeric,
        amount_y_out -> Numeric,
        fee_amount -> Numeric,
        sqrt_price_after -> Nullable<Numeric>,
        current_tick_after -> Nullable<Int8>,
        liquidity -> Nullable<Numeric>,
        event_data -> Jsonb,
        inserted_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    ledger_infos,
    processor_status,
    hyperion_swap_events,
);
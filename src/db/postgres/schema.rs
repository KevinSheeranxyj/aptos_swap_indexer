// @generated automatically by Diesel CLI.

diesel::table! {
    hyperion_swap_events (id) {
        id -> Int8,
        transaction_version -> Nullable<Int8>,
        event_index -> Nullable<Int8>,
        transaction_block_height -> Nullable<Int8>,
        transaction_timestamp -> Nullable<Timestamp>,
        #[max_length = 66]
        sender -> Nullable<Varchar>,
        #[max_length = 66]
        pool_address -> Nullable<Varchar>,
        token_x_type -> Nullable<Text>,
        token_y_type -> Nullable<Text>,
        amount_x_in -> Nullable<Numeric>,
        amount_y_in -> Nullable<Numeric>,
        amount_x_out -> Nullable<Numeric>,
        amount_y_out -> Nullable<Numeric>,
        fee_amount -> Nullable<Numeric>,
        sqrt_price_after -> Nullable<Numeric>,
        current_tick_after -> Nullable<Numeric>,
        liquidity -> Nullable<Numeric>,
        event_data -> Nullable<Jsonb>,
        inserted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    processor_status (processor) {
        #[max_length = 255]
        processor -> Varchar,
        last_success_version -> Int8,
        last_updated -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(hyperion_swap_events, processor_status,);

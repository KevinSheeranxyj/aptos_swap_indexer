// @generated automatically by Diesel CLI.

diesel::table! {
    events (transaction_version, event_index) {
        sequence_number -> Int8,
        creation_number -> Int8,
        #[max_length = 66]
        account_address -> Varchar,
        transaction_version -> Int8,
        transaction_block_height -> Int8,
        #[sql_name = "type"]
        type_ -> Text,
        data -> Jsonb,
        inserted_at -> Timestamp,
        event_index -> Int8,
        #[max_length = 300]
        indexed_type -> Varchar,
    }
}

diesel::table! {
    pool_swaps (id) {
        id -> Int4,
        pool_address -> Nullable<Text>,
        timestamp -> Nullable<Timestamp>,
        amount_usd -> Nullable<Numeric>,
        fee_usd -> Nullable<Numeric>,
        sender -> Nullable<Text>,
        recipient -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    events,
    pool_swaps,
);

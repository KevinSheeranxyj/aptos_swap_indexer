-- Your SQL goes here

CREATE TABLE "processor_status" (
    processor VARCHAR(255) PRIMARY KEY,
    last_success_version BIGINT NOT NULL DEFAULT 0,
    last_updated TIMESTAMP NOT NULL DEFAULT NOW(),
    last_transaction_timestamp TIMESTAMP NULL,
);
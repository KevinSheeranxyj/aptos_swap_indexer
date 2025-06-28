-- Your SQL goes here
CREATE TABLE pool_swaps (
  id                SERIAL PRIMARY KEY,
  pool_address      TEXT,
  timestamp         TIMESTAMP,
  amount_usd        NUMERIC,
  fee_usd           NUMERIC,
  sender            TEXT,
  recipient         TEXT
);
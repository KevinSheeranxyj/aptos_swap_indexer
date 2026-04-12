# Aptos_swap_indexer
 Create a custom Aptos indexer processor using Rust

## Features
- Listens for swap transactions.
- Filters by block version (starting from a specific version).
- Stores relevant data into PostgreSQL.

## Architecture

Aptos gRPC Stream
      │
      │  Vec<Transaction>  (batches of ~1000 txns)
      ▼
┌─────────────────────────┐
│  TransactionStreamStep  │  (provided by SDK)
└───────────┬─────────────┘
            │
            │  Vec<Transaction>
            ▼
┌─────────────────────────┐
│  HyperionSwapExtractor  │  ← filters events by type string,
│                         │    parses JSON event data into
│                         │    HyperionSwapEvent structs
└───────────┬─────────────┘
            │
            │  Vec<HyperionSwapEvent>
            ▼
┌─────────────────────────┐
│  HyperionSwapStorer     │  ← bulk INSERT … ON CONFLICT DO UPDATE
│                         │    into PostgreSQL, updates processor_status
└─────────────────────────┘
            │
            │  Vec<HyperionSwapEvent>  (passed through for logging)
            ▼
       [output channel]
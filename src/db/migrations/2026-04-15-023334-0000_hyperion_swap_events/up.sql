-- Your SQL goes here
CREATE TABLE "public"."hyperion_swap_events" ("id" BIGSERIAL, "transaction_version" BIGINT,"event_index" BIGINT,"transaction_block_height" BIGINT,"transaction_timestamp" timestamp,"sender" varchar(66),"pool_address" varchar(66),"token_x_type" text,"token_y_type" text,"amount_x_in" numeric(40,0),"amount_y_in" numeric(40,0),"amount_x_out" numeric(40,0),"amount_y_out" numeric(40,0),"fee_amount" numeric(40,0),"sqrt_price_after" numeric(80,0), "current_tick_after" numeric(40,0), "liquidity" numeric(40,0),"event_data" jsonb,"inserted_at" timestamp, PRIMARY KEY ("id"));




use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;

use crate::db::postgres::schema::hyperion_swap_events;

#[derive(Serialize, Deserialize, Debug, Insertable, Queryable)]
#[diesel(table_name = hyperion_swap_events)]
pub struct HyperionSwapEvent {
    pub transaction_version: i64,
    pub event_index: i64,

    pub transaction_block_height: i64,
    pub transaction_timestamp: NaiveDateTime,

    pub sender: String,
    pub pool_address: String,

    pub token_x_type: String,
    pub token_y_type: String,

    pub amount_x_in: BigDecimal,
    pub amount_y_in: BigDecimal,

    pub amount_x_out: BigDecimal,
    pub amount_y_out: BigDecimal,

    pub sqrt_price_after: Option<BigDecimal>,
    pub current_tick_after: Option<i64>,
    pub liquidity: Option<BigDecimal>,

    pub event_data: Value,
    pub inserted_at: NaiveDateTime,
}


// Raw event data deserialization

#[derive(Debug, Deserialize)]
pub struct RawSwapEventData {


    pub sender: Option<String>,
    pub pool_address: Option<String>,
    pub token_x_type: Option<RawTypeInfo>,
    pub token_y_type: Option<RawTypeInfo>,

    pub amount_x_in: Option<String>,
    pub amount_y_in: Option<String>,

    pub amount_x_out: Option<String>,
    pub amount_y_out: Option<String>,
    pub fee_amount: Option<String>,
    #[serde(default)]
    pub sqrt_price_after: Option<String>,
    #[serde(default)]
    pub current_tick_after: Option<serde_json:Value>,
    #[serde(default)]
    pub liquidity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawTypeInfo {
    pub account_address: Option<String>,
    pub module_name: Option<String>,
    pub struct_name: Option<String>,
}


impl RawTypeInfo {
    pub fn to_type_string(&self) -> String {
        match (
            &self.account_address,
            &self.module_name,
            &self.struct_name,
        ) {
            (Some(addr), Some(module), Some(struct_name)) => {
                format!("{}::{}::{}", addr, module, struct_name)
            }
            _ => "unknown".to_string(),
        }
    }
}

// Helpers

fn parse_u128_str(s: Option<(&String)>) -> BigDecimal {
    s.and_then(|v| v.parse::<BigDecimal>().ok())
    .unwrap_or_else(|| BigDecimal::from(0))
}

impl HyperionSwapEvent {
    pub fn from_raw_event(
        txn_value: i64,
        event_index: i64,
        block_height: i64,
        txn_timestamp: NaiveDateTime,
        _event_type: &str,
        event_data: &Value,
    ) -> anyhow::Result<Self> {
        let raw: RawSwapEventData = serde_json::from_value(event_data.clone().map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse SwapEvent data at version {}: {}",
                txn_version,
                e
            )
        }))?;

        let sender = raw
        .sender
        .clone()
        .unwrap_or_else(|| "0x0".to_string());

    // The pool address may come from the event data or from the event account address.
    // For Hyperion the pool object address is embedded in the event data.
    let pool_address = raw
        .pool_address
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let token_x_type = raw
        .token_x_type
        .as_ref()
        .map(|t| t.to_type_string())
        .unwrap_or_else(|| "unknown".to_string());

    let token_y_type = raw
        .token_y_type
        .as_ref()
        .map(|t| t.to_type_string())
        .unwrap_or_else(|| "unknown".to_string());

    let amount_x_in = parse_u128_str(raw.amount_x_in.as_ref());
    let amount_y_in = parse_u128_str(raw.amount_y_in.as_ref());
    let amount_x_out = parse_u128_str(raw.amount_x_out.as_ref());
    let amount_y_out = parse_u128_str(raw.amount_y_out.as_ref());
    let fee_amount = parse_u128_str(raw.fee_amount.as_ref());

    let sqrt_price_after = raw
        .sqrt_price_after
        .as_deref()
        .and_then(|v| v.parse::<BigDecimal>().ok());

    let current_tick_after = match &raw.current_tick_after {
        Some(Value::Number(n)) => n.as_i64(),
        Some(Value::String(s)) => s.parse::<i64>().ok(),
        _ => None,
    };

    let liquidity = raw
        .liquidity
        .as_deref()
        .and_then(|v| v.parse::<BigDecimal>().ok());

    Ok(HyperionSwapEvent {
        transaction_version: txn_version,
        event_index,
        transaction_block_height: block_height,
        transaction_timestamp: txn_timestamp,
        sender,
        pool_address,
        token_x_type,
        token_y_type,
        amount_x_in,
        amount_y_in,
        amount_x_out,
        amount_y_out,
        fee_amount,
        sqrt_price_after,
        current_tick_after,
        liquidity,
        event_data: event_data.clone(),
        inserted_at: chrono::Utc::now().naive_utc(),
    })
    }
}
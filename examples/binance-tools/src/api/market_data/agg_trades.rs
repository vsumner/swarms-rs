use anyhow::Result;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

const API_URL: &str = "/api/v3/aggTrades";

/// Get compressed, aggregate trades. Trades that fill at the time, from the same taker order, with the same price will have the quantity aggregated.
pub async fn agg_trades(
    client: &Client,
    base_url: &str,
    mut params: AggTradesRequest,
) -> Result<Vec<AggTradesResponse>> {
    params.validate()?;
    if params.limit.is_none() {
        params.limit = Some(500);
    }

    let url = format!("{}{}", base_url, API_URL);
    let response = client.get(&url).query(&params).send().await?;
    response.error_for_status_ref()?;

    Ok(response.json().await?)
}

#[derive(Debug, Validate, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// The difference from trades is that the transactions of the same taker with multiple makers at the same time and the same price will be combined into one record.
///
/// If no filter parameters (fromId, startTime, endTime) are sent, the most recent transaction records will be returned by default.
pub struct AggTradesRequest {
    pub symbol: String,
    /// From which transaction ID to start returning. If not specified, the most recent transaction records will be returned by default.
    pub from_id: Option<i64>,
    /// Unix timestamp in milliseconds. Return the results starting from the transaction records after that moment
    pub start_time: Option<i64>,
    /// Unix timestamp in milliseconds. Return the results ending at the transaction records before that moment
    pub end_time: Option<i64>,
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AggTradesResponse {
    /// Aggregate trade ID
    pub a: i64,
    /// Price
    pub p: String,
    /// Quantity
    pub q: String,
    /// First trade ID
    pub f: i64,
    /// Last trade ID
    pub l: i64,

    #[serde(rename = "T")]
    /// Unix timestamp in milliseconds.
    pub T: i64,
    /// Is the buyer the market maker?
    pub m: bool,
    #[serde(rename = "M")]
    /// Was the trade the best price match? (can be ignored, always true)
    pub M: bool,
}

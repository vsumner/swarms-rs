use anyhow::Result;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const API_URL: &str = "/api/v3/avgPrice";

/// Current average price for a symbol.
pub async fn avg_price(
    client: &Client,
    base_url: &str,
    params: AvgPriceRequest,
) -> Result<AvgPriceResponse> {
    let url = format!("{}{}", base_url, API_URL);
    let response = client.get(&url).query(&params).send().await?;
    response.error_for_status_ref()?;

    Ok(response.json().await?)
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Request current average price for a symbol.
pub struct AvgPriceRequest {
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AvgPriceResponse {
    /// Average price interval (in minutes)
    pub mins: u64,
    /// Average price
    pub price: String,
    /// Last trade time. Unix timestamp in milliseconds.
    pub close_time: u64,
}

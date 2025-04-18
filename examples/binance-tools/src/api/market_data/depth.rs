use anyhow::Result;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

const API_URL: &str = "/api/v3/depth";

/// Depth information.
pub async fn depth(
    client: &Client,
    base_url: &str,
    mut params: DepthRequest,
) -> Result<DepthResponse> {
    params.validate()?;

    if params.limit.is_none() {
        params.limit = Some(100);
    }

    let url = format!("{}{}", base_url, API_URL);
    let response = client.get(&url).query(&params).send().await?;
    response.error_for_status_ref()?;

    let response = response.json::<DepthResponse>().await?;
    Ok(response)
}

#[derive(Debug, Validate, Serialize, Deserialize, JsonSchema)]
pub struct DepthRequest {
    pub symbol: String,
    #[validate(range(min = 1, max = 5000))]
    pub limit: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Depth information.
/// # Example
/// ```
/// {
///   "lastUpdateId": 1027024,
///   "bids": [
///     [
///       "4.00000000",     // PRICE
///       "431.00000000"    // QTY
///     ]
///   ],
///   "asks": [
///     [
///       "4.00000200",
///       "12.00000000"
///     ]
///   ]
/// }
/// ```
pub struct DepthResponse {
    pub last_update_id: i64,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
}

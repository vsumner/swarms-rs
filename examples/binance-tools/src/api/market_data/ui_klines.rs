use anyhow::Result;
use reqwest::Client;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::klines::KLineInterval;

const API_URL: &str = "/api/v3/uiKlines";

/// The request is similar to klines having the same parameters and response.
/// uiKlines return modified kline data, optimized for presentation of candlestick charts
pub async fn ui_klines(
    client: &Client,
    base_url: &str,
    mut params: UIKlinesRequest,
) -> Result<Vec<UIKlinesResponse>> {
    params.validate()?;
    if params.limit.is_none() {
        params.limit = Some(500);
    }

    if params.time_zone.is_none() {
        params.time_zone = Some("0".to_owned());
    }

    let url = format!("{}{}", base_url, API_URL);
    let response = client.get(&url).query(&params).send().await?;
    response.error_for_status_ref()?;

    Ok(response.json().await?)
}

#[derive(Debug, Validate, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// The request is similar to klines having the same parameters and response.
/// uiKlines return modified kline data, optimized for presentation of candlestick charts.
///
/// - If `startTime` and `endTime` are not sent, the most recent klines are returned.
/// - Supported values for `timeZone`:
///   - Hours and minutes (e.g. `-1:00`, `05:45`)
///   - Only hours (e.g. `0`, `8`, `4`)
///   - Accepted range is strictly [-12:00 to +14:00] inclusive
/// - If `timeZone` provided, kline intervals are interpreted in that `timezone` instead of UTC.
/// - Note that `startTime` and `endTime` are always interpreted in UTC, regardless of `timeZone`.
pub struct UIKlinesRequest {
    pub symbol: String,
    /// K-line interval
    pub interval: KLineInterval,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    /// time zone, default: 0(UTC)
    pub time_zone: Option<String>,
    #[validate(range(min = 1, max = 1000))]
    pub limit: Option<i32>,
}

/// 0: Kline open time. Unix timestamp in milliseconds.
/// 1: Open price
/// 2: High price
/// 3: Low price
/// 4: Close price
/// 5: Volume
/// 6: Kline close time. Unix timestamp in milliseconds.
/// 7: Quote asset volume
/// 8: Number of trades
/// 9: Taker buy base asset volume
/// 10: Taker buy quote asset volume
/// 11: Unused field. Ignore.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UIKlinesResponse(
    /// 0: Kline open time. Unix timestamp in milliseconds.
    pub i64,
    /// 1: Open price
    pub String,
    /// 2: High price
    pub String,
    /// 3: Low price
    pub String,
    /// 4: Close price
    pub String,
    /// 5: Volume
    pub String,
    /// 6: Kline close time. Unix timestamp in milliseconds.
    pub i64,
    /// 7: Quote asset volume
    pub String,
    /// 8: Number of trades
    pub i64,
    /// 9: Taker buy base asset volume
    pub String,
    /// 10: Taker buy quote asset volume
    pub String,
    /// 11: Unused field. Ignore.
    pub String,
);

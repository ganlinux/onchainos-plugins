/// HTTP client with proxy support.
/// All HTTP requests must use this client instead of reqwest::get() or Client::new().
///
/// Verified API responses (2026-04-05):
/// GET https://api.gateio.ws/api/v4/spot/tickers?currency_pair=GTBTC_USDT
/// [{"currency_pair":"GTBTC_USDT","last":"67149","lowest_ask":"67150.6",...,"change_percentage":"-0.6",...}]
///
/// GET https://api.gateio.ws/api/v4/earn/uni/currencies/GTBTC
/// (returns empty / no data — APR endpoint may not be populated)
use reqwest::Client;
use serde::Deserialize;

pub fn build_client() -> Client {
    let mut builder = Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
    {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }
    builder.build().unwrap_or_default()
}

#[derive(Debug, Deserialize)]
pub struct Ticker {
    pub currency_pair: String,
    pub last: String,
    pub change_percentage: String,
    pub high_24h: String,
    pub low_24h: String,
    pub base_volume: String,
    pub quote_volume: String,
}

#[derive(Debug, Deserialize)]
pub struct UniCurrency {
    pub currency: Option<String>,
    pub min_lend_rate: Option<String>,
    pub max_lend_rate: Option<String>,
    pub avg_lend_rate: Option<String>,
}

/// GET /spot/tickers?currency_pair=GTBTC_USDT
pub async fn get_price(client: &Client) -> anyhow::Result<Ticker> {
    let url = format!(
        "{}/spot/tickers?currency_pair=GTBTC_USDT",
        crate::config::GATE_API_BASE
    );
    let resp: Vec<Ticker> = client.get(&url).send().await?.json().await?;
    resp.into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No ticker data for GTBTC_USDT"))
}

/// GET /earn/uni/currencies/GTBTC
/// Note: This endpoint may return empty if Gate Flex Earn for GTBTC is not currently active.
pub async fn get_apr(client: &Client) -> anyhow::Result<Option<UniCurrency>> {
    let url = format!(
        "{}/earn/uni/currencies/GTBTC",
        crate::config::GATE_API_BASE
    );
    let resp = client.get(&url).send().await?;
    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() || text.trim() == "null" || text.trim().is_empty() {
        return Ok(None);
    }

    // The endpoint may return a single object or an array
    if text.trim().starts_with('[') {
        let arr: Vec<UniCurrency> = serde_json::from_str(&text)?;
        Ok(arr.into_iter().next())
    } else {
        let obj: UniCurrency = serde_json::from_str(&text)?;
        Ok(Some(obj))
    }
}

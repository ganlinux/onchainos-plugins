use anyhow::Context;
use serde_json::Value;

use crate::config::API_BASE;

/// GET /v2/kamino-market — list all Kamino markets
pub async fn get_markets(client: &reqwest::Client) -> anyhow::Result<Value> {
    let url = format!("{API_BASE}/v2/kamino-market");
    let resp = client
        .get(&url)
        .send()
        .await
        .context("GET /v2/kamino-market")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("parse markets response")?;
    if !status.is_success() {
        return Err(anyhow::anyhow!("API error {status}: {body}"));
    }
    Ok(body)
}

/// GET /kamino-market/{pubkey}/reserves/metrics — reserve metrics for a market
pub async fn get_reserves_metrics(
    client: &reqwest::Client,
    market_pubkey: &str,
) -> anyhow::Result<Value> {
    let url = format!("{API_BASE}/kamino-market/{market_pubkey}/reserves/metrics");
    let resp = client
        .get(&url)
        .send()
        .await
        .context("GET reserves/metrics")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("parse reserves metrics response")?;
    if !status.is_success() {
        return Err(anyhow::anyhow!("API error {status}: {body}"));
    }
    Ok(body)
}

/// GET /kamino-market/{marketPubkey}/users/{userPubkey}/obligations — user obligations
pub async fn get_user_obligations(
    client: &reqwest::Client,
    market_pubkey: &str,
    user_pubkey: &str,
) -> anyhow::Result<Value> {
    let url = format!(
        "{API_BASE}/kamino-market/{market_pubkey}/users/{user_pubkey}/obligations"
    );
    let resp = client
        .get(&url)
        .send()
        .await
        .context("GET user obligations")?;
    let status = resp.status();
    let body: Value = resp.json().await.context("parse obligations response")?;
    if !status.is_success() {
        return Err(anyhow::anyhow!("API error {status}: {body}"));
    }
    Ok(body)
}

/// POST /ktx/klend/{action} — build an unsigned Solana transaction
/// Returns the base64-encoded unsigned transaction string.
pub async fn build_transaction(
    client: &reqwest::Client,
    action: &str,
    wallet: &str,
    market: &str,
    reserve: &str,
    amount: &str,
) -> anyhow::Result<String> {
    let url = format!("{API_BASE}/ktx/klend/{action}");
    let body = serde_json::json!({
        "wallet": wallet,
        "market": market,
        "reserve": reserve,
        "amount": amount,
    });
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .with_context(|| format!("POST /ktx/klend/{action}"))?;
    let status = resp.status();
    let json: Value = resp
        .json()
        .await
        .with_context(|| format!("parse {action} response"))?;
    if !status.is_success() {
        return Err(anyhow::anyhow!("API error {status}: {json}"));
    }
    json["transaction"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("No 'transaction' field in response: {json}"))
}

/// Extract the health factor from an obligations response.
/// Returns None if obligations are empty or health factor is not present.
pub fn parse_health_factor(obligations: &Value) -> Option<f64> {
    // Try array format
    if let Some(arr) = obligations.as_array() {
        for obl in arr {
            if let Some(hf) = obl["healthFactor"]
                .as_f64()
                .or_else(|| obl["loanToValue"].as_f64())
            {
                return Some(hf);
            }
        }
    }
    // Try object with data array
    if let Some(arr) = obligations["data"].as_array() {
        for obl in arr {
            if let Some(hf) = obl["healthFactor"].as_f64() {
                return Some(hf);
            }
        }
    }
    // Top-level field
    obligations["healthFactor"].as_f64()
}

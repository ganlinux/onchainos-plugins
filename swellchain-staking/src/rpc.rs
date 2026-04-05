use anyhow::Context;
use serde_json::{json, Value};

use crate::config::{ETH_RPC_FALLBACK1, ETH_RPC_FALLBACK2, ETH_RPC_PRIMARY};

/// Build a reqwest client with proxy support.
pub fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .http1_only()
        .timeout(std::time::Duration::from_secs(30));
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

/// Low-level eth_call via JSON-RPC.
pub async fn eth_call(rpc_url: &str, to: &str, calldata: &str) -> anyhow::Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": calldata },
            "latest"
        ],
        "id": 1
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("eth_call HTTP request failed")?
        .json()
        .await
        .context("eth_call response parse failed")?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call RPC error: {}", err);
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// eth_call with automatic fallback through RPC list.
pub async fn eth_call_with_fallback(to: &str, calldata: &str) -> anyhow::Result<String> {
    let rpcs = [ETH_RPC_PRIMARY, ETH_RPC_FALLBACK1, ETH_RPC_FALLBACK2];
    let mut last_err = anyhow::anyhow!("no RPC endpoints configured");
    for rpc in &rpcs {
        match eth_call(rpc, to, calldata).await {
            Ok(r) if r != "0x" => return Ok(r),
            Ok(_) => {
                last_err = anyhow::anyhow!("eth_call returned empty result from {}", rpc);
            }
            Err(e) => {
                last_err = e;
            }
        }
    }
    Err(last_err)
}

/// Decode a single uint256 from eth_call result (raw 32-byte hex).
pub fn decode_uint256(hex: &str) -> u128 {
    let data = hex.trim_start_matches("0x");
    if data.len() < 64 {
        return 0;
    }
    u128::from_str_radix(&data[data.len().saturating_sub(32)..], 16).unwrap_or(0)
}

/// Decode address from eth_call result (last 20 bytes of 32-byte word).
pub fn decode_address(hex: &str) -> String {
    let data = hex.trim_start_matches("0x");
    if data.len() < 40 {
        return "0x0000000000000000000000000000000000000000".to_string();
    }
    format!("0x{}", &data[data.len() - 40..])
}

/// Query ERC-20 balanceOf(address).
/// Selector: 0x70a08231
pub async fn balance_of(token: &str, owner: &str) -> anyhow::Result<u128> {
    let owner_stripped = owner.strip_prefix("0x").unwrap_or(owner);
    let calldata = format!("0x70a08231{:0>64}", owner_stripped);
    let result = eth_call_with_fallback(token, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Query swETHToETHRate() from swETH proxy.
/// Selector: 0xd68b2cb6
pub async fn sweth_to_eth_rate(sweth_proxy: &str) -> anyhow::Result<u128> {
    let result = eth_call_with_fallback(sweth_proxy, "0xd68b2cb6").await?;
    Ok(decode_uint256(&result))
}

/// Query ethToSwETHRate() from swETH proxy.
/// Selector: 0x0de3ff57
pub async fn eth_to_sweth_rate(sweth_proxy: &str) -> anyhow::Result<u128> {
    let result = eth_call_with_fallback(sweth_proxy, "0x0de3ff57").await?;
    Ok(decode_uint256(&result))
}

/// Query rswETHToETHRate() from rswETH proxy.
/// Selector: 0xa7b9544e
pub async fn rsweth_to_eth_rate(rsweth_proxy: &str) -> anyhow::Result<u128> {
    let result = eth_call_with_fallback(rsweth_proxy, "0xa7b9544e").await?;
    Ok(decode_uint256(&result))
}

/// Query ethToRswETHRate() from rswETH proxy.
/// Selector: 0x780a47e0
pub async fn eth_to_rsweth_rate(rsweth_proxy: &str) -> anyhow::Result<u128> {
    let result = eth_call_with_fallback(rsweth_proxy, "0x780a47e0").await?;
    Ok(decode_uint256(&result))
}

/// Query getLastTokenIdCreated() from swEXIT proxy.
/// Selector: 0x061a499f
pub async fn get_last_token_id_created(swexit_proxy: &str) -> anyhow::Result<u128> {
    let result = eth_call_with_fallback(swexit_proxy, "0x061a499f").await?;
    Ok(decode_uint256(&result))
}

/// Query getLastTokenIdProcessed() from swEXIT proxy.
/// Selector: 0xb61d5978
pub async fn get_last_token_id_processed(swexit_proxy: &str) -> anyhow::Result<u128> {
    let result = eth_call_with_fallback(swexit_proxy, "0xb61d5978").await?;
    Ok(decode_uint256(&result))
}

/// Query getProcessedRateForTokenId(uint256) from swEXIT proxy.
/// Selector: 0xde886fb0
/// Returns (is_processed: bool, processed_rate: u128)
pub async fn get_processed_rate_for_token_id(
    swexit_proxy: &str,
    token_id: u128,
) -> anyhow::Result<(bool, u128)> {
    let token_id_hex = format!("{:064x}", token_id);
    let calldata = format!("0xde886fb0{}", token_id_hex);
    let result = eth_call_with_fallback(swexit_proxy, &calldata).await?;
    let data = result.trim_start_matches("0x");
    if data.len() < 128 {
        return Ok((false, 0));
    }
    // Returns (bool isProcessed, uint256 processedRate)
    let is_processed = &data[62..64] != "00";
    let processed_rate = u128::from_str_radix(&data[64..128], 16).unwrap_or(0);
    Ok((is_processed, processed_rate))
}

/// Format a u128 wei value as a human-readable ETH string (18 decimals).
pub fn format_eth(wei: u128) -> String {
    let whole = wei / 1_000_000_000_000_000_000u128;
    let frac = (wei % 1_000_000_000_000_000_000u128) / 1_000_000_000_000u128; // 6 decimal places
    format!("{}.{:06}", whole, frac)
}

/// Format a rate (1e18 = 1.0) as a human-readable ratio string.
pub fn format_rate(rate: u128) -> String {
    format_eth(rate)
}

/// Pretty-print a Value as JSON to stdout.
pub fn print_json(val: &Value) {
    println!("{}", serde_json::to_string_pretty(val).unwrap_or_default());
}

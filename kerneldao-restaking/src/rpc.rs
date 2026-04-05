use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::config::BSC_RPC;

/// Build an HTTP client that respects HTTPS_PROXY / HTTP_PROXY env vars.
pub fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
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

/// Perform an eth_call to the BSC RPC.
pub async fn eth_call(to: &str, data: &str) -> Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": data },
            "latest"
        ],
        "id": 1
    });

    let resp: Value = client
        .post(BSC_RPC)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(err) = resp.get("error") {
        return Err(anyhow!("eth_call error: {}", err));
    }

    resp["result"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("eth_call returned no result: {}", resp))
}

/// Decode a 32-byte hex-padded uint256 from an eth_call result.
pub fn decode_uint256(hex: &str) -> u128 {
    let s = hex.trim_start_matches("0x");
    u128::from_str_radix(s.get(s.len().saturating_sub(32)..).unwrap_or(s), 16).unwrap_or(0)
}

/// Decode an address[] from eth_call result (ABI-encoded dynamic array).
/// Returns lowercase hex addresses (with 0x prefix).
#[allow(dead_code)]
pub fn decode_address_array(hex: &str) -> Vec<String> {
    let s = hex.trim_start_matches("0x");
    if s.len() < 128 {
        return vec![];
    }
    // First 32 bytes: offset (should be 0x20)
    // Next 32 bytes: length of array
    let len_hex = s.get(64..128).unwrap_or("0");
    let len_trimmed = len_hex.trim_start_matches('0');
    let len_trimmed = if len_trimmed.is_empty() { "0" } else { len_trimmed };
    let count = usize::from_str_radix(len_trimmed, 16)
        .unwrap_or(0);

    (0..count)
        .filter_map(|i| {
            let start = 128 + i * 64;
            let end = start + 64;
            let chunk = s.get(start..end)?;
            // Address is the last 40 hex chars (20 bytes)
            let addr_hex = chunk.get(24..64)?;
            Some(format!("0x{}", addr_hex.to_lowercase()))
        })
        .collect()
}

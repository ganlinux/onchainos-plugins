use anyhow::Context;
use serde_json::{json, Value};

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

/// Low-level eth_call via JSON-RPC. Tries primary RPC; on failure retries with fallback.
pub async fn eth_call(rpc_url: &str, to: &str, calldata: &str) -> anyhow::Result<String> {
    use crate::config::RPC_FALLBACK;
    match eth_call_once(rpc_url, to, calldata).await {
        Ok(v) => Ok(v),
        Err(_) if rpc_url != RPC_FALLBACK => {
            eth_call_once(RPC_FALLBACK, to, calldata).await
        }
        Err(e) => Err(e),
    }
}

async fn eth_call_once(rpc_url: &str, to: &str, calldata: &str) -> anyhow::Result<String> {
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
    let result_str = resp["result"].as_str().unwrap_or("0x");
    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call RPC error: {}", err);
    }
    // Treat "Too many connections" response as error so fallback kicks in
    if result_str == "0x" {
        if let Some(body_str) = resp.as_str() {
            if body_str.contains("Too many") {
                anyhow::bail!("RPC rate limited");
            }
        }
    }
    Ok(result_str.to_string())
}

/// Decode a single uint256 from eth_call result hex.
pub fn decode_uint256(hex: &str) -> u128 {
    let data = hex.trim_start_matches("0x");
    if data.len() < 64 {
        return 0;
    }
    u128::from_str_radix(&data[..64], 16).unwrap_or(0)
}

/// Encode address parameter (padded to 32 bytes).
pub fn encode_address(addr: &str) -> String {
    let stripped = addr.strip_prefix("0x").unwrap_or(addr);
    format!("{:0>64}", stripped)
}

/// Encode uint256 parameter (padded to 32 bytes).
pub fn encode_uint256(val: u128) -> String {
    format!("{:064x}", val)
}

/// Call balanceOf(address) — selector 0x70a08231
pub async fn balance_of(rpc_url: &str, token: &str, user: &str) -> anyhow::Result<u128> {
    let calldata = format!("0x70a08231{}", encode_address(user));
    let result = eth_call(rpc_url, token, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call currentPrice() on Saving Module — selector 0x9d1b464a
/// Returns uint256 with 1e8 precision (e.g. 113561126 = 1.13561126 rUSD/srUSD)
pub async fn current_price(rpc_url: &str, saving_module: &str) -> anyhow::Result<u128> {
    let result = eth_call(rpc_url, saving_module, "0x9d1b464a").await?;
    Ok(decode_uint256(&result))
}

/// Call previewRedeem(uint256) on Saving Module — selector 0x4cdad506
/// Returns rUSD amount for given srUSD amount (18 decimals)
pub async fn preview_redeem(rpc_url: &str, saving_module: &str, srusd_amount: u128) -> anyhow::Result<u128> {
    let calldata = format!("0x4cdad506{}", encode_uint256(srusd_amount));
    let result = eth_call(rpc_url, saving_module, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call previewMint(uint256) on Saving Module — selector 0xb3d7f6b9
/// Returns srUSD amount for given rUSD amount (18 decimals)
pub async fn preview_mint(rpc_url: &str, saving_module: &str, rusd_amount: u128) -> anyhow::Result<u128> {
    let calldata = format!("0xb3d7f6b9{}", encode_uint256(rusd_amount));
    let result = eth_call(rpc_url, saving_module, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call underlyingBalance() on PSM — selector 0x59356c5c
/// Returns USDC balance in PSM (6 decimals)
pub async fn psm_underlying_balance(rpc_url: &str, psm: &str) -> anyhow::Result<u128> {
    let result = eth_call(rpc_url, psm, "0x59356c5c").await?;
    Ok(decode_uint256(&result))
}

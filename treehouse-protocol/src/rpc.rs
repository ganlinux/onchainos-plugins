use anyhow::Context;
use serde_json::{json, Value};

use crate::config::{
    AVAX_RPC_FALLBACK, AVAX_RPC_PRIMARY, ETH_RPC_FALLBACKS, ETH_RPC_PRIMARY,
};

/// Build an HTTP client that respects system proxy env vars.
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

/// Low-level eth_call via JSON-RPC for Ethereum (with fallback).
pub async fn eth_call_ethereum(to: &str, calldata: &str) -> anyhow::Result<String> {
    let mut urls: Vec<&str> = vec![ETH_RPC_PRIMARY];
    for fb in ETH_RPC_FALLBACKS {
        if *fb != ETH_RPC_PRIMARY {
            urls.push(fb);
        }
    }
    eth_call_with_fallbacks(&urls, to, calldata).await
}

/// Low-level eth_call via JSON-RPC for Avalanche (with fallback).
pub async fn eth_call_avalanche(to: &str, calldata: &str) -> anyhow::Result<String> {
    let urls = [AVAX_RPC_PRIMARY, AVAX_RPC_FALLBACK];
    eth_call_with_fallbacks(&urls, to, calldata).await
}

async fn eth_call_with_fallbacks(
    urls: &[&str],
    to: &str,
    calldata: &str,
) -> anyhow::Result<String> {
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": calldata },
            "latest"
        ],
        "id": 1
    });

    let mut last_err = anyhow::anyhow!("no RPC endpoints tried");
    for url in urls {
        let client = build_client();
        match client.post(*url).json(&body).send().await {
            Ok(response) => match response.json::<Value>().await {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        last_err = anyhow::anyhow!("eth_call RPC error: {}", err);
                        continue;
                    }
                    return Ok(resp["result"].as_str().unwrap_or("0x").to_string());
                }
                Err(e) => {
                    last_err = anyhow::anyhow!("eth_call response parse failed: {}", e);
                }
            },
            Err(e) => {
                last_err = anyhow::anyhow!("eth_call HTTP request failed on {}: {}", url, e);
            }
        }
    }
    Err(last_err)
}

/// Decode a single uint256 from eth_call result hex.
pub fn decode_uint256(hex_str: &str) -> u128 {
    let data = hex_str.trim_start_matches("0x");
    if data.len() < 64 {
        return 0;
    }
    u128::from_str_radix(&data[..64], 16).unwrap_or(0)
}

/// Encode an address parameter (padded to 32 bytes).
pub fn encode_address(addr: &str) -> String {
    let stripped = addr.strip_prefix("0x").unwrap_or(addr);
    format!("{:0>64}", stripped)
}

/// Encode a uint256 value to 32-byte hex (no 0x prefix).
pub fn encode_uint256(val: u128) -> String {
    format!("{:064x}", val)
}

/// Query ERC-20 balanceOf(address) — works on both Ethereum and Avalanche.
pub async fn erc20_balance_of(
    token: &str,
    account: &str,
    chain_id: u64,
) -> anyhow::Result<u128> {
    // balanceOf(address) selector = 0x70a08231
    let calldata = format!("0x70a08231{}", encode_address(account));
    let result = if chain_id == crate::config::ETH_CHAIN_ID {
        eth_call_ethereum(token, &calldata).await?
    } else {
        eth_call_avalanche(token, &calldata).await?
    };
    Ok(decode_uint256(&result))
}

/// Query ERC-4626 convertToAssets(shares) — works on both chains.
pub async fn erc4626_convert_to_assets(
    token: &str,
    shares: u128,
    chain_id: u64,
) -> anyhow::Result<u128> {
    // convertToAssets(uint256) selector = 0x07a2d13a
    let calldata = format!("0x07a2d13a{}", encode_uint256(shares));
    let result = if chain_id == crate::config::ETH_CHAIN_ID {
        eth_call_ethereum(token, &calldata).await?
    } else {
        eth_call_avalanche(token, &calldata).await?
    };
    Ok(decode_uint256(&result))
}

/// Query Curve pool get_dy(i, j, dx) to estimate swap output.
/// Uses selector for get_dy(int128,int128,uint256) = 0x5e0d443f
pub async fn curve_get_dy(pool: &str, i: i128, j: i128, dx: u128) -> anyhow::Result<u128> {
    // get_dy(int128,int128,uint256) selector = 0x5e0d443f
    // int128 is ABI-encoded as 32-byte signed integer (two's complement)
    let i_encoded = encode_int128(i);
    let j_encoded = encode_int128(j);
    let dx_encoded = encode_uint256(dx);
    let calldata = format!("0x5e0d443f{}{}{}", i_encoded, j_encoded, dx_encoded);
    let result = eth_call_ethereum(pool, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Encode an int128 value as a 32-byte ABI word (two's complement for negatives).
fn encode_int128(val: i128) -> String {
    if val >= 0 {
        format!("{:064x}", val as u128)
    } else {
        // Two's complement: u256 = 2^256 + val
        // For i128 negative, use 0xFF...FF padding
        let abs = (-val) as u128;
        // Two's complement for 32-byte word
        let twos_complement = u128::MAX - abs + 1;
        format!("ffffffffffffffffffffffffffffffff{:032x}", twos_complement)
    }
}

/// Fetch tETH/tAVAX APY from DeFiLlama yields API.
pub async fn fetch_defi_llama_apy(pool_symbol: &str) -> anyhow::Result<(f64, f64)> {
    // pool_symbol: "tETH" or "tAVAX"
    let client = build_client();
    let resp = client
        .get("https://yields.llama.fi/pools")
        .send()
        .await
        .context("Failed to fetch DeFiLlama pools")?;
    let data: Value = resp.json().await.context("Failed to parse DeFiLlama response")?;
    let pools = data["data"].as_array().ok_or_else(|| anyhow::anyhow!("no data array in DeFiLlama response"))?;

    let symbol_upper = pool_symbol.to_uppercase();
    for pool in pools {
        let project = pool["project"].as_str().unwrap_or("");
        let sym = pool["symbol"].as_str().unwrap_or("").to_uppercase();
        if project == "treehouse-protocol" && sym.contains(&symbol_upper) {
            let apy = pool["apy"].as_f64().unwrap_or(0.0);
            let tvl = pool["tvlUsd"].as_f64().unwrap_or(0.0);
            return Ok((apy, tvl));
        }
    }
    // Return 0 if not found (pool might not be indexed yet)
    Ok((0.0, 0.0))
}

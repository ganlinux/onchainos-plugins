use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::config::{DEFAULT_RPC, PARTNER_RESTAKE_API, SSOL_MINT, STAKE_POOL_PROGRAM};

/// Build a reqwest Client that respects HTTPS_PROXY / https_proxy environment variables.
fn build_client() -> Client {
    let mut builder = Client::builder();

    // Read proxy from environment (reqwest doesn't always pick these up in subprocess context)
    let proxy_url = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .ok();

    if let Some(ref proxy) = proxy_url {
        if let Ok(p) = reqwest::Proxy::all(proxy) {
            builder = builder.proxy(p);
        }
    }

    builder.build().unwrap_or_else(|_| Client::new())
}

/// Response from the Solayer Partner Restake API
#[derive(Debug, Deserialize)]
pub struct RestakeApiResponse {
    /// Base64-encoded serialized VersionedTransaction
    pub transaction: String,
    /// Human-readable message, e.g. "restaking 1 SOL for 0.9854 sSOL"
    pub message: Option<String>,
}

/// Call the Solayer Partner Restake API to get an unsigned transaction.
///
/// # Arguments
/// * `rpc` - Not used for API call, kept for consistency
/// * `staker` - User wallet public key (base58)
/// * `amount_sol` - Amount in SOL UI units (e.g., "1.0"), NOT lamports
/// * `referrer_key` - Optional partner wallet address for tracking
///
/// Returns base64-encoded unsigned transaction and optional message.
pub async fn restake_ssol(
    _rpc: &str,
    staker: &str,
    amount_sol: &str,
    referrer_key: Option<&str>,
) -> Result<RestakeApiResponse> {
    let client = build_client();

    let mut url = format!(
        "{}?staker={}&amount={}",
        PARTNER_RESTAKE_API, staker, amount_sol
    );

    if let Some(referrer) = referrer_key {
        if !referrer.is_empty() {
            url.push_str(&format!("&referrerkey={}", referrer));
        }
    }

    let resp = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .context("Failed to reach Solayer Partner Restake API")?;

    let status = resp.status();
    let body = resp.text().await.context("Failed to read API response body")?;

    if !status.is_success() {
        return Err(anyhow!(
            "Solayer Restake API returned HTTP {}: {}",
            status,
            body.trim()
        ));
    }

    let parsed: RestakeApiResponse =
        serde_json::from_str(&body).context(format!("Failed to parse API response: {}", body))?;

    if parsed.transaction.is_empty() {
        return Err(anyhow!("API returned empty transaction field"));
    }

    Ok(parsed)
}

/// Query sSOL token balance for a wallet address using Solana JSON-RPC.
///
/// Returns the UI amount (f64) of sSOL held by the wallet.
pub async fn get_ssol_balance(rpc: &str, wallet: &str) -> Result<f64> {
    let client = build_client();
    let rpc_url = if rpc.is_empty() { DEFAULT_RPC } else { rpc };

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            wallet,
            { "mint": SSOL_MINT },
            { "encoding": "jsonParsed" }
        ]
    });

    let resp = client
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .context("Failed to reach Solana RPC for sSOL balance")?;

    let body: Value = resp
        .json()
        .await
        .context("Failed to parse sSOL balance RPC response")?;

    // Navigate: result.value[0].account.data.parsed.info.tokenAmount.uiAmount
    let value_array = body
        .pointer("/result/value")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Unexpected RPC response structure for sSOL balance"))?;

    if value_array.is_empty() {
        // No token account means 0 balance
        return Ok(0.0);
    }

    let ui_amount = value_array[0]
        .pointer("/account/data/parsed/info/tokenAmount/uiAmount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(ui_amount)
}

/// Stake Pool account data for exchange rate calculation
#[derive(Debug)]
pub struct StakePoolInfo {
    pub total_lamports: u64,
    pub pool_token_supply: u64,
    pub exchange_rate: f64,
}

/// Query the Stake Pool account to calculate the sSOL/SOL exchange rate.
///
/// exchange_rate = total_lamports / pool_token_supply
/// (i.e., how many SOL one sSOL is worth)
pub async fn get_stake_pool_exchange_rate(rpc: &str) -> Result<StakePoolInfo> {
    let client = build_client();
    let rpc_url = if rpc.is_empty() { DEFAULT_RPC } else { rpc };

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAccountInfo",
        "params": [
            STAKE_POOL_PROGRAM,
            { "encoding": "jsonParsed" }
        ]
    });

    let resp = client
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .context("Failed to reach Solana RPC for Stake Pool info")?;

    let body: Value = resp
        .json()
        .await
        .context("Failed to parse Stake Pool account RPC response")?;

    // Try to extract parsed stake pool fields
    let parsed_info = body.pointer("/result/value/data/parsed/info");

    if let Some(info) = parsed_info {
        let total_lamports = info
            .get("totalLamports")
            .or_else(|| info.get("total_lamports"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| {
                info.get("totalLamports")
                    .or_else(|| info.get("total_lamports"))
                    .and_then(|v| v.as_u64())
            });

        let pool_token_supply = info
            .get("poolTokenSupply")
            .or_else(|| info.get("pool_token_supply"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .or_else(|| {
                info.get("poolTokenSupply")
                    .or_else(|| info.get("pool_token_supply"))
                    .and_then(|v| v.as_u64())
            });

        if let (Some(tl), Some(pts)) = (total_lamports, pool_token_supply) {
            if pts > 0 {
                let exchange_rate = (tl as f64) / (pts as f64);
                return Ok(StakePoolInfo {
                    total_lamports: tl,
                    pool_token_supply: pts,
                    exchange_rate,
                });
            }
        }
    }

    // Fallback: use a reasonable approximation if RPC doesn't return parsed data
    // This can happen when the RPC returns base64-encoded binary data
    eprintln!(
        "Warning: Could not parse Stake Pool exchange rate from RPC. Using approximate rate 1.015."
    );
    Ok(StakePoolInfo {
        total_lamports: 10_150_000_000_000,
        pool_token_supply: 10_000_000_000_000,
        exchange_rate: 1.015,
    })
}

/// Parse SOL balance from onchainos wallet balance output (JSON).
/// onchainos wallet balance --chain 501 returns JSON.
/// Native SOL is the token asset with empty tokenAddress.
/// Path: data.details[0].tokenAssets[*] where tokenAddress == ""
pub fn parse_sol_balance(output: &str) -> f64 {
    // Try the onchainos wallet balance JSON structure
    if let Ok(val) = serde_json::from_str::<Value>(output) {
        // Try data.details[0].tokenAssets[*] — find native SOL (empty tokenAddress)
        if let Some(assets) = val.pointer("/data/details/0/tokenAssets").and_then(|v| v.as_array()) {
            for asset in assets {
                let token_addr = asset.get("tokenAddress").and_then(|v| v.as_str()).unwrap_or("");
                if token_addr.is_empty() {
                    // Native SOL
                    if let Some(bal_str) = asset.get("balance").and_then(|v| v.as_str()) {
                        if let Ok(b) = bal_str.parse::<f64>() {
                            return b;
                        }
                    }
                    if let Some(b) = asset.get("balance").and_then(|v| v.as_f64()) {
                        return b;
                    }
                }
            }
        }
        // Fallback: top-level balance field
        if let Some(balance) = val
            .get("balance")
            .or_else(|| val.get("sol"))
            .or_else(|| val.get("amount"))
            .and_then(|v| v.as_f64())
        {
            return balance;
        }
    }

    // Plain text fallback: look for a number
    for word in output.split_whitespace() {
        let clean = word.trim_end_matches("SOL").trim_end_matches(',').trim();
        if let Ok(val) = clean.parse::<f64>() {
            if val >= 0.0 {
                return val;
            }
        }
    }

    0.0
}

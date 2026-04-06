// Direct eth_call to HyperEVM RPC — no onchainos required for read operations.
//
// API response sample (abridged):
// { "reserves": [ { "underlyingAsset":"0x55...", "symbol":"WHYPE", "decimals":"18",
//   "liquidityRate":"4517915...", "variableBorrowRate":"8401930...",
//   "availableLiquidity":"13708654...", "totalScaledVariableDebt":"2752101...",
//   "baseLTVasCollateral":"6000", "reserveLiquidationThreshold":"7520",
//   "isActive":true, "isFrozen":false, "borrowingEnabled":true, ... } ] }

use crate::config::RPC_URL;
use anyhow::anyhow;
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

/// Raw eth_call: returns the hex-encoded result string (no "0x" prefix guaranteed).
pub async fn eth_call(to: &str, data: &str) -> anyhow::Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_call",
        "params": [{"to": to, "data": data}, "latest"]
    });
    let resp: Value = client
        .post(RPC_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    if let Some(err) = resp.get("error") {
        return Err(anyhow!("eth_call error: {}", err));
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// Decode word at byte offset `word_index` (each word = 64 hex chars) from result hex.
pub fn decode_word(hex: &str, word_index: usize) -> u128 {
    let h = hex.trim_start_matches("0x");
    let start = word_index * 64;
    if h.len() < start + 64 {
        return 0;
    }
    let slice = &h[start..start + 64];
    u128::from_str_radix(slice, 16).unwrap_or(0)
}

/// Convert a 8-decimals USD value to human-readable dollars.
pub fn base_to_usd(val: u128) -> f64 {
    (val as f64) / 1e8
}

/// Convert 18-decimal health factor to f64.
pub fn hf_to_f64(val: u128) -> f64 {
    (val as f64) / 1e18
}

/// getUserAccountData(address user) — selector 0xbf92857c
/// Returns (totalCollateralBase, totalDebtBase, availableBorrowsBase,
///          currentLiquidationThreshold, ltv, healthFactor)
pub async fn get_user_account_data(pool: &str, user: &str) -> anyhow::Result<Value> {
    let user_hex = user.trim_start_matches("0x");
    let user_padded = format!("{:0>64}", user_hex);
    let data = format!("0xbf92857c{}", user_padded);
    let result = eth_call(pool, &data).await?;
    let h = result.trim_start_matches("0x");
    if h.len() < 6 * 64 {
        return Err(anyhow!("Unexpected eth_call result length: {}", result));
    }
    let total_collateral = decode_word(h, 0);
    let total_debt = decode_word(h, 1);
    let available_borrows = decode_word(h, 2);
    let liquidation_threshold = decode_word(h, 3);
    let ltv = decode_word(h, 4);
    let health_factor = decode_word(h, 5);
    Ok(serde_json::json!({
        "totalCollateralUsd": base_to_usd(total_collateral),
        "totalDebtUsd": base_to_usd(total_debt),
        "availableBorrowsUsd": base_to_usd(available_borrows),
        "currentLiquidationThreshold": liquidation_threshold,
        "ltv": ltv,
        "healthFactor": hf_to_f64(health_factor),
        "raw": {
            "totalCollateralBase": total_collateral.to_string(),
            "totalDebtBase": total_debt.to_string(),
            "availableBorrowsBase": available_borrows.to_string(),
            "healthFactorRaw": health_factor.to_string()
        }
    }))
}

/// getUserReserveData(address asset, address user) — selector 0x28dd0f6e
/// Returns (currentATokenBalance, currentStableDebt, currentVariableDebt,
///          scaledVariableDebt, liquidityRate, stableBorrowRate,
///          scaledATokenBalance, usageAsCollateralEnabled, stableDebtLastUpdateTimestamp)
pub async fn get_user_reserve_data(
    data_provider: &str,
    asset: &str,
    user: &str,
) -> anyhow::Result<Value> {
    let asset_hex = asset.trim_start_matches("0x");
    let user_hex = user.trim_start_matches("0x");
    let calldata = format!(
        "0x28dd0f6e{:0>64}{:0>64}",
        asset_hex, user_hex
    );
    let result = eth_call(data_provider, &calldata).await?;
    let h = result.trim_start_matches("0x");
    let a_token_balance = decode_word(h, 0);
    let variable_debt = decode_word(h, 2);
    let usage_as_collateral = decode_word(h, 7) != 0;
    Ok(serde_json::json!({
        "asset": asset,
        "currentATokenBalance": a_token_balance.to_string(),
        "currentVariableDebt": variable_debt.to_string(),
        "usageAsCollateralEnabled": usage_as_collateral
    }))
}

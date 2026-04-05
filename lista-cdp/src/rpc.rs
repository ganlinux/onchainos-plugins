use serde_json::{json, Value};
use crate::config::{BSC_FALLBACK_RPCS, encode_address};

fn build_client() -> reqwest::Client {
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

/// Low-level eth_call via JSON-RPC with BSC fallback endpoints.
pub async fn eth_call(to: &str, calldata: &str) -> anyhow::Result<String> {
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
    for url in BSC_FALLBACK_RPCS {
        let client = build_client();
        match client.post(*url).json(&body).send().await {
            Ok(response) => match response.json::<Value>().await {
                Ok(resp) => {
                    if let Some(err) = resp.get("error") {
                        last_err = anyhow::anyhow!("eth_call RPC error at {}: {}", url, err);
                        continue;
                    }
                    return Ok(resp["result"].as_str().unwrap_or("0x").to_string());
                }
                Err(e) => {
                    last_err =
                        anyhow::anyhow!("eth_call response parse failed at {}: {}", url, e);
                }
            },
            Err(e) => {
                last_err = anyhow::anyhow!("eth_call HTTP request failed on {}: {}", url, e);
            }
        }
    }
    Err(last_err)
}

/// Decode a single uint256 (first 32 bytes) from eth_call result hex.
pub fn decode_uint256(hex: &str) -> u128 {
    let data = hex.trim_start_matches("0x");
    if data.len() < 64 {
        return 0;
    }
    u128::from_str_radix(&data[..64], 16).unwrap_or(0)
}

/// Decode a single int256 (first 32 bytes) from eth_call result hex.
/// Returns as i128 (sufficient for sane lisUSD amounts).
pub fn decode_int256(hex: &str) -> i128 {
    let data = hex.trim_start_matches("0x");
    if data.len() < 64 {
        return 0;
    }
    let raw = u128::from_str_radix(&data[..64], 16).unwrap_or(0);
    // Check sign bit (MSB of 32-byte word): top 128 bits stripped, but
    // for practical lisUSD amounts this fits in i128 safely.
    raw as i128
}

// ─── Interaction queries ───────────────────────────────────────────────────

/// locked(address token, address usr) → uint256
/// Returns amount of slisBNB locked as CDP collateral (18 decimals).
pub async fn locked(interaction: &str, token: &str, usr: &str) -> anyhow::Result<u128> {
    // selector: 0xdb20266f
    let calldata = format!("0xdb20266f{}{}", encode_address(token), encode_address(usr));
    let result = eth_call(interaction, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// borrowed(address token, address usr) → uint256
/// Returns amount of lisUSD currently borrowed (18 decimals).
pub async fn borrowed(interaction: &str, token: &str, usr: &str) -> anyhow::Result<u128> {
    // selector: 0xb0a02abe
    let calldata = format!("0xb0a02abe{}{}", encode_address(token), encode_address(usr));
    let result = eth_call(interaction, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// availableToBorrow(address token, address usr) → int256
/// Returns remaining borrow capacity in lisUSD (18 decimals).
/// Negative means over-borrowed.
pub async fn available_to_borrow(
    interaction: &str,
    token: &str,
    usr: &str,
) -> anyhow::Result<i128> {
    // selector: 0xdc7e91dd
    let calldata = format!("0xdc7e91dd{}{}", encode_address(token), encode_address(usr));
    let result = eth_call(interaction, &calldata).await?;
    Ok(decode_int256(&result))
}

/// currentLiquidationPrice(address token, address usr) → uint256
/// Returns liquidation trigger price (USD, 18 decimals).
pub async fn current_liquidation_price(
    interaction: &str,
    token: &str,
    usr: &str,
) -> anyhow::Result<u128> {
    // selector: 0xfc085c11
    let calldata = format!("0xfc085c11{}{}", encode_address(token), encode_address(usr));
    let result = eth_call(interaction, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// borrowApr(address token) → uint256
/// Returns annualized borrow rate with 1e20 precision (e.g. 4.35e18 = 4.35%).
pub async fn borrow_apr(interaction: &str, token: &str) -> anyhow::Result<u128> {
    // selector: 0x9c2b9b63
    let calldata = format!("0x9c2b9b63{}", encode_address(token));
    let result = eth_call(interaction, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// collateralRate(address token) → uint256
/// Returns max LTV as 1e18-scaled value (e.g. 0.8e18 = 80%).
pub async fn collateral_rate(interaction: &str, token: &str) -> anyhow::Result<u128> {
    // selector: 0x37ffefd4
    let calldata = format!("0x37ffefd4{}", encode_address(token));
    let result = eth_call(interaction, &calldata).await?;
    Ok(decode_uint256(&result))
}

// ─── StakeManager queries ──────────────────────────────────────────────────

/// convertSnBnbToBnb(uint256 amount) → uint256
/// Converts slisBNB amount to BNB equivalent.
pub async fn convert_slisbnb_to_bnb(stake_manager: &str, amount: u128) -> anyhow::Result<u128> {
    // selector: 0xa999d3ac
    let amount_hex = format!("{:064x}", amount);
    let calldata = format!("0xa999d3ac{}", amount_hex);
    let result = eth_call(stake_manager, &calldata).await?;
    Ok(decode_uint256(&result))
}

// ─── ERC-20 queries ────────────────────────────────────────────────────────

/// balanceOf(address account) → uint256
pub async fn balance_of(token: &str, account: &str) -> anyhow::Result<u128> {
    // selector: 0x70a08231
    let calldata = format!("0x70a08231{}", encode_address(account));
    let result = eth_call(token, &calldata).await?;
    Ok(decode_uint256(&result))
}

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

/// Fallback RPC endpoints for Ethereum mainnet (chain 1).
const ETH_FALLBACK_RPCS: &[&str] = &[
    "https://rpc.mevblocker.io",
    "https://mainnet.gateway.tenderly.co",
    "https://ethereum-rpc.publicnode.com",
    "https://eth-rpc.publicnode.com",
];

/// Low-level eth_call via JSON-RPC, with fallback to alternate endpoints on failure.
pub async fn eth_call(rpc_url: &str, to: &str, calldata: &str) -> anyhow::Result<String> {
    // Build list: primary first, then fallbacks (deduped)
    let mut urls: Vec<&str> = vec![rpc_url];
    for fb in ETH_FALLBACK_RPCS {
        if *fb != rpc_url {
            urls.push(fb);
        }
    }

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
        match client.post(url).json(&body).send().await {
            Ok(response) => {
                match response.json::<Value>().await {
                    Ok(resp) => {
                        if let Some(err) = resp.get("error") {
                            last_err = anyhow::anyhow!("eth_call RPC error: {}", err);
                            continue;
                        }
                        return Ok(resp["result"].as_str().unwrap_or("0x").to_string());
                    }
                    Err(e) => { last_err = anyhow::anyhow!("eth_call response parse failed: {}", e); }
                }
            }
            Err(e) => { last_err = anyhow::anyhow!("eth_call HTTP request failed on {}: {}", url, e); }
        }
    }
    Err(last_err)
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

/// Call getVesselDebt(asset, borrower) — selector 0x7f8da425
/// Returns (debt) as u128 in wei (18 decimals).
pub async fn get_vessel_debt(rpc_url: &str, vessel_manager: &str, asset: &str, borrower: &str) -> anyhow::Result<u128> {
    let calldata = format!("0x7f8da425{}{}", encode_address(asset), encode_address(borrower));
    let result = eth_call(rpc_url, vessel_manager, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call getVesselColl(asset, borrower) — selector 0x41f0f4bd
/// Returns (coll) as u128 in wei (18 decimals).
pub async fn get_vessel_coll(rpc_url: &str, vessel_manager: &str, asset: &str, borrower: &str) -> anyhow::Result<u128> {
    let calldata = format!("0x41f0f4bd{}{}", encode_address(asset), encode_address(borrower));
    let result = eth_call(rpc_url, vessel_manager, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call getVesselStatus(asset, borrower) — selector 0xd9721b63
/// Returns status: 0=nonExistent, 1=active, 2=closedByOwner, 3=closedByLiquidation, 4=closedByRedemption
pub async fn get_vessel_status(rpc_url: &str, vessel_manager: &str, asset: &str, borrower: &str) -> anyhow::Result<u64> {
    let calldata = format!("0xd9721b63{}{}", encode_address(asset), encode_address(borrower));
    let result = eth_call(rpc_url, vessel_manager, &calldata).await?;
    Ok(decode_uint256(&result) as u64)
}

/// Call getEntireDebtAndColl(asset, borrower) — selector 0x26f7a0d4
/// Returns (debt, coll, pendingDebtReward, pendingCollReward).
pub async fn get_entire_debt_and_coll(
    rpc_url: &str,
    vessel_manager: &str,
    asset: &str,
    borrower: &str,
) -> anyhow::Result<(u128, u128, u128, u128)> {
    let calldata = format!("0x26f7a0d4{}{}", encode_address(asset), encode_address(borrower));
    let result = eth_call(rpc_url, vessel_manager, &calldata).await?;
    let data = result.trim_start_matches("0x");
    if data.len() < 256 {
        return Ok((0, 0, 0, 0));
    }
    let debt         = u128::from_str_radix(&data[0..64],   16).unwrap_or(0);
    let coll         = u128::from_str_radix(&data[64..128], 16).unwrap_or(0);
    let pending_debt = u128::from_str_radix(&data[128..192],16).unwrap_or(0);
    let pending_coll = u128::from_str_radix(&data[192..256],16).unwrap_or(0);
    Ok((debt, coll, pending_debt, pending_coll))
}

/// Call getMcr(collateral) — selector 0x78aaf4de — from AdminContract
pub async fn get_mcr(rpc_url: &str, admin_contract: &str, collateral: &str) -> anyhow::Result<u128> {
    let calldata = format!("0x78aaf4de{}", encode_address(collateral));
    let result = eth_call(rpc_url, admin_contract, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call getMinNetDebt(collateral) — selector 0x86d10e8c — from AdminContract
pub async fn get_min_net_debt(rpc_url: &str, admin_contract: &str, collateral: &str) -> anyhow::Result<u128> {
    let calldata = format!("0x86d10e8c{}", encode_address(collateral));
    let result = eth_call(rpc_url, admin_contract, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Call getBorrowingFee(collateral) — selector 0x300581d9 — from AdminContract
pub async fn get_borrowing_fee(rpc_url: &str, admin_contract: &str, collateral: &str) -> anyhow::Result<u128> {
    let calldata = format!("0x300581d9{}", encode_address(collateral));
    let result = eth_call(rpc_url, admin_contract, &calldata).await?;
    Ok(decode_uint256(&result))
}

/// Vessel status code to human-readable string.
pub fn status_str(status: u64) -> &'static str {
    match status {
        0 => "nonExistent",
        1 => "active",
        2 => "closedByOwner",
        3 => "closedByLiquidation",
        4 => "closedByRedemption",
        _ => "unknown",
    }
}

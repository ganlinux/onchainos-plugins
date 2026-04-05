use crate::api::build_client;
use crate::config::{ETH_RPC_FALLBACKS, SOLANA_RPC_URL};
use serde_json::{json, Value};

/// Raw eth_call to an EVM RPC.
/// Returns the raw hex string from "result".
pub async fn eth_call(to: &str, data: &str, rpc_url: &str) -> anyhow::Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{"to": to, "data": data}, "latest"],
        "id": 1
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call error: {}", err);
    }
    Ok(resp["result"].as_str().unwrap_or("0x").to_string())
}

/// eth_call with Ethereum mainnet fallback chain.
pub async fn eth_call_with_fallback(to: &str, data: &str) -> anyhow::Result<String> {
    let mut last_err = anyhow::anyhow!("No RPC available");
    for url in ETH_RPC_FALLBACKS {
        match eth_call(to, data, url).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_err = e;
            }
        }
    }
    Err(last_err)
}

/// Query GTBTC ERC-20 balance for an EVM address.
/// Returns the balance in atomic units (decimals=8).
pub async fn get_evm_balance(address: &str, chain_id: u64) -> anyhow::Result<u64> {
    let token = crate::config::GTBTC_TOKEN_ADDRESS;

    // balanceOf(address) selector = 0x70a08231
    let addr_no_prefix = address.trim_start_matches("0x");
    let calldata = format!("0x70a08231{:0>64}", addr_no_prefix);

    let result = if chain_id == 1 {
        eth_call_with_fallback(token, &calldata).await?
    } else {
        let rpc = crate::config::get_rpc_url(chain_id);
        eth_call(token, &calldata, rpc).await?
    };

    // decode: 32-byte big-endian uint256
    let hex_val = result.trim_start_matches("0x");
    if hex_val.is_empty() || hex_val == "0" {
        return Ok(0);
    }
    let padded = format!("{:0>64}", hex_val);
    let bytes = hex::decode(&padded)?;
    // Take the last 8 bytes of the 32-byte value (u64 is sufficient for GTBTC amounts)
    let val = u64::from_be_bytes(bytes[24..32].try_into()?);
    Ok(val)
}

/// Query GTBTC SPL token balance on Solana via getTokenAccountsByOwner.
/// Returns the balance in atomic units (decimals=8).
pub async fn get_solana_balance(wallet_address: &str) -> anyhow::Result<u64> {
    let client = build_client();
    let mint = crate::config::GTBTC_SOLANA_MINT;

    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            wallet_address,
            {"mint": mint},
            {"encoding": "jsonParsed"}
        ]
    });

    let resp: Value = client
        .post(SOLANA_RPC_URL)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    if let Some(err) = resp.get("error") {
        anyhow::bail!("Solana RPC error: {}", err);
    }

    let value = &resp["result"]["value"];
    if !value.is_array() || value.as_array().map(|a| a.is_empty()).unwrap_or(true) {
        return Ok(0);
    }

    // Sum all token accounts for this mint
    let mut total: u64 = 0;
    if let Some(accounts) = value.as_array() {
        for account in accounts {
            let amount_str = account["account"]["data"]["parsed"]["info"]["tokenAmount"]["amount"]
                .as_str()
                .unwrap_or("0");
            total += amount_str.parse::<u64>().unwrap_or(0);
        }
    }
    Ok(total)
}

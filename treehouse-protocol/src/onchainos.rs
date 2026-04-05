use std::process::Command;
use serde_json::Value;

/// Resolve the current logged-in wallet address for a given EVM chain.
///
/// Uses `onchainos wallet balance --chain <chain_id>` (no --output json).
/// Falls back to `onchainos wallet addresses` for zero-balance wallets.
pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let chain_str = chain_id.to_string();
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_str])
        .output()?;
    let json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap_or_default();

    // Primary path: data.details[0].tokenAssets[0].address
    if let Some(addr) = json["data"]["details"]
        .get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    // Fallback 2: data.address
    let addr = json["data"]["address"].as_str().unwrap_or("").to_string();
    if !addr.is_empty() {
        return Ok(addr);
    }

    // Fallback 3: onchainos wallet addresses (zero-balance wallets)
    let addr_output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let addr_json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&addr_output.stdout))
            .unwrap_or_default();
    let chain_id_str = chain_id.to_string();

    // Response structure: data.evm (array), data.solana (array), data.xlayer (array), etc.
    // Collect all address lists into one flat iterator.
    let evm_list = addr_json["data"]["evm"].as_array();
    let xlayer_list = addr_json["data"]["xlayer"].as_array();

    let all_lists: Vec<&Vec<Value>> = [evm_list, xlayer_list]
        .into_iter()
        .flatten()
        .collect();

    // Try exact chainIndex match first
    for list in &all_lists {
        for entry in list.iter() {
            if entry["chainIndex"].as_str() == Some(&chain_id_str) {
                if let Some(a) = entry["address"].as_str() {
                    if !a.is_empty() {
                        return Ok(a.to_string());
                    }
                }
            }
        }
    }

    // Fallback: take first EVM address (all EVM chains share same address)
    if let Some(evm) = addr_json["data"]["evm"].as_array() {
        if let Some(first) = evm.first() {
            if let Some(a) = first["address"].as_str() {
                if !a.is_empty() {
                    return Ok(a.to_string());
                }
            }
        }
    }

    // Legacy fallback: data as flat array (older onchainos versions)
    if let Some(addrs) = addr_json["data"].as_array() {
        for entry in addrs {
            if entry["chainIndex"].as_str() == Some(&chain_id_str) {
                if let Some(a) = entry["address"].as_str() {
                    if !a.is_empty() {
                        return Ok(a.to_string());
                    }
                }
            }
        }
        for entry in addrs {
            let idx = entry["chainIndex"].as_str().unwrap_or("0");
            if idx != "501" && idx != "784" {
                if let Some(a) = entry["address"].as_str() {
                    if !a.is_empty() {
                        return Ok(a.to_string());
                    }
                }
            }
        }
    }

    Ok(String::new())
}

/// Submit an EVM contract call via `onchainos wallet contract-call`.
///
/// dry_run=true returns a simulated response without calling onchainos.
pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt: Option<u64>, // wei value for ETH/AVAX-valued calls
    dry_run: bool,
) -> anyhow::Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000" },
            "calldata": input_data
        }));
    }

    let chain_str = chain_id.to_string();
    let mut args = vec![
        "wallet",
        "contract-call",
        "--chain",
        &chain_str,
        "--to",
        to,
        "--input-data",
        input_data,
    ];
    let amt_str;
    if let Some(v) = amt {
        amt_str = v.to_string();
        args.extend_from_slice(&["--amt", &amt_str]);
    }
    let from_str;
    if let Some(f) = from {
        from_str = f.to_string();
        args.extend_from_slice(&["--from", &from_str]);
    }

    let output = Command::new("onchainos").args(&args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(serde_json::from_str(&stdout)?)
}

/// Extract the transaction hash from an onchainos response.
/// Checks data.swapTxHash → data.txHash → root txHash.
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["swapTxHash"]
        .as_str()
        .or_else(|| result["data"]["txHash"].as_str())
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}

/// ERC-20 approve — manually encoded (no onchainos dex approve command).
/// approve(address,uint256) selector = 0x095ea7b3
pub async fn erc20_approve(
    chain_id: u64,
    token_addr: &str,
    spender: &str,
    amount: u128,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    let spender_padded = format!("{:0>64}", spender.strip_prefix("0x").unwrap_or(spender));
    let amount_hex = format!("{:064x}", amount);
    let calldata = format!("0x095ea7b3{}{}", spender_padded, amount_hex);
    wallet_contract_call(chain_id, token_addr, &calldata, from, None, dry_run).await
}

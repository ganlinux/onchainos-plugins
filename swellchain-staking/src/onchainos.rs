use std::process::Command;
use serde_json::Value;

/// Resolve the currently logged-in wallet address for a given EVM chain.
/// Includes fallback to `onchainos wallet addresses` for zero-balance wallets.
pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let chain_str = chain_id.to_string();
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_str])
        .output()?;
    let json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&output.stdout)).unwrap_or_default();

    // fallback 1: data.details[0].tokenAssets[0].address
    if let Some(addr) = json["data"]["details"]
        .get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }

    // fallback 2: data.address
    let addr = json["data"]["address"].as_str().unwrap_or("").to_string();
    if !addr.is_empty() {
        return Ok(addr);
    }

    // fallback 3: onchainos wallet addresses (zero-balance wallets)
    let addr_output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let addr_json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&addr_output.stdout)).unwrap_or_default();
    let chain_id_str = chain_id.to_string();
    // wallet addresses response: data.evm[] / data.solana[] (NOT data[] flat array)
    if let Some(evm_addrs) = addr_json["data"]["evm"].as_array() {
        // First: exact chainIndex match
        for entry in evm_addrs {
            if entry["chainIndex"].as_str() == Some(&chain_id_str) {
                if let Some(a) = entry["address"].as_str() {
                    if !a.is_empty() {
                        return Ok(a.to_string());
                    }
                }
            }
        }
        // Second: any EVM address (all EVM chains share same address)
        for entry in evm_addrs {
            if let Some(a) = entry["address"].as_str() {
                if !a.is_empty() {
                    return Ok(a.to_string());
                }
            }
        }
    }

    Ok(String::new())
}

/// Call `onchainos wallet contract-call` for EVM chains.
/// dry_run=true returns simulated response without broadcasting.
pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt: Option<u128>, // ETH value in wei (for payable calls)
    dry_run: bool,
) -> anyhow::Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": {
                "txHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
            },
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
    let amt_str: String;
    let from_owned: String;
    if let Some(v) = amt {
        amt_str = v.to_string();
        args.extend_from_slice(&["--amt", &amt_str]);
    }
    if let Some(f) = from {
        from_owned = f.to_string();
        args.extend_from_slice(&["--from", &from_owned]);
    }

    let output = Command::new("onchainos").args(&args).output()?;
    Ok(serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?)
}

/// Extract txHash from onchainos response.
/// Priority: data.swapTxHash -> data.txHash -> root txHash
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["swapTxHash"]
        .as_str()
        .or_else(|| result["data"]["txHash"].as_str())
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}

/// ERC-20 approve helper.
/// Encodes approve(address,uint256) calldata and submits via wallet_contract_call.
pub async fn erc20_approve(
    chain_id: u64,
    token_addr: &str,
    spender: &str,
    amount: u128,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    // approve(address,uint256) selector = 0x095ea7b3
    let spender_stripped = spender.strip_prefix("0x").unwrap_or(spender);
    let spender_padded = format!("{:0>64}", spender_stripped);
    let amount_hex = format!("{:064x}", amount);
    let calldata = format!("0x095ea7b3{}{}", spender_padded, amount_hex);
    wallet_contract_call(chain_id, token_addr, &calldata, from, None, dry_run).await
}

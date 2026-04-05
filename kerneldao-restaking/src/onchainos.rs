use std::process::Command;
use serde_json::Value;

pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    let chain_str = chain_id.to_string();
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_str])
        .output()?;
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    // Try tokenAssets[0].address first (populated when wallet has balance)
    if let Some(addr) = json["data"]["details"]
        .get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    // Try data.address (some onchainos versions return this)
    let addr = json["data"]["address"].as_str().unwrap_or("").to_string();
    if !addr.is_empty() {
        return Ok(addr);
    }
    // Fallback: use `onchainos wallet addresses` to find the EVM address for the given chain
    let addrs_output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let addrs_json: Value =
        serde_json::from_str(&String::from_utf8_lossy(&addrs_output.stdout)).unwrap_or_default();
    if let Some(evm_list) = addrs_json["data"]["evm"].as_array() {
        let chain_str_ref = chain_str.as_str();
        for entry in evm_list {
            if entry["chainIndex"].as_str() == Some(chain_str_ref) {
                if let Some(a) = entry["address"].as_str() {
                    if !a.is_empty() {
                        return Ok(a.to_string());
                    }
                }
            }
        }
        // EVM addresses are the same across chains; return the first one
        if let Some(a) = evm_list.first().and_then(|e| e["address"].as_str()) {
            if !a.is_empty() {
                return Ok(a.to_string());
            }
        }
    }
    anyhow::bail!("Could not resolve wallet address. Please run: onchainos wallet login");
}

pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt: Option<u128>,
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

pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["swapTxHash"]
        .as_str()
        .or_else(|| result["data"]["txHash"].as_str())
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}

pub async fn erc20_approve(
    chain_id: u64,
    token_addr: &str,
    spender: &str,
    amount: u128,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<Value> {
    let spender_stripped = spender.strip_prefix("0x").unwrap_or(spender);
    let spender_padded = format!("{:0>64}", spender_stripped);
    let amount_hex = format!("{:064x}", amount);
    let calldata = format!("0x095ea7b3{}{}", spender_padded, amount_hex);
    wallet_contract_call(chain_id, token_addr, &calldata, from, None, dry_run).await
}

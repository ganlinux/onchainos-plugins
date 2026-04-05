use std::process::Command;
use serde_json::Value;

pub fn resolve_wallet(chain_id: u64) -> anyhow::Result<String> {
    // First try: wallet balance (works when wallet has token assets on this chain)
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", &chain_id.to_string()])
        .output()?;
    let json: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    if let Some(addr) = json["data"]["details"].get(0)
        .and_then(|d| d["tokenAssets"].get(0))
        .and_then(|t| t["address"].as_str())
    {
        if !addr.is_empty() {
            return Ok(addr.to_string());
        }
    }
    // Fallback: wallet addresses — works even when balance is zero
    let addr_output = Command::new("onchainos")
        .args(["wallet", "addresses"])
        .output()?;
    let addr_json: Value = serde_json::from_str(&String::from_utf8_lossy(&addr_output.stdout))
        .unwrap_or(Value::Null);
    // Look for EVM address matching chain_id
    if let Some(evm_list) = addr_json["data"]["evm"].as_array() {
        let chain_str = chain_id.to_string();
        for entry in evm_list {
            if entry["chainIndex"].as_str().unwrap_or("") == chain_str {
                if let Some(addr) = entry["address"].as_str() {
                    if !addr.is_empty() {
                        return Ok(addr.to_string());
                    }
                }
            }
        }
        // If no exact chain match, use first EVM address (EVM addresses are shared across chains)
        if let Some(first) = evm_list.first() {
            if let Some(addr) = first["address"].as_str() {
                if !addr.is_empty() {
                    return Ok(addr.to_string());
                }
            }
        }
    }
    Ok(String::new())
}

pub async fn wallet_contract_call(
    chain_id: u64,
    to: &str,
    input_data: &str,
    from: Option<&str>,
    amt: Option<u64>,
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
        "wallet", "contract-call",
        "--chain", &chain_str,
        "--to", to,
        "--input-data", input_data,
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
    result["data"]["swapTxHash"].as_str()
        .or_else(|| result["data"]["txHash"].as_str())
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}

/// ERC-20 approve(spender, amount) — selector 0x095ea7b3
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

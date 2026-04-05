use std::process::Command;
use serde_json::Value;

/// Resolve the currently logged-in Solana wallet address via onchainos.
pub fn resolve_wallet_solana() -> anyhow::Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", "501", "--output", "json"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)
        .map_err(|e| anyhow::anyhow!("Failed to parse onchainos output: {e}\nOutput: {stdout}"))?;
    let address = json["data"]["address"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not find address in onchainos output: {stdout}"))?;
    Ok(address.to_string())
}

/// Submit a Solana unsigned transaction via onchainos wallet contract-call.
/// The serialized_tx must be a base64-encoded unsigned transaction.
/// This should be called IMMEDIATELY after obtaining the transaction from the API
/// because Solana blockhashes expire in ~60 seconds.
pub fn wallet_contract_call_solana(serialized_tx: &str, dry_run: bool) -> anyhow::Result<Value> {
    if dry_run {
        println!("[dry-run] Would call: onchainos wallet contract-call --chain 501 --unsigned-tx <tx>");
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "" },
            "serialized_tx": serialized_tx
        }));
    }

    let output = Command::new("onchainos")
        .args([
            "wallet",
            "contract-call",
            "--chain",
            "501",
            "--to",
            "KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD",
            "--unsigned-tx",
            serialized_tx,
            "--force",
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "onchainos contract-call failed (exit {})\nstdout: {stdout}\nstderr: {stderr}",
            output.status.code().unwrap_or(-1)
        ));
    }

    serde_json::from_str(&stdout).map_err(|e| {
        anyhow::anyhow!("Failed to parse onchainos response: {e}\nOutput: {stdout}")
    })
}

/// Extract txHash from onchainos contract-call response.
pub fn extract_tx_hash(result: &Value) -> &str {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
}

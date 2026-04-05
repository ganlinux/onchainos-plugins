use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::process::Command;

use crate::config::SOLANA_CHAIN_ID;

/// Run `onchainos wallet balance --chain 501` once and return raw JSON output.
/// onchainos wallet balance returns JSON even on Solana (chain 501).
/// Handles the case where exit code is 0 but "ok": false in JSON body.
fn run_wallet_balance() -> Result<String> {
    let output = Command::new("onchainos")
        .args(["wallet", "balance", "--chain", SOLANA_CHAIN_ID])
        .output()
        .context("Failed to run 'onchainos wallet balance'. Is onchainos installed?")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        return Err(anyhow!(
            "onchainos wallet balance failed (exit {}): {}",
            output.status,
            if stderr.is_empty() { &stdout } else { &stderr }
        ));
    }

    // Check "ok": false in JSON body (onchainos may return exit 0 but ok:false on rate limit)
    if let Ok(val) = serde_json::from_str::<Value>(&stdout) {
        if val.get("ok").and_then(|v| v.as_bool()) == Some(false) {
            let err_msg = val
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            return Err(anyhow!("onchainos wallet balance returned ok:false: {}", err_msg));
        }
    }

    if stdout.is_empty() {
        return Err(anyhow!("onchainos wallet balance returned empty output"));
    }

    Ok(stdout)
}

/// Resolve wallet address AND return raw balance JSON in one call.
/// Returns (wallet_address, raw_json_output).
pub fn resolve_wallet_and_balance_solana() -> Result<(String, String)> {
    let raw = run_wallet_balance()?;

    // Parse JSON: data.details[0].tokenAssets[0].address
    let addr = if let Ok(val) = serde_json::from_str::<Value>(&raw) {
        val.pointer("/data/details/0/tokenAssets/0/address")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("Could not parse Solana wallet address from onchainos wallet balance output"))?
    } else {
        return Err(anyhow!("Failed to parse JSON from onchainos wallet balance"));
    };

    Ok((addr, raw))
}

/// Resolve the current Solana wallet address via onchainos.
/// Uses `onchainos wallet balance --chain 501` which returns JSON.
/// Address path: json["data"]["details"][0]["tokenAssets"][0]["address"]
pub fn resolve_wallet_solana() -> Result<String> {
    let (addr, _) = resolve_wallet_and_balance_solana()?;
    Ok(addr)
}

/// Query SOL balance for the current wallet via onchainos.
/// Returns raw JSON output string for further parsing.
pub fn wallet_balance_solana() -> Result<String> {
    run_wallet_balance()
}

/// Broadcast a Solana transaction via onchainos wallet contract-call.
///
/// # Arguments
/// * `to` - Target program ID (e.g., Restaking Program)
/// * `unsigned_tx_base64` - Base64-encoded serialized transaction from the API
/// * `dry_run` - If true, print the command instead of executing it
///
/// Returns the raw stdout from onchainos on success.
pub fn wallet_contract_call_solana(
    to: &str,
    unsigned_tx_base64: &str,
    dry_run: bool,
) -> Result<String> {
    // Convert base64 → base58 (onchainos requires base58 for --unsigned-tx)
    let tx_bytes = base64::engine::general_purpose::STANDARD
        .decode(unsigned_tx_base64)
        .context("Failed to decode base64 transaction from API")?;
    let tx_base58 = bs58::encode(&tx_bytes).into_string();

    let args = vec![
        "wallet",
        "contract-call",
        "--chain",
        SOLANA_CHAIN_ID,
        "--to",
        to,
        "--unsigned-tx",
        &tx_base58,
        "--force",
    ];

    if dry_run {
        println!("[dry-run] Would execute:");
        println!("  onchainos {}", args.join(" "));
        return Ok(String::from("{\"txHash\":\"dry-run\"}"));
    }

    let output = Command::new("onchainos")
        .args(&args)
        .output()
        .context("Failed to run 'onchainos wallet contract-call'")?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() {
        return Err(anyhow!(
            "onchainos wallet contract-call failed.\nstdout: {}\nstderr: {}",
            stdout,
            stderr
        ));
    }

    Ok(stdout)
}

/// Extract transaction hash from onchainos output.
/// For Solana: checks data.swapTxHash first, then data.txHash, then top-level txHash.
pub fn extract_tx_hash(output: &str) -> Option<String> {
    // Try to parse as JSON
    if let Ok(val) = serde_json::from_str::<Value>(output) {
        // Solana: data.swapTxHash
        if let Some(hash) = val
            .pointer("/data/swapTxHash")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != "pending")
        {
            return Some(hash.to_string());
        }
        // Solana: data.txHash
        if let Some(hash) = val
            .pointer("/data/txHash")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != "pending")
        {
            return Some(hash.to_string());
        }
        // Top-level txHash
        if let Some(hash) = val
            .get("txHash")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty() && *s != "pending")
        {
            return Some(hash.to_string());
        }
    }

    // Fallback: look for a long base58-like string in the raw output
    for word in output.split_whitespace() {
        if word.len() >= 43 && word.len() <= 88 {
            return Some(word.to_string());
        }
    }

    None
}

// Re-export base64 engine for use in this module
use base64::Engine;

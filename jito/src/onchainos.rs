use std::process::Command;
use serde_json::Value;
use anyhow::{anyhow, Result};

/// 解析当前登录的 Solana 钱包地址（base58）
/// Solana 不支持 --output json；地址路径：data.details[0].tokenAssets[0].address
/// 失败时自动重试最多 3 次（处理偶发 TLS 错误）
pub fn resolve_wallet_solana() -> Result<String> {
    let mut last_err = anyhow!("No attempts made");
    for attempt in 1..=3 {
        let output = Command::new("onchainos")
            .args(["wallet", "balance", "--chain", "501"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Combine stdout and stderr for error messages
        let combined = if stdout.trim().is_empty() { stderr.to_string() } else { stdout.to_string() };
        match serde_json::from_str::<Value>(&combined) {
            Ok(json) => {
                if let Some(addr) = json["data"]["details"][0]["tokenAssets"][0]["address"].as_str() {
                    return Ok(addr.to_string());
                }
                last_err = anyhow!("Could not find Solana address in onchainos output (attempt {attempt}/3). Make sure you are logged in: onchainos wallet login\nOutput: {combined}");
            }
            Err(e) => {
                last_err = anyhow!("Failed to parse onchainos wallet balance output (attempt {attempt}/3): {e}\nOutput: {combined}");
            }
        }
        if attempt < 3 {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
    Err(last_err)
}

/// onchainos defi invest — 获取 Jito stake 序列化交易
/// 返回 JSON，从中提取 .data.dataList[0].serializedData（base58）
/// 失败时自动重试最多 3 次（处理偶发 TLS 错误）
pub fn defi_invest_jito(wallet: &str, amount_lamports: u64) -> Result<Value> {
    let amount_str = amount_lamports.to_string();
    let mut last_err = anyhow!("No attempts made");
    for attempt in 1..=3 {
        let output = Command::new("onchainos")
            .args([
                "defi", "invest",
                "--investment-id", "22414",
                "--address", wallet,
                "--token", "SOL",
                "--amount", &amount_str,
                "--chain", "501",
            ])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        match serde_json::from_str::<Value>(&stdout) {
            Ok(json) => {
                // Check if ok: true and serializedData present
                if json["ok"].as_bool() == Some(true) {
                    return Ok(json);
                }
                last_err = anyhow!("defi invest returned ok:false (attempt {attempt}/3)\nOutput: {stdout}");
            }
            Err(e) => {
                last_err = anyhow!("defi invest parse error (attempt {attempt}/3): {e}\nOutput: {stdout}");
            }
        }
        if attempt < 3 {
            eprintln!("defi invest attempt {attempt} failed, retrying in 3s...");
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }
    Err(last_err)
}

/// onchainos wallet contract-call (Solana)
/// serialized_tx: base58 编码（来自 onchainos defi invest 的 serializedData，已是 base58，无需转换）
/// --force 必须加；Solana blockhash 60 秒过期，必须立即调用
pub fn wallet_contract_call_solana(to: &str, serialized_tx: &str, dry_run: bool) -> Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "" },
            "serialized_tx_preview": &serialized_tx[..20.min(serialized_tx.len())]
        }));
    }
    let output = Command::new("onchainos")
        .args([
            "wallet", "contract-call",
            "--chain", "501",
            "--to", to,
            "--unsigned-tx", serialized_tx,
            "--force",
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| anyhow!("contract-call parse error: {e}\nOutput: {stdout}"))
}

/// onchainos swap execute — JitoSOL → SOL (DEX / Jupiter)
/// 命令是 "swap execute"；不加 --force
pub fn swap_execute_jitosol_to_sol(amount_ui: f64, wallet: &str, slippage: f64, dry_run: bool) -> Result<Value> {
    if dry_run {
        return Ok(serde_json::json!({
            "ok": true,
            "dry_run": true,
            "data": { "txHash": "" }
        }));
    }
    let amount_str = amount_ui.to_string();
    let slippage_str = slippage.to_string();
    let output = Command::new("onchainos")
        .args([
            "swap", "execute",
            "--chain", "solana",
            "--from", "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn",
            "--to", "So11111111111111111111111111111111111111112",
            "--readable-amount", &amount_str,
            "--wallet", wallet,
            "--slippage", &slippage_str,
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| anyhow!("swap execute parse error: {e}\nOutput: {stdout}"))
}

/// onchainos defi detail — 获取 Jito investmentId=22414 的详细信息（APY 等）
pub fn defi_detail_jito() -> Result<Value> {
    let output = Command::new("onchainos")
        .args(["defi", "detail", "--investment-id", "22414"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| anyhow!("defi detail parse error: {e}\nOutput: {stdout}"))
}

/// onchainos wallet balance (Solana) — 获取指定 token 余额
pub fn wallet_balance_solana_token(token_address: &str) -> Result<Value> {
    let output = Command::new("onchainos")
        .args([
            "wallet", "balance",
            "--chain", "501",
            "--token-address", token_address,
        ])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&stdout)
        .map_err(|e| anyhow!("wallet balance parse error: {e}\nOutput: {stdout}"))
}

/// base64 → base58（用于 Restaking API 返回 base64 tx 的场景）
#[allow(dead_code)]
pub fn base64_to_base58(b64: &str) -> Result<String> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    let bytes = STANDARD.decode(b64.trim())
        .map_err(|e| anyhow!("base64 decode error: {e}"))?;
    Ok(bs58::encode(bytes).into_string())
}

/// 提取 txHash
pub fn extract_tx_hash(result: &Value) -> String {
    result["data"]["txHash"]
        .as_str()
        .or_else(|| result["txHash"].as_str())
        .unwrap_or("pending")
        .to_string()
}

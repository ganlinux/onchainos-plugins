use anyhow::{anyhow, Result};
use serde_json::Value;

const SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";
const KOBE_API: &str = "https://kobe.mainnet.jito.network/api/v1";
const JITOSOL_MINT: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";
const JITO_STAKE_POOL: &str = "Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb";
const JITO_VAULT_PROGRAM: &str = "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8";

/// Kobe API: 获取当前 epoch MEV 奖励数据
pub async fn get_mev_rewards() -> Result<Value> {
    let url = format!("{}/mev_rewards", KOBE_API);
    let client = reqwest::Client::new();
    let resp = client.get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send().await?
        .json::<Value>().await?;
    Ok(resp)
}

/// Solana RPC: 获取 JitoSOL 总供应量
pub async fn get_jitosol_supply() -> Result<f64> {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "getTokenSupply",
        "params": [JITOSOL_MINT]
    });
    let client = reqwest::Client::new();
    let resp: Value = client.post(SOLANA_RPC)
        .timeout(std::time::Duration::from_secs(10))
        .json(&body).send().await?.json().await?;
    let supply = resp["result"]["value"]["uiAmount"]
        .as_f64()
        .ok_or_else(|| anyhow!("Failed to parse JitoSOL supply"))?;
    Ok(supply)
}

/// Solana RPC: 获取用户 JitoSOL token 账户余额
pub async fn get_user_jitosol_balance(wallet: &str) -> Result<f64> {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "getTokenAccountsByOwner",
        "params": [
            wallet,
            {"mint": JITOSOL_MINT},
            {"encoding": "jsonParsed"}
        ]
    });
    let client = reqwest::Client::new();
    let resp: Value = client.post(SOLANA_RPC)
        .timeout(std::time::Duration::from_secs(10))
        .json(&body).send().await?.json().await?;
    let accounts = resp["result"]["value"].as_array()
        .ok_or_else(|| anyhow!("No token accounts found"))?;
    if accounts.is_empty() {
        return Ok(0.0);
    }
    let amount = accounts[0]["account"]["data"]["parsed"]["info"]["tokenAmount"]["uiAmount"]
        .as_f64()
        .unwrap_or(0.0);
    Ok(amount)
}

/// Solana RPC: 列出 Jito Vault Program 的账户（取前 20 个）
pub async fn get_vault_accounts() -> Result<Vec<Value>> {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "getProgramAccounts",
        "params": [
            JITO_VAULT_PROGRAM,
            {
                "encoding": "base64",
                "dataSlice": {"offset": 0, "length": 0},
                "withContext": true,
                "filters": [{"dataSize": 536}]
            }
        ]
    });
    let client = reqwest::Client::new();
    let resp: Value = client.post(SOLANA_RPC)
        .timeout(std::time::Duration::from_secs(20))
        .json(&body).send().await?.json().await?;
    let accounts = resp["result"]["value"].as_array()
        .or_else(|| resp["result"].as_array())
        .cloned()
        .unwrap_or_default();
    Ok(accounts.into_iter().take(20).collect())
}

pub fn jitosol_mint() -> &'static str { JITOSOL_MINT }
pub fn jito_stake_pool() -> &'static str { JITO_STAKE_POOL }
pub fn jito_vault_program() -> &'static str { JITO_VAULT_PROGRAM }

use crate::{api, onchainos};
use anyhow::Result;

pub async fn run() -> Result<()> {
    // APY from onchainos defi detail
    let detail = onchainos::defi_detail_jito()?;
    let apy_str = detail["data"]["baseRate"].as_str().unwrap_or("0");
    let apy: f64 = apy_str.parse().unwrap_or(0.0);
    let sol_price = detail["data"]["aboutToken"][0]["price"].as_str().unwrap_or("0");
    let apy_title = detail["data"]["apyDetailInfo"]["title"].as_str().unwrap_or("");

    // MEV rewards from Kobe API
    let mev = api::get_mev_rewards().await.unwrap_or(serde_json::json!({}));
    let epoch = mev["epoch"].as_u64().unwrap_or(0);
    let mev_rate = mev["mev_reward_per_lamport"].as_f64().unwrap_or(0.0);
    let total_mev = mev["total_network_mev_lamports"].as_u64().unwrap_or(0);

    // JitoSOL supply from Solana RPC
    let supply = api::get_jitosol_supply().await.unwrap_or(0.0);

    println!("Jito JitoSOL Pool Info");
    println!("======================");
    println!("APY:              {:.2}%", apy * 100.0);
    if !apy_title.is_empty() {
        println!("APY Detail:       {}", apy_title);
    }
    println!("SOL Price:        ${}", sol_price);
    println!("JitoSOL Supply:   {:.2} JitoSOL", supply);
    println!("Current Epoch:    {}", epoch);
    if mev_rate > 0.0 {
        println!("MEV Rate:         {:.4e} SOL/SOL", mev_rate);
    }
    if total_mev > 0 {
        println!("Total MEV (epoch): {} lamports", total_mev);
    }
    println!();
    println!("Key Addresses:");
    println!("  Stake Pool:     {}", api::jito_stake_pool());
    println!("  JitoSOL Mint:   {}", api::jitosol_mint());
    println!("  Vault Program:  {}", api::jito_vault_program());
    Ok(())
}

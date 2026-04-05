use anyhow::Result;

use crate::api;
use crate::config::MAIN_MARKET;
use crate::onchainos;

pub async fn run(
    client: &reqwest::Client,
    reserve: &str,
    amount: &str,
    wallet: Option<&str>,
    market: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let market_pubkey = market.unwrap_or(MAIN_MARKET);

    // Step 1: resolve wallet address
    let user_pubkey = match wallet {
        Some(w) => w.to_string(),
        None => onchainos::resolve_wallet_solana()?,
    };

    println!("Withdraw from Kamino Lend");
    println!("  Wallet:  {user_pubkey}");
    println!("  Market:  {market_pubkey}");
    println!("  Reserve: {reserve}");
    println!("  Amount:  {amount}");

    // Step 2: build unsigned transaction — must call onchainos immediately after
    let serialized_tx =
        api::build_transaction(client, "withdraw", &user_pubkey, market_pubkey, reserve, amount)
            .await?;

    // Step 3: submit via onchainos IMMEDIATELY (Solana blockhash expires in ~60s)
    let result = onchainos::wallet_contract_call_solana(&serialized_tx, dry_run)?;

    let tx_hash = onchainos::extract_tx_hash(&result);
    if dry_run {
        println!("[dry-run] Withdraw transaction built successfully (not submitted).");
        println!("  Serialized TX (base64): {}...", &serialized_tx[..serialized_tx.len().min(40)]);
    } else {
        println!("Withdraw submitted!");
        println!("  txHash: {tx_hash}");
    }

    Ok(())
}

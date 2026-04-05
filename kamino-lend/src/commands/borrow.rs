use anyhow::Result;

use crate::api;
use crate::config::{HEALTH_FACTOR_MIN, HEALTH_FACTOR_WARNING, MAIN_MARKET};
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

    println!("Borrow from Kamino Lend");
    println!("  Wallet:  {user_pubkey}");
    println!("  Market:  {market_pubkey}");
    println!("  Reserve: {reserve}");
    println!("  Amount:  {amount}");

    // Step 2: check health factor BEFORE borrowing
    println!("Checking health factor before borrow...");
    let obligations =
        api::get_user_obligations(client, market_pubkey, &user_pubkey).await;
    match obligations {
        Ok(ref data) => {
            if let Some(hf) = api::parse_health_factor(data) {
                if hf < HEALTH_FACTOR_MIN {
                    return Err(anyhow::anyhow!(
                        "Health factor {hf:.4} is below minimum ({HEALTH_FACTOR_MIN}). Borrow rejected to prevent liquidation."
                    ));
                }
                if hf < HEALTH_FACTOR_WARNING {
                    println!(
                        "[WARNING] Health factor {hf:.4} is below warning threshold ({HEALTH_FACTOR_WARNING}). Proceed with caution."
                    );
                } else {
                    println!("  Health factor: {hf:.4} [OK]");
                }
            } else {
                println!("  No existing obligations found, proceeding.");
            }
        }
        Err(e) => {
            println!("[WARNING] Could not fetch obligations for health check: {e}");
        }
    }

    // Step 3: build unsigned transaction — must call onchainos immediately after
    let serialized_tx =
        api::build_transaction(client, "borrow", &user_pubkey, market_pubkey, reserve, amount)
            .await?;

    // Step 4: submit via onchainos IMMEDIATELY (Solana blockhash expires in ~60s)
    let result = onchainos::wallet_contract_call_solana(&serialized_tx, dry_run)?;

    let tx_hash = onchainos::extract_tx_hash(&result);
    if dry_run {
        println!("[dry-run] Borrow transaction built successfully (not submitted).");
        println!("  Serialized TX (base64): {}...", &serialized_tx[..serialized_tx.len().min(40)]);
    } else {
        println!("Borrow submitted!");
        println!("  txHash: {tx_hash}");
    }

    Ok(())
}

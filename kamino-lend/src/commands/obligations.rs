use anyhow::Result;
use serde_json::Value;

use crate::api;
use crate::config::{HEALTH_FACTOR_MIN, HEALTH_FACTOR_WARNING, MAIN_MARKET};
use crate::onchainos;

pub async fn run(
    client: &reqwest::Client,
    wallet: Option<&str>,
    market: Option<&str>,
) -> Result<()> {
    let market_pubkey = market.unwrap_or(MAIN_MARKET);

    let user_pubkey = match wallet {
        Some(w) => w.to_string(),
        None => onchainos::resolve_wallet_solana()?,
    };

    println!("Fetching obligations for wallet: {user_pubkey}");
    println!("Market: {market_pubkey}");

    let obligations: Value =
        api::get_user_obligations(client, market_pubkey, &user_pubkey).await?;

    let display = if obligations.is_array() {
        obligations.clone()
    } else if let Some(arr) = obligations["data"].as_array() {
        Value::Array(arr.clone())
    } else {
        obligations.clone()
    };

    if let Some(arr) = display.as_array() {
        if arr.is_empty() {
            println!("No obligations found for this wallet in the specified market.");
            return Ok(());
        }
        println!("Obligations ({} total):", arr.len());
        for obl in arr {
            let health_factor = obl["healthFactor"]
                .as_f64()
                .or_else(|| obl["loanToValue"].as_f64());

            println!("{}", serde_json::to_string_pretty(obl)?);

            if let Some(hf) = health_factor {
                if hf < HEALTH_FACTOR_MIN {
                    println!(
                        "  [DANGER] Health factor {hf:.4} is below minimum ({HEALTH_FACTOR_MIN}). Risk of liquidation!"
                    );
                } else if hf < HEALTH_FACTOR_WARNING {
                    println!(
                        "  [WARNING] Health factor {hf:.4} is below warning threshold ({HEALTH_FACTOR_WARNING})."
                    );
                } else {
                    println!("  [OK] Health factor {hf:.4}");
                }
            }
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&display)?);
    }

    Ok(())
}

use anyhow::{anyhow, Result};

use crate::commands::encode_string;
use crate::config::{format_amount, parse_amount, BSC_CHAIN_ID, STAKER_GATEWAY};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Selector: stakeNative(string referralId) => 0xc412056b
const STAKE_NATIVE_SELECTOR: &str = "c412056b";

/// BNB has 18 decimals.
const BNB_DECIMALS: u32 = 18;

pub async fn run(amount_str: &str, referral: &str, dry_run: bool) -> Result<()> {
    let wallet = resolve_wallet(BSC_CHAIN_ID)?;
    println!("Wallet: {}", wallet);
    println!("Chain:  BSC ({})", BSC_CHAIN_ID);

    let amount_raw = parse_amount(amount_str, BNB_DECIMALS)?;
    if amount_raw == 0 {
        return Err(anyhow!("Amount converts to 0 wei. Use a larger value."));
    }

    println!();
    println!("Asset:   Native BNB");
    println!("Amount:  {} BNB ({} wei)", format_amount(amount_raw, BNB_DECIMALS), amount_raw);
    println!("Gateway: {}", STAKER_GATEWAY);
    println!("Referral: \"{}\"", referral);
    if dry_run {
        println!("[dry-run mode enabled]");
    }

    println!();
    println!("This will stake {} BNB ({} wei) as native BNB into KernelDAO.", format_amount(amount_raw, BNB_DECIMALS), amount_raw);
    println!("No approve step is needed for native BNB.");

    if !dry_run {
        println!();
        println!(">>> Please confirm the STAKE NATIVE transaction above. Press Enter to continue or Ctrl+C to cancel.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    let calldata = encode_string(STAKE_NATIVE_SELECTOR, referral);

    let result = wallet_contract_call(
        BSC_CHAIN_ID,
        STAKER_GATEWAY,
        &calldata,
        Some(&wallet),
        // msg.value in wei
        Some(amount_raw),
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    if dry_run {
        println!("[dry-run] stakeNative calldata: {}", calldata);
        println!("[dry-run] msg.value: {} wei", amount_raw);
        println!("[dry-run] txHash (simulated): {}", tx_hash);
        println!("[dry-run] No transactions were broadcast.");
    } else {
        println!("txHash: {}", tx_hash);
        println!();
        println!("Successfully staked {} BNB on KernelDAO!", format_amount(amount_raw, BNB_DECIMALS));
        println!("You are now earning Kernel Points.");
        println!();
        println!("Check your positions with:");
        println!("  kerneldao-restaking balance");
    }

    Ok(())
}

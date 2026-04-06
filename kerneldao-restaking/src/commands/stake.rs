use anyhow::{anyhow, Result};

use crate::commands::{encode_addr_uint_string};
use crate::config::{asset_info, format_amount, parse_amount, BSC_CHAIN_ID, STAKER_GATEWAY};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Selector: stake(address asset, uint256 amount, string referralId) => 0x4df42566
const STAKE_SELECTOR: &str = "4df42566";

pub async fn run(asset: &str, amount_str: &str, referral: &str, dry_run: bool) -> Result<()> {
    let wallet = resolve_wallet(BSC_CHAIN_ID)?;
    println!("Wallet: {}", wallet);
    println!("Chain:  BSC ({})", BSC_CHAIN_ID);

    // Determine decimals
    let (sym, decimals) = asset_info(asset).unwrap_or(("TOKEN", 18));
    let amount_raw = parse_amount(amount_str, decimals)?;
    if amount_raw == 0 {
        return Err(anyhow!("Amount converts to 0 atomic units. Use a larger value."));
    }

    println!();
    println!("Asset:   {} ({})", sym, asset);
    println!("Amount:  {} {} ({} wei)", format_amount(amount_raw, decimals), sym, amount_raw);
    println!("Gateway: {}", STAKER_GATEWAY);
    if dry_run {
        println!("[dry-run mode enabled]");
    }

    // Step 1: Approve
    println!();
    println!("Step 1/2: Approve StakerGateway to spend {} {}", format_amount(amount_raw, decimals), sym);
    println!("  Token:   {}", asset);
    println!("  Spender: {}", STAKER_GATEWAY);
    println!("  Amount:  {} wei", amount_raw);
    if !dry_run {
        println!();
        println!(">>> Please confirm the APPROVE transaction above. Press Enter to continue or Ctrl+C to cancel.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    let approve_result = erc20_approve(
        BSC_CHAIN_ID,
        asset,
        STAKER_GATEWAY,
        amount_raw,
        Some(&wallet),
        dry_run,
    )
    .await?;

    if dry_run {
        println!("[dry-run] approve calldata: {}", approve_result["calldata"].as_str().unwrap_or(""));
    } else {
        if approve_result["ok"].as_bool() != Some(true) {
            anyhow::bail!("approve failed: {}", approve_result["error"].as_str().unwrap_or("unknown error"));
        }
        let approve_hash = extract_tx_hash(&approve_result);
        println!("Approve txHash: {}", approve_hash);
        println!("Waiting for approve confirmation...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    // Step 2: Stake
    println!();
    println!("Step 2/2: Stake {} {} into KernelDAO", format_amount(amount_raw, decimals), sym);
    println!("  Contract: {}", STAKER_GATEWAY);
    println!("  Asset:    {}", asset);
    println!("  Amount:   {} wei", amount_raw);
    println!("  Referral: \"{}\"", referral);

    if !dry_run {
        println!();
        println!(">>> Please confirm the STAKE transaction above. Press Enter to continue or Ctrl+C to cancel.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    let calldata = encode_addr_uint_string(STAKE_SELECTOR, asset, amount_raw, referral);

    let stake_result = wallet_contract_call(
        BSC_CHAIN_ID,
        STAKER_GATEWAY,
        &calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    if dry_run {
        let stake_hash = extract_tx_hash(&stake_result);
        println!("[dry-run] stake calldata: {}", calldata);
        println!("[dry-run] txHash (simulated): {}", stake_hash);
        println!("[dry-run] No transactions were broadcast.");
    } else {
        if stake_result["ok"].as_bool() != Some(true) {
            anyhow::bail!("stake failed: {}", stake_result["error"].as_str().unwrap_or("unknown error"));
        }
        let stake_hash = extract_tx_hash(&stake_result);
        println!("Stake txHash: {}", stake_hash);
        println!();
        println!("Successfully staked {} {} on KernelDAO!", format_amount(amount_raw, decimals), sym);
        println!("You are now earning Kernel Points.");
        println!();
        println!("Check your balance with:");
        println!("  kerneldao-restaking balance --asset {}", asset);
    }

    Ok(())
}

use anyhow::{anyhow, Result};

use crate::commands::encode_addr_uint_string;
use crate::config::{asset_info, format_amount, parse_amount, BSC_CHAIN_ID, STAKER_GATEWAY};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Selector: unstake(address asset, uint256 amount, string referralId) => 0xf91daa33
const UNSTAKE_SELECTOR: &str = "f91daa33";

pub async fn run(asset: &str, amount_str: &str, referral: &str, dry_run: bool) -> Result<()> {
    let wallet = resolve_wallet(BSC_CHAIN_ID)?;
    println!("Wallet: {}", wallet);
    println!("Chain:  BSC ({})", BSC_CHAIN_ID);

    let (sym, decimals) = asset_info(asset).unwrap_or(("TOKEN", 18));
    let amount_raw = parse_amount(amount_str, decimals)?;
    if amount_raw == 0 {
        return Err(anyhow!("Amount converts to 0 atomic units. Use a larger value."));
    }

    println!();
    println!("Asset:   {} ({})", sym, asset);
    println!("Amount:  {} {} ({} wei)", format_amount(amount_raw, decimals), sym, amount_raw);
    println!("Gateway: {}", STAKER_GATEWAY);
    println!("Referral: \"{}\"", referral);
    if dry_run {
        println!("[dry-run mode enabled]");
    }

    println!();
    println!("WARNING: Unstaking initiates a 7-14 day unbonding period.");
    println!("Your tokens will NOT be immediately available after this transaction.");
    println!("After the unbonding period, you will need to claim your tokens separately.");

    if !dry_run {
        println!();
        println!(">>> Please confirm the UNSTAKE transaction above. Press Enter to continue or Ctrl+C to cancel.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    let calldata = encode_addr_uint_string(UNSTAKE_SELECTOR, asset, amount_raw, referral);

    let result = wallet_contract_call(
        BSC_CHAIN_ID,
        STAKER_GATEWAY,
        &calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    if dry_run {
        println!("[dry-run] unstake calldata: {}", calldata);
        println!("[dry-run] txHash (simulated): {}", tx_hash);
        println!("[dry-run] No transactions were broadcast.");
    } else {
        println!("Unstake txHash: {}", tx_hash);
        println!();
        println!("Unstaking request submitted for {} {}.", format_amount(amount_raw, decimals), sym);
        println!("Unbonding period: 7-14 days.");
        println!("Check the KernelDAO dashboard to track your withdrawal status.");
        println!("  https://app.kerneldao.com");
    }

    Ok(())
}

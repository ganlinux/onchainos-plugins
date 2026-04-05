use anyhow::{anyhow, Result};

use crate::commands::encode_uint_string;
use crate::config::{format_amount, parse_amount, BSC_CHAIN_ID, STAKER_GATEWAY};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Selector: unstakeNative(uint256 amount, string referralId) => 0x4693cf07
const UNSTAKE_NATIVE_SELECTOR: &str = "4693cf07";

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
    println!("WARNING: Unstaking initiates a 7-14 day unbonding period.");
    println!("Your BNB will NOT be immediately available after this transaction.");
    println!("After the unbonding period, you will need to claim your BNB separately.");

    if !dry_run {
        println!();
        println!(">>> Please confirm the UNSTAKE NATIVE transaction above. Press Enter to continue or Ctrl+C to cancel.");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
    }

    let calldata = encode_uint_string(UNSTAKE_NATIVE_SELECTOR, amount_raw, referral);

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
        println!("[dry-run] unstakeNative calldata: {}", calldata);
        println!("[dry-run] txHash (simulated): {}", tx_hash);
        println!("[dry-run] No transactions were broadcast.");
    } else {
        println!("Unstake Native txHash: {}", tx_hash);
        println!();
        println!(
            "Unstaking request submitted for {} BNB.",
            format_amount(amount_raw, BNB_DECIMALS)
        );
        println!("Unbonding period: 7-14 days.");
        println!("Check the KernelDAO dashboard to track your withdrawal status.");
        println!("  https://app.kerneldao.com");
    }

    Ok(())
}

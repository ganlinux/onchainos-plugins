/// unstake — requestWithdraw slisBNB from StakeManager
/// StakeManager.requestWithdraw(uint256) selector: 0x745400c9
/// Note: two-step process — after requestWithdraw, user must wait for unlock period
///       then call claimWithdraw(idx). This command handles Step 1 only.

use crate::config::{CHAIN_ID, STAKE_MANAGER, parse_18};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Encode requestWithdraw(uint256 amount) calldata.
fn encode_request_withdraw(amount: u128) -> String {
    let amount_hex = format!("{:064x}", amount);
    format!("0x745400c9{}", amount_hex)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let amount_wei: u128 = parse_18(amount)?;

    println!("=== Lista CDP — Unstake slisBNB (requestWithdraw) ===");
    println!("Contract: StakeManager ({})", STAKE_MANAGER);
    println!("Amount:   {} slisBNB ({} wei)", amount, amount_wei);
    println!();
    println!("Note: This initiates the withdrawal request (Step 1).");
    println!("      After the unbonding period, run claimWithdraw to receive BNB.");

    if dry_run {
        let calldata = encode_request_withdraw(amount_wei);
        println!("[dry-run] calldata: {}", calldata);
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    println!();
    println!(">>> Please confirm: requestWithdraw {} slisBNB. Proceed? [y/N]", amount);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let calldata = encode_request_withdraw(amount_wei);
    let result = wallet_contract_call(
        CHAIN_ID,
        STAKE_MANAGER,
        &calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    println!("requestWithdraw tx: {}", tx_hash);
    println!();
    println!("Withdrawal request submitted.");
    println!("Wait for the unbonding period, then call claimWithdraw to receive BNB.");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

/// stake — deposit BNB to StakeManager.deposit() payable → receive slisBNB
/// StakeManager.deposit() selector: 0xd0e30db0 (no ABI params; value via --amt)

use crate::config::{CHAIN_ID, STAKE_MANAGER};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};

pub async fn run(amt_wei: u64, dry_run: bool) -> anyhow::Result<()> {
    println!("=== Lista CDP — Stake BNB for slisBNB ===");
    println!("Contract:  StakeManager ({})", STAKE_MANAGER);
    println!("BNB value: {} wei ({} BNB)", amt_wei, (amt_wei as f64) / 1e18);
    println!("Min BNB:   0.001 BNB (1e15 wei)");
    println!();

    if amt_wei < 1_000_000_000_000_000u64 {
        anyhow::bail!(
            "Amount {} wei is below minimum 0.001 BNB (1e15 wei).",
            amt_wei
        );
    }

    if dry_run {
        println!("[dry-run] Would call StakeManager.deposit() with --amt {}", amt_wei);
        println!("[dry-run] calldata: 0xd0e30db0");
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    println!();
    println!(">>> Please confirm: stake {} wei ({} BNB) → slisBNB. Proceed? [y/N]",
        amt_wei, (amt_wei as f64) / 1e18);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    // StakeManager.deposit() payable — calldata is just the selector
    let calldata = "0xd0e30db0";
    let result = wallet_contract_call(
        CHAIN_ID,
        STAKE_MANAGER,
        calldata,
        Some(&wallet),
        Some(amt_wei),
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    println!("Stake tx: {}", tx_hash);
    println!();
    println!("BNB staked! slisBNB will appear in your wallet.");
    println!("Exchange rate: ~0.9659 slisBNB per BNB (varies with accrued rewards).");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

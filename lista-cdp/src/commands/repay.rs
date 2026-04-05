/// repay — payback lisUSD debt
/// Step 1: lisUSD.approve(Interaction, amount) — selector 0x095ea7b3
/// Step 2: Interaction.payback(address token, uint256 hayAmount) — selector 0x35ed8ab8
///         Parameters:
///           token     = slisBNB address (collateral token identifier)
///           hayAmount = lisUSD amount to repay (18 decimals)
/// Note: For full repayment, query borrowed() on-chain for exact amount (no uint256.max).

use crate::config::{CHAIN_ID, INTERACTION, LISUSD, SLISBNB, format_18, parse_18, encode_address};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{borrowed, locked};

/// Encode Interaction.payback(address token, uint256 hayAmount) calldata.
fn encode_payback(token: &str, hay_amount: u128) -> String {
    let token_padded = encode_address(token);
    let amount_hex = format!("{:064x}", hay_amount);
    format!("0x35ed8ab8{}{}", token_padded, amount_hex)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let amount_wei: u128 = parse_18(amount)?;

    println!("=== Lista CDP — Repay lisUSD ===");
    println!("Interaction: {}", INTERACTION);
    println!("lisUSD:      {}", LISUSD);
    println!("Collateral:  slisBNB ({})", SLISBNB);
    println!("Repay:       {} lisUSD", amount);
    println!();
    println!("Two-step operation:");
    println!("  Step 1: Approve lisUSD -> Interaction");
    println!("  Step 2: Payback lisUSD (3s after approve)");

    if dry_run {
        let payback_calldata = encode_payback(SLISBNB, amount_wei);
        println!();
        println!("[dry-run] approve calldata: (lisUSD.approve(Interaction, {}))", amount_wei);
        println!("[dry-run] payback calldata: {}", payback_calldata);
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // Query current debt
    let current_debt = borrowed(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let locked_amt = locked(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    println!("Current debt:      {} lisUSD", format_18(current_debt));
    println!("Locked collateral: {} slisBNB", format_18(locked_amt));
    println!();

    if current_debt == 0 {
        anyhow::bail!("No outstanding debt to repay.");
    }

    if amount_wei > current_debt {
        println!(
            "Warning: repay amount {} > current debt {}. Capped to actual debt.",
            format_18(amount_wei),
            format_18(current_debt)
        );
    }

    // ── Step 1: approve lisUSD → Interaction ─────────────────────────────
    println!("--- Step 1: Approve lisUSD -> Interaction ---");
    println!("  Token:   {} (lisUSD)", LISUSD);
    println!("  Spender: {} (Interaction)", INTERACTION);
    println!("  Amount:  {} lisUSD", format_18(amount_wei));
    println!();
    println!(">>> Please confirm Step 1 (approve lisUSD for Interaction). Proceed? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let approve_result =
        erc20_approve(CHAIN_ID, LISUSD, INTERACTION, amount_wei, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);
    println!("Waiting 3 seconds before payback to avoid nonce conflict...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // ── Step 2: payback lisUSD ────────────────────────────────────────────
    println!();
    println!("--- Step 2: Payback lisUSD ---");
    println!("  Interaction: {}", INTERACTION);
    println!("  token:       {} (slisBNB collateral key)", SLISBNB);
    println!("  hayAmount:   {} lisUSD", format_18(amount_wei));
    println!();
    println!(">>> Please confirm Step 2 (repay {} lisUSD). Proceed? [y/N]",
        format_18(amount_wei));
    let mut input2 = String::new();
    std::io::stdin().read_line(&mut input2)?;
    if !input2.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let payback_calldata = encode_payback(SLISBNB, amount_wei);
    let result = wallet_contract_call(
        CHAIN_ID,
        INTERACTION,
        &payback_calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    println!("Payback tx: {}", tx_hash);
    println!();
    println!("lisUSD repaid! Use 'lista-cdp cdp-withdraw' to retrieve your slisBNB collateral.");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

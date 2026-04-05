/// cdp-deposit — approve slisBNB → Interaction, then deposit slisBNB as CDP collateral
/// Step 1: slisBNB.approve(Interaction, amount) — selector 0x095ea7b3
/// Step 2: Interaction.deposit(participant, token, dink) — selector 0x8340f549
///         Parameters: (address participant, address token, uint256 dink)
///           participant = user wallet address
///           token       = slisBNB address
///           dink        = amount to deposit (18 decimals)

use crate::config::{CHAIN_ID, INTERACTION, SLISBNB, format_18, parse_18, encode_address};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Encode Interaction.deposit(address participant, address token, uint256 dink) calldata.
fn encode_cdp_deposit(participant: &str, token: &str, dink: u128) -> String {
    let participant_padded = encode_address(participant);
    let token_padded = encode_address(token);
    let dink_hex = format!("{:064x}", dink);
    format!("0x8340f549{}{}{}", participant_padded, token_padded, dink_hex)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let amount_wei: u128 = parse_18(amount)?;

    println!("=== Lista CDP — Deposit slisBNB Collateral ===");
    println!("Interaction contract: {}", INTERACTION);
    println!("slisBNB token:        {}", SLISBNB);
    println!("Amount:               {} slisBNB", amount);
    println!();
    println!("This is a two-step operation:");
    println!("  Step 1: Approve slisBNB -> Interaction");
    println!("  Step 2: Deposit slisBNB as CDP collateral (3s after approve)");

    if dry_run {
        let placeholder = "0x0000000000000000000000000000000000000000";
        let deposit_calldata = encode_cdp_deposit(placeholder, SLISBNB, amount_wei);
        println!();
        println!("[dry-run] approve calldata: (slisBNB.approve(Interaction, {}))", amount_wei);
        println!("[dry-run] deposit calldata: {}", deposit_calldata);
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // ── Step 1: approve slisBNB → Interaction ────────────────────────────
    println!();
    println!("--- Step 1: Approve slisBNB -> Interaction ---");
    println!("  Token:   {} (slisBNB)", SLISBNB);
    println!("  Spender: {} (Interaction)", INTERACTION);
    println!("  Amount:  {} slisBNB", format_18(amount_wei));
    println!();
    println!(">>> Please confirm Step 1 (approve slisBNB for Interaction). Proceed? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let approve_result =
        erc20_approve(CHAIN_ID, SLISBNB, INTERACTION, amount_wei, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);
    println!("Waiting 3 seconds before deposit to avoid nonce conflict...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // ── Step 2: deposit slisBNB as CDP collateral ─────────────────────────
    println!();
    println!("--- Step 2: Deposit slisBNB as CDP collateral ---");
    println!("  Interaction: {}", INTERACTION);
    println!("  participant: {}", wallet);
    println!("  token:       {} (slisBNB)", SLISBNB);
    println!("  dink:        {} slisBNB", format_18(amount_wei));
    println!();
    println!(">>> Please confirm Step 2 (deposit {} slisBNB as collateral). Proceed? [y/N]",
        format_18(amount_wei));
    let mut input2 = String::new();
    std::io::stdin().read_line(&mut input2)?;
    if !input2.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let deposit_calldata = encode_cdp_deposit(&wallet, SLISBNB, amount_wei);
    let result = wallet_contract_call(
        CHAIN_ID,
        INTERACTION,
        &deposit_calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    println!("Deposit tx: {}", tx_hash);
    println!();
    println!("slisBNB deposited as collateral! Use 'lista-cdp borrow' to borrow lisUSD.");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

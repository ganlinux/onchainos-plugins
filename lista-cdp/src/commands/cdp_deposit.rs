/// cdp-deposit — approve slisBNB → slisBNBProvider, then call slisBNBProvider.provide(dink)
/// Step 1: slisBNB.approve(slisBNBProvider, amount) — selector 0x095ea7b3
/// Step 2: slisBNBProvider.provide(dink) — selector 0x2e2ebe06
///
/// Note: Interaction.deposit() restricts slisBNB deposits to "helio provider" only.
///       slisBNBProvider is the correct entry point for slisBNB collateral.

use crate::config::{CHAIN_ID, SLISBNB, SLISBNB_PROVIDER, format_18, parse_18};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};

/// Encode slisBNBProvider.provide(uint256 dink) calldata.
/// Selector: keccak256("provide(uint256)") = 0x2e2ebe06
fn encode_provide(dink: u128) -> String {
    let dink_hex = format!("{:064x}", dink);
    format!("0x2e2ebe06{}", dink_hex)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let amount_wei: u128 = parse_18(amount)?;

    println!("=== Lista CDP — Deposit slisBNB Collateral ===");
    println!("slisBNBProvider: {}", SLISBNB_PROVIDER);
    println!("slisBNB token:   {}", SLISBNB);
    println!("Amount:          {} slisBNB", amount);
    println!();
    println!("This is a two-step operation:");
    println!("  Step 1: Approve slisBNB -> slisBNBProvider");
    println!("  Step 2: slisBNBProvider.provide(dink) — 3s after approve");

    if dry_run {
        let provide_calldata = encode_provide(amount_wei);
        println!();
        println!("[dry-run] approve calldata: (slisBNB.approve(slisBNBProvider, {}))", amount_wei);
        println!("[dry-run] provide calldata: {}", provide_calldata);
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // ── Step 1: approve slisBNB → slisBNBProvider ────────────────────────
    println!();
    println!("--- Step 1: Approve slisBNB -> slisBNBProvider ---");
    println!("  Token:   {} (slisBNB)", SLISBNB);
    println!("  Spender: {} (slisBNBProvider)", SLISBNB_PROVIDER);
    println!("  Amount:  {} slisBNB", format_18(amount_wei));
    println!();
    println!(">>> Please confirm Step 1 (approve slisBNB for slisBNBProvider). Proceed? [y/N]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let approve_result =
        erc20_approve(CHAIN_ID, SLISBNB, SLISBNB_PROVIDER, amount_wei, Some(&wallet), dry_run).await?;
    if approve_result["ok"].as_bool() != Some(true) {
        let err = approve_result["error"].as_str().unwrap_or("unknown error");
        anyhow::bail!("approve failed: {}", err);
    }
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);
    println!("Waiting 3 seconds before provide to avoid nonce conflict...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // ── Step 2: slisBNBProvider.provide(dink) ─────────────────────────────
    println!();
    println!("--- Step 2: slisBNBProvider.provide({} slisBNB) ---", format_18(amount_wei));
    println!("  Provider: {}", SLISBNB_PROVIDER);
    println!("  dink:     {} slisBNB ({} wei)", format_18(amount_wei), amount_wei);
    println!();
    println!(">>> Please confirm Step 2 (provide {} slisBNB as collateral). Proceed? [y/N]",
        format_18(amount_wei));
    let mut input2 = String::new();
    std::io::stdin().read_line(&mut input2)?;
    if !input2.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let provide_calldata = encode_provide(amount_wei);
    let result = wallet_contract_call(
        CHAIN_ID,
        SLISBNB_PROVIDER,
        &provide_calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    if result["ok"].as_bool() != Some(true) {
        let err = result["error"].as_str().unwrap_or("unknown error");
        anyhow::bail!("provide failed: {}", err);
    }
    let tx_hash = extract_tx_hash(&result);
    println!("Provide tx: {}", tx_hash);
    println!();
    println!("slisBNB deposited as collateral! Use 'lista-cdp borrow' to borrow lisUSD.");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

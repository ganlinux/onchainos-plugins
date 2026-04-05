/// borrow — borrow lisUSD against slisBNB collateral
/// Interaction.borrow(address token, uint256 hayAmount) — selector 0x4b8a3529
/// Parameters:
///   token     = slisBNB address (collateral token)
///   hayAmount = lisUSD amount to borrow (18 decimals, minimum ~15 lisUSD)
/// No approve needed: protocol mints lisUSD directly to caller.

use crate::config::{CHAIN_ID, INTERACTION, SLISBNB, format_18, parse_18, encode_address};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{available_to_borrow, borrowed, locked};

/// Encode Interaction.borrow(address token, uint256 hayAmount) calldata.
fn encode_borrow(token: &str, hay_amount: u128) -> String {
    let token_padded = encode_address(token);
    let amount_hex = format!("{:064x}", hay_amount);
    format!("0x4b8a3529{}{}", token_padded, amount_hex)
}

const MIN_BORROW_WEI: u128 = 15_000_000_000_000_000_000u128; // 15 lisUSD

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let amount_wei: u128 = parse_18(amount)?;

    println!("=== Lista CDP — Borrow lisUSD ===");
    println!("Interaction: {}", INTERACTION);
    println!("Collateral:  slisBNB ({})", SLISBNB);
    println!("Borrow:      {} lisUSD", amount);
    println!();

    if amount_wei < MIN_BORROW_WEI {
        anyhow::bail!(
            "Borrow amount {} lisUSD is below minimum 15 lisUSD.",
            amount
        );
    }

    if dry_run {
        let calldata = encode_borrow(SLISBNB, amount_wei);
        println!("[dry-run] calldata: {}", calldata);
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // Query current position
    let locked_amt = locked(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let already_borrowed = borrowed(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let available = available_to_borrow(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);

    println!("Current position:");
    println!("  Locked collateral: {} slisBNB", format_18(locked_amt));
    println!("  Already borrowed:  {} lisUSD", format_18(already_borrowed));
    println!("  Available to borrow: {} lisUSD", if available < 0 {
        format!("-{}", format_18((-available) as u128))
    } else {
        format_18(available as u128)
    });
    println!();

    if locked_amt == 0 {
        anyhow::bail!(
            "No collateral deposited. Use 'lista-cdp cdp-deposit' first."
        );
    }

    if available > 0 && (amount_wei as i128) > available {
        anyhow::bail!(
            "Borrow amount {} lisUSD exceeds available {} lisUSD. Reduce amount or add more collateral.",
            format_18(amount_wei),
            format_18(available as u128)
        );
    }

    println!(">>> Please confirm: borrow {} lisUSD against {} slisBNB collateral. Proceed? [y/N]",
        format_18(amount_wei), format_18(locked_amt));
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let calldata = encode_borrow(SLISBNB, amount_wei);
    let result = wallet_contract_call(
        CHAIN_ID,
        INTERACTION,
        &calldata,
        Some(&wallet),
        None,
        dry_run,
    )
    .await?;

    let tx_hash = extract_tx_hash(&result);
    println!("Borrow tx: {}", tx_hash);
    println!();
    println!("lisUSD borrowed successfully!");
    println!("Warning: Collateral ratio must remain >= 125% to avoid liquidation.");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

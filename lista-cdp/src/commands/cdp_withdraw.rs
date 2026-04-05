/// cdp-withdraw — withdraw slisBNB collateral from CDP
/// Interaction.withdraw(address participant, address token, uint256 dink) — selector 0xd9caed12
/// Parameters:
///   participant = user wallet address
///   token       = slisBNB address
///   dink        = amount to withdraw (18 decimals)
/// No approve needed: Interaction pushes tokens directly to user.
/// Constraint: after withdrawal, collateral ratio must remain >= 125% (if debt outstanding).

use crate::config::{CHAIN_ID, INTERACTION, SLISBNB, format_18, parse_18, encode_address};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{borrowed, locked};

/// Encode Interaction.withdraw(address participant, address token, uint256 dink) calldata.
fn encode_cdp_withdraw(participant: &str, token: &str, dink: u128) -> String {
    let participant_padded = encode_address(participant);
    let token_padded = encode_address(token);
    let dink_hex = format!("{:064x}", dink);
    format!("0xd9caed12{}{}{}", participant_padded, token_padded, dink_hex)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let amount_wei: u128 = parse_18(amount)?;

    println!("=== Lista CDP — Withdraw slisBNB Collateral ===");
    println!("Interaction: {}", INTERACTION);
    println!("slisBNB:     {}", SLISBNB);
    println!("Amount:      {} slisBNB", amount);
    println!();

    if dry_run {
        let placeholder = "0x0000000000000000000000000000000000000000";
        let calldata = encode_cdp_withdraw(placeholder, SLISBNB, amount_wei);
        println!("[dry-run] calldata: {}", calldata);
        return Ok(());
    }

    let wallet = resolve_wallet(CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // Query position
    let locked_amt = locked(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let debt_amt = borrowed(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    println!("Locked collateral: {} slisBNB", format_18(locked_amt));
    println!("Outstanding debt:  {} lisUSD", format_18(debt_amt));
    println!();

    if locked_amt == 0 {
        anyhow::bail!("No collateral to withdraw.");
    }

    if amount_wei > locked_amt {
        anyhow::bail!(
            "Withdrawal amount {} slisBNB exceeds locked {} slisBNB.",
            format_18(amount_wei),
            format_18(locked_amt)
        );
    }

    if debt_amt > 0 {
        println!(
            "Warning: You have outstanding debt of {} lisUSD.",
            format_18(debt_amt)
        );
        println!("After withdrawal, collateral ratio must remain >= 125%.");
        println!("Repay debt first with 'lista-cdp repay' for full withdrawal.");
    }

    println!(">>> Please confirm: withdraw {} slisBNB from CDP collateral. Proceed? [y/N]",
        format_18(amount_wei));
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Aborted by user.");
        return Ok(());
    }

    let calldata = encode_cdp_withdraw(&wallet, SLISBNB, amount_wei);
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
    println!("Withdraw tx: {}", tx_hash);
    println!();
    println!("slisBNB collateral withdrawn to your wallet.");
    println!("BSCScan: https://bscscan.com/tx/{}", tx_hash);

    Ok(())
}

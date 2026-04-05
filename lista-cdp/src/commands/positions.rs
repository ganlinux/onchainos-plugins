/// positions — query CDP position: collateral, debt, available borrow, liquidation price
/// All queries via eth_call (read-only, no transactions).

use crate::config::{CHAIN_ID, INTERACTION, SLISBNB, LISUSD, STAKE_MANAGER, format_18};
use crate::onchainos::resolve_wallet;
use crate::rpc::{
    available_to_borrow, balance_of, borrow_apr, borrowed, collateral_rate,
    convert_slisbnb_to_bnb, current_liquidation_price, locked,
};

pub async fn run(wallet_override: Option<&str>) -> anyhow::Result<()> {
    println!("=== Lista CDP — Position Overview ===");
    println!("Chain:       BSC Mainnet (chain 56)");
    println!("Interaction: {}", INTERACTION);
    println!("Collateral:  slisBNB ({})", SLISBNB);
    println!("Stablecoin:  lisUSD ({})", LISUSD);
    println!();

    // Resolve wallet
    let wallet = if let Some(w) = wallet_override {
        w.to_string()
    } else {
        resolve_wallet(CHAIN_ID)?
    };

    if wallet.is_empty() {
        anyhow::bail!("No wallet found on BSC (chain 56). Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);
    println!();

    // ── CDP position ──────────────────────────────────────────────────────
    let locked_amt = locked(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let debt_amt = borrowed(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let available = available_to_borrow(INTERACTION, SLISBNB, &wallet).await.unwrap_or(0);
    let liq_price = current_liquidation_price(INTERACTION, SLISBNB, &wallet)
        .await
        .unwrap_or(0);

    println!("CDP Position:");
    println!("  Locked collateral: {} slisBNB", format_18(locked_amt));
    println!("  Outstanding debt:  {} lisUSD", format_18(debt_amt));
    println!(
        "  Available borrow:  {} lisUSD{}",
        if available < 0 {
            format!("-{}", format_18((-available) as u128))
        } else {
            format_18(available as u128)
        },
        if available < 0 { " (OVER-BORROWED!)" } else { "" }
    );
    if liq_price > 0 {
        println!("  Liquidation price: ${} per slisBNB", format_18(liq_price));
    }
    println!();

    // ── Protocol parameters ───────────────────────────────────────────────
    if let Ok(apr_raw) = borrow_apr(INTERACTION, SLISBNB).await {
        let apr_pct = (apr_raw as f64) / 1e18;
        println!("Borrow APR: {:.4}%", apr_pct);
    }
    if let Ok(rate_raw) = collateral_rate(INTERACTION, SLISBNB).await {
        let ltv_pct = (rate_raw as f64) / 1e16;
        println!("Max LTV:    {:.0}% (min collateral ratio: {:.0}%)", ltv_pct, 10000.0 / ltv_pct);
    }
    println!();

    // ── Wallet balances ───────────────────────────────────────────────────
    let slisbnb_bal = balance_of(SLISBNB, &wallet).await.unwrap_or(0);
    let lisusd_bal = balance_of(LISUSD, &wallet).await.unwrap_or(0);
    println!("Wallet Balances:");
    println!("  slisBNB: {} slisBNB", format_18(slisbnb_bal));
    println!("  lisUSD:  {} lisUSD", format_18(lisusd_bal));

    // Show BNB equivalent of locked slisBNB
    if locked_amt > 0 {
        if let Ok(bnb_equiv) = convert_slisbnb_to_bnb(STAKE_MANAGER, locked_amt).await {
            println!();
            println!(
                "Collateral BNB equivalent: {} BNB (at current exchange rate)",
                format_18(bnb_equiv)
            );
        }
    }

    println!();
    println!("Health:");
    if locked_amt > 0 && debt_amt > 0 {
        // Simplified: ratio of locked/borrowed (units don't match without price)
        println!("  Note: Health factor calculation requires current slisBNB price.");
        println!("  Use current_liquidation_price above to assess liquidation risk.");
    } else if locked_amt > 0 && debt_amt == 0 {
        println!("  No outstanding debt — position is safe.");
    } else if locked_amt == 0 && debt_amt == 0 {
        println!("  No active CDP position.");
    }

    Ok(())
}

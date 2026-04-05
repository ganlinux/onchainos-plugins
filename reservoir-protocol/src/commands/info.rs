/// info: query rUSD/srUSD balances, srUSD price, and PSM USDC liquidity (read-only eth_call)

use crate::config::{format_18, format_6, RUSD, SAVING_MODULE, SRUSD, PSM_USDC, RPC_URL};
use crate::onchainos::resolve_wallet;
use crate::rpc::{balance_of, current_price, preview_redeem, psm_underlying_balance};

pub async fn run(wallet_override: Option<String>) -> anyhow::Result<()> {
    println!("=== Reservoir Protocol — Portfolio Info ===");
    println!("Chain: 1 (Ethereum Mainnet)");
    println!();

    // Resolve wallet address
    let wallet = if let Some(addr) = wallet_override {
        addr
    } else {
        let addr = resolve_wallet(1)?;
        if addr.is_empty() {
            anyhow::bail!("No wallet found on Ethereum mainnet. Run: onchainos wallet login");
        }
        addr
    };
    println!("Wallet: {}", wallet);
    println!();

    // Query rUSD balance
    let rusd_bal = balance_of(RPC_URL, RUSD, &wallet).await.unwrap_or(0);
    println!("rUSD balance:  {} rUSD", format_18(rusd_bal));

    // Query srUSD balance
    let srusd_bal = balance_of(RPC_URL, SRUSD, &wallet).await.unwrap_or(0);
    println!("srUSD balance: {} srUSD", format_18(srusd_bal));

    // Query srUSD current price
    let price = current_price(RPC_URL, SAVING_MODULE).await.unwrap_or(0);
    let price_display = price as f64 / 1e8;
    println!();
    println!("=== srUSD Exchange Rate ===");
    println!("currentPrice: {:.8} rUSD per srUSD", price_display);

    // Preview redeem if user holds srUSD
    if srusd_bal > 0 {
        let redeemable = preview_redeem(RPC_URL, SAVING_MODULE, srusd_bal).await.unwrap_or(0);
        println!("Your {} srUSD -> ~{} rUSD (if redeemed now)", format_18(srusd_bal), format_18(redeemable));
    }

    // PSM USDC liquidity
    let psm_usdc = psm_underlying_balance(RPC_URL, PSM_USDC).await.unwrap_or(0);
    println!();
    println!("=== PSM USDC Liquidity ===");
    println!("PSM USDC balance: {} USDC", format_6(psm_usdc));
    println!("(Max rUSD redeemable 1:1 for USDC via PSM)");

    println!();
    println!("Contracts:");
    println!("  rUSD:           {}", RUSD);
    println!("  srUSD:          {}", SRUSD);
    println!("  Saving Module:  {}", SAVING_MODULE);
    println!("  PSM (USDC):     {}", PSM_USDC);

    Ok(())
}

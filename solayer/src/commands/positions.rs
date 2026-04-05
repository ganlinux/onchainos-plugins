use anyhow::Result;
use clap::Args;

use crate::api::{get_ssol_balance, get_stake_pool_exchange_rate, parse_sol_balance};
use crate::config::DEFAULT_RPC;
use crate::onchainos::resolve_wallet_and_balance_solana;

#[derive(Debug, Args)]
pub struct PositionsArgs {
    /// Solana JSON-RPC endpoint (default: https://mainnet-rpc.solayer.org)
    #[arg(long, default_value = DEFAULT_RPC)]
    pub rpc: String,
}

pub async fn run(args: PositionsArgs) -> Result<()> {
    println!("Fetching Solayer positions...");

    // Step 1: Get wallet address AND SOL balance in a single onchainos call
    let (wallet, sol_output) = resolve_wallet_and_balance_solana()?;
    let sol_balance = parse_sol_balance(&sol_output);

    // Step 3: Query sSOL balance
    let ssol_balance = get_ssol_balance(&args.rpc, &wallet).await?;

    // Step 4: Query Stake Pool exchange rate
    let pool_info = get_stake_pool_exchange_rate(&args.rpc).await?;

    // Step 5: Calculate position value
    let position_value_sol = ssol_balance * pool_info.exchange_rate;
    let yield_pct = (pool_info.exchange_rate - 1.0) * 100.0;

    // Output formatted result
    println!();
    println!("=== Solayer Restaking Positions ===");
    println!("Wallet:              {}", wallet);
    println!();
    println!("--- Liquid Assets ---");
    println!("  SOL (available):   {:.4} SOL", sol_balance);
    println!();
    println!("--- Restaked Position ---");
    if ssol_balance > 0.0 {
        println!("  sSOL balance:      {:.6} sSOL", ssol_balance);
        println!(
            "  Exchange rate:     1 sSOL ≈ {:.6} SOL",
            pool_info.exchange_rate
        );
        println!("  Position value:    {:.6} SOL", position_value_sol);
        println!("  Cumulative yield:  +{:.2}%", yield_pct);
    } else {
        println!("  No sSOL position found.");
        println!("  Use 'solayer restake --amount <SOL>' to start restaking.");
    }
    println!();
    println!(
        "--- Total Portfolio ---  {:.4} SOL (liquid + restaked)",
        sol_balance + position_value_sol
    );

    Ok(())
}

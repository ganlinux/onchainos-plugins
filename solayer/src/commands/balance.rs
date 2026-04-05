use anyhow::Result;
use clap::Args;

use crate::api::{get_ssol_balance, parse_sol_balance};
use crate::config::DEFAULT_RPC;
use crate::onchainos::resolve_wallet_and_balance_solana;

#[derive(Debug, Args)]
pub struct BalanceArgs {
    /// Solana JSON-RPC endpoint (default: https://mainnet-rpc.solayer.org)
    #[arg(long, default_value = DEFAULT_RPC)]
    pub rpc: String,
}

pub async fn run(args: BalanceArgs) -> Result<()> {
    println!("Fetching wallet balances...");

    // Step 1: Get wallet address AND SOL balance in a single onchainos call
    let (wallet, sol_output) = resolve_wallet_and_balance_solana()?;
    println!("Wallet: {}", wallet);

    // Step 2: Parse SOL balance from the JSON output
    let sol_balance = parse_sol_balance(&sol_output);

    // Step 3: Query sSOL balance via Solana JSON-RPC
    let ssol_balance = get_ssol_balance(&args.rpc, &wallet).await?;

    // Output formatted result
    println!();
    println!("=== Solayer Balance Summary ===");
    println!("Wallet:       {}", wallet);
    println!("SOL balance:  {:.4} SOL", sol_balance);
    println!("sSOL balance: {:.6} sSOL", ssol_balance);
    println!();

    if ssol_balance > 0.0 {
        println!(
            "Tip: Use 'solayer positions' to see your sSOL position value in SOL."
        );
    }

    Ok(())
}

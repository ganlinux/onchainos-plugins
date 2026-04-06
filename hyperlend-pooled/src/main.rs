mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};
use commands::{borrow, get_markets, positions, repay, supply, withdraw};

#[derive(Parser)]
#[command(
    name = "hyperlend-pooled",
    about = "HyperLend Core Pools (Aave V3 fork) on HyperEVM — supply, borrow, repay, withdraw",
    version = "0.1.0"
)]
struct Cli {
    /// Simulate without broadcasting (no onchainos calls)
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch all Core Pool markets: APY, utilization, risk params
    GetMarkets(get_markets::GetMarketsArgs),

    /// Show user's supply/borrow positions and health factor
    Positions(positions::PositionsArgs),

    /// Supply an ERC-20 token to the Core Pool (approve + supply)
    Supply(supply::SupplyArgs),

    /// Borrow an asset against supplied collateral (no approval needed)
    Borrow(borrow::BorrowArgs),

    /// Repay borrowed debt (approve + repay)
    Repay(repay::RepayArgs),

    /// Withdraw supplied asset from the Core Pool (no approval needed)
    Withdraw(withdraw::WithdrawArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::GetMarkets(args) => get_markets::execute(args).await?,
        Commands::Positions(args) => positions::execute(args).await?,
        Commands::Supply(args) => supply::execute(args, cli.dry_run).await?,
        Commands::Borrow(args) => borrow::execute(args, cli.dry_run).await?,
        Commands::Repay(args) => repay::execute(args, cli.dry_run).await?,
        Commands::Withdraw(args) => withdraw::execute(args, cli.dry_run).await?,
    }
    Ok(())
}

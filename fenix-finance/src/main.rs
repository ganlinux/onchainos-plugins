mod api;
mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

use commands::{
    add_liquidity::AddLiquidityArgs,
    balance::BalanceArgs,
    positions::PositionsArgs,
    price::PriceArgs,
    remove_liquidity::RemoveLiquidityArgs,
    swap::SwapArgs,
};

#[derive(Parser)]
#[command(name = "fenix-finance", about = "Fenix Finance V3 plugin for onchainos")]
struct Cli {
    /// Chain ID (default: 81457 Blast)
    #[arg(long, default_value = "81457")]
    chain: u64,

    /// Simulate without broadcasting on-chain transactions
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get price quote for a token swap
    Price(PriceArgs),

    /// Swap tokens via Fenix SwapRouter (Algebra exactInputSingle)
    Swap(SwapArgs),

    /// List your LP positions on Fenix NFPM
    Positions(PositionsArgs),

    /// Add liquidity (mint new LP position)
    AddLiquidity(AddLiquidityArgs),

    /// Remove liquidity and collect fees from a position
    RemoveLiquidity(RemoveLiquidityArgs),

    /// Show wallet token balances on Blast
    Balance(BalanceArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let chain_id = cli.chain;
    let dry_run = cli.dry_run;

    match &cli.command {
        Commands::Price(args) => commands::price::execute(args).await?,
        Commands::Swap(args) => commands::swap::execute(args, dry_run, chain_id).await?,
        Commands::Positions(args) => commands::positions::execute(args, chain_id).await?,
        Commands::AddLiquidity(args) => {
            commands::add_liquidity::execute(args, dry_run, chain_id).await?
        }
        Commands::RemoveLiquidity(args) => {
            commands::remove_liquidity::execute(args, dry_run, chain_id).await?
        }
        Commands::Balance(args) => commands::balance::execute(args, chain_id).await?,
    }

    Ok(())
}

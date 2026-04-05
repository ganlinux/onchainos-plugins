mod api;
mod commands;
mod config;
mod onchainos;

use anyhow::Result;
use clap::{Parser, Subcommand};

use commands::{
    balance::BalanceArgs, positions::PositionsArgs, restake::RestakeArgs,
    unrestake::UnrestakeArgs,
};

/// Solayer Restaking Plugin — stake SOL, receive sSOL liquid restaking tokens
#[derive(Debug, Parser)]
#[command(
    name = "solayer",
    version = "0.1.0",
    about = "Solayer restaking plugin: stake SOL to receive sSOL on Solana mainnet"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Restake SOL to receive sSOL (calls Solayer Partner API + onchainos)
    Restake(RestakeArgs),

    /// Unrestake sSOL to recover SOL (5-step process, requires waiting ~2-3 days)
    Unrestake(UnrestakeArgs),

    /// Query SOL and sSOL balances for the current wallet
    Balance(BalanceArgs),

    /// Query sSOL restaking positions and their current SOL value
    Positions(PositionsArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Restake(args) => commands::restake::run(args).await,
        Commands::Unrestake(args) => commands::unrestake::run(args).await,
        Commands::Balance(args) => commands::balance::run(args).await,
        Commands::Positions(args) => commands::positions::run(args).await,
    }
}

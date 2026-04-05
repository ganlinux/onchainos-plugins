mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "stargate-v2",
    about = "Cross-chain bridge using Stargate V2 / LayerZero V2",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get a cross-chain bridge quote (fees + expected received amount)
    Quote(commands::quote::QuoteArgs),

    /// Execute a cross-chain token transfer via Stargate V2
    Send(commands::send::SendArgs),

    /// Query cross-chain transaction status via LayerZero Scan API
    Status(commands::status::StatusArgs),

    /// List supported pools, assets, and chains
    Pools(commands::pools::PoolsArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Quote(args) => commands::quote::run(args).await,
        Commands::Send(args) => commands::send::run(args).await,
        Commands::Status(args) => commands::status::run(args).await,
        Commands::Pools(args) => commands::pools::run(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

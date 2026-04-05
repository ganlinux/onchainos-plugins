mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "reservoir-protocol",
    version = "0.1.0",
    about = "Reservoir Protocol: mint rUSD stablecoin from USDC and earn yield via srUSD"
)]
struct Cli {
    /// Simulate actions without broadcasting transactions
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query rUSD/srUSD balances, exchange rate, and PSM liquidity
    Info {
        /// Wallet address to query (optional, defaults to onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
    },

    /// Mint rUSD by depositing USDC (1:1 via PSM, Ethereum mainnet only)
    Mint {
        /// Amount of USDC to deposit (e.g. "100" or "100.5")
        #[arg(long)]
        amount: String,
    },

    /// Deposit rUSD to earn yield as srUSD (~7-8% APY, Ethereum mainnet only)
    Save {
        /// Amount of rUSD to deposit (e.g. "100" or "100.0")
        #[arg(long)]
        amount: String,
    },

    /// Redeem rUSD back to USDC via PSM (1:1, Ethereum mainnet only)
    RedeemRusd {
        /// Amount of rUSD to redeem (e.g. "100" or "100.0")
        #[arg(long)]
        amount: String,
    },

    /// Redeem srUSD back to rUSD via Saving Module (includes yield, Ethereum mainnet only)
    RedeemSrusd {
        /// Amount of srUSD to redeem (e.g. "100" or "100.0")
        #[arg(long)]
        amount: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { wallet } => {
            commands::info::run(wallet).await?;
        }
        Commands::Mint { amount } => {
            commands::mint::run(&amount, cli.dry_run).await?;
        }
        Commands::Save { amount } => {
            commands::save::run(&amount, cli.dry_run).await?;
        }
        Commands::RedeemRusd { amount } => {
            commands::redeem::run_rusd(&amount, cli.dry_run).await?;
        }
        Commands::RedeemSrusd { amount } => {
            commands::redeem::run_srusd(&amount, cli.dry_run).await?;
        }
    }

    Ok(())
}

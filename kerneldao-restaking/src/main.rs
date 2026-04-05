use clap::{Parser, Subcommand};

mod commands;
mod config;
mod onchainos;
mod rpc;

#[derive(Parser)]
#[command(
    name = "kerneldao-restaking",
    about = "Restake BTC and BNB assets on KernelDAO (BSC) to earn Kernel Points"
)]
struct Cli {
    /// Simulate without broadcasting transactions
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query staked balance for one or all supported assets
    Balance {
        /// ERC-20 token address to query (omit to show all non-zero positions)
        #[arg(long)]
        asset: Option<String>,
    },

    /// Stake an ERC-20 token (approve + stake). Supports BTCB, SolvBTC, uniBTC, WBNB, etc.
    Stake {
        /// ERC-20 token address (e.g. BTCB: 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c)
        #[arg(long)]
        asset: String,

        /// Amount in human-readable units (e.g. 0.001 for 0.001 BTCB)
        #[arg(long)]
        amount: String,

        /// Referral code (default: empty string)
        #[arg(long, default_value = "")]
        referral: String,
    },

    /// Stake native BNB directly (no approve needed)
    StakeNative {
        /// Amount of BNB to stake (e.g. 0.01)
        #[arg(long)]
        amount: String,

        /// Referral code (default: empty string)
        #[arg(long, default_value = "")]
        referral: String,
    },

    /// Unstake an ERC-20 token. Initiates 7-14 day unbonding period.
    Unstake {
        /// ERC-20 token address (e.g. BTCB: 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c)
        #[arg(long)]
        asset: String,

        /// Amount in human-readable units (e.g. 0.001)
        #[arg(long)]
        amount: String,

        /// Referral code (default: empty string)
        #[arg(long, default_value = "")]
        referral: String,
    },

    /// Unstake native BNB. Initiates 7-14 day unbonding period.
    UnstakeNative {
        /// Amount of BNB to unstake (e.g. 0.01)
        #[arg(long)]
        amount: String,

        /// Referral code (default: empty string)
        #[arg(long, default_value = "")]
        referral: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Balance { asset } => {
            commands::balance::run(asset.as_deref()).await
        }
        Commands::Stake { asset, amount, referral } => {
            commands::stake::run(&asset, &amount, &referral, cli.dry_run).await
        }
        Commands::StakeNative { amount, referral } => {
            commands::stake_native::run(&amount, &referral, cli.dry_run).await
        }
        Commands::Unstake { asset, amount, referral } => {
            commands::unstake::run(&asset, &amount, &referral, cli.dry_run).await
        }
        Commands::UnstakeNative { amount, referral } => {
            commands::unstake_native::run(&amount, &referral, cli.dry_run).await
        }
    }
}

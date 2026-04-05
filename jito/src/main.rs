use clap::{Parser, Subcommand};

mod api;
mod onchainos;
mod commands;

#[derive(Parser)]
#[command(name = "jito", about = "Jito JitoSOL liquid staking and restaking on Solana")]
struct Cli {
    #[arg(long, global = true, help = "Simulate without broadcasting transactions")]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get JitoSOL pool info: APY, TVL, exchange rate, MEV rewards
    Info,
    /// Stake SOL to receive JitoSOL (via onchainos defi invest)
    Stake {
        /// Amount of SOL to stake (UI units, e.g. 0.01)
        #[arg(long)]
        amount: f64,
    },
    /// Instantly exchange JitoSOL for SOL via DEX (no waiting period)
    Unstake {
        /// Amount of JitoSOL to exchange (UI units, e.g. 0.5)
        #[arg(long)]
        amount: f64,
        /// Max slippage percentage (default: 1.0)
        #[arg(long, default_value = "1.0")]
        slippage: f64,
    },
    /// View your JitoSOL balance and current SOL value
    Positions,
    /// List available Jito Restaking vaults
    RestakeVaults,
    /// Deposit JitoSOL into a Jito Restaking Vault to receive VRT tokens
    RestakeDeposit {
        /// Vault address (base58)
        #[arg(long)]
        vault: String,
        /// Amount of JitoSOL to deposit (UI units, e.g. 0.1)
        #[arg(long)]
        amount: f64,
    },
    /// Initiate withdrawal from a Jito Restaking Vault (enqueue withdrawal)
    RestakeWithdraw {
        /// Vault address (base58)
        #[arg(long)]
        vault: String,
        /// Amount of VRT tokens to redeem (UI units)
        #[arg(long)]
        amount: f64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => commands::info::run().await,
        Commands::Stake { amount } => commands::stake::run(amount, cli.dry_run).await,
        Commands::Unstake { amount, slippage } => commands::unstake::run(amount, slippage, cli.dry_run).await,
        Commands::Positions => commands::positions::run().await,
        Commands::RestakeVaults => commands::restake_vaults::run().await,
        Commands::RestakeDeposit { vault, amount } => commands::restake_deposit::run(&vault, amount, cli.dry_run).await,
        Commands::RestakeWithdraw { vault, amount } => commands::restake_withdraw::run(&vault, amount, cli.dry_run).await,
    }
}

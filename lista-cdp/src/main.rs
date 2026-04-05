use clap::{Parser, Subcommand};

mod commands;
mod config;
mod onchainos;
mod rpc;

#[derive(Parser)]
#[command(
    name = "lista-cdp",
    about = "Lista DAO CDP on BSC — stake BNB, deposit slisBNB collateral, borrow lisUSD"
)]
struct Cli {
    #[arg(long, global = true, help = "Simulate without broadcasting transactions")]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Stake BNB to receive slisBNB via StakeManager (payable)
    Stake {
        /// BNB amount in wei (e.g. 1000000000000000000 = 1 BNB)
        #[arg(long)]
        amt: u64,
    },

    /// Request withdrawal of slisBNB from StakeManager (Step 1 of 2)
    Unstake {
        /// slisBNB amount in human-readable units (e.g. 0.5)
        #[arg(long)]
        amount: String,
    },

    /// Deposit slisBNB as CDP collateral (approve + deposit)
    CdpDeposit {
        /// slisBNB amount in human-readable units (e.g. 0.5)
        #[arg(long)]
        amount: String,
    },

    /// Borrow lisUSD against deposited slisBNB collateral
    Borrow {
        /// lisUSD amount in human-readable units (e.g. 100)
        #[arg(long)]
        amount: String,
    },

    /// Repay lisUSD debt (approve + payback)
    Repay {
        /// lisUSD amount in human-readable units (e.g. 100)
        #[arg(long)]
        amount: String,
    },

    /// Withdraw slisBNB collateral from CDP
    CdpWithdraw {
        /// slisBNB amount in human-readable units (e.g. 0.5)
        #[arg(long)]
        amount: String,
    },

    /// Query CDP position: collateral, debt, available borrow, liquidation price
    Positions {
        /// Wallet address to query (defaults to logged-in wallet)
        #[arg(long)]
        wallet: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Stake { amt } => commands::stake::run(amt, cli.dry_run).await,
        Commands::Unstake { amount } => commands::unstake::run(&amount, cli.dry_run).await,
        Commands::CdpDeposit { amount } => commands::cdp_deposit::run(&amount, cli.dry_run).await,
        Commands::Borrow { amount } => commands::borrow::run(&amount, cli.dry_run).await,
        Commands::Repay { amount } => commands::repay::run(&amount, cli.dry_run).await,
        Commands::CdpWithdraw { amount } => {
            commands::cdp_withdraw::run(&amount, cli.dry_run).await
        }
        Commands::Positions { wallet } => {
            commands::positions::run(wallet.as_deref()).await
        }
    }
}

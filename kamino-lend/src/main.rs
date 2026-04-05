mod api;
mod commands;
mod config;
mod onchainos;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "kamino-lend",
    about = "Kamino Lend CLI — supply, borrow, repay on Solana via onchainos",
    version = "0.1.0"
)]
struct Cli {
    /// Lending market public key (defaults to main market)
    #[arg(long, default_value = config::MAIN_MARKET)]
    market: String,

    /// Dry-run mode: build transactions but do not submit them
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all Kamino lending markets (off-chain)
    Markets,

    /// Show reserve metrics for a market (off-chain)
    Reserves {
        /// Market public key (defaults to --market flag)
        #[arg(long)]
        market: Option<String>,
    },

    /// Show user obligations / positions (off-chain)
    Obligations {
        /// Wallet public key (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
        /// Market public key (defaults to --market flag)
        #[arg(long)]
        market: Option<String>,
    },

    /// Deposit tokens into a Kamino reserve (on-chain)
    Deposit {
        /// Reserve public key
        #[arg(long)]
        reserve: String,
        /// Amount to deposit (in token units, e.g. "100" for 100 USDC)
        #[arg(long)]
        amount: String,
        /// Wallet public key (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
        /// Market public key (defaults to --market flag)
        #[arg(long)]
        market: Option<String>,
    },

    /// Withdraw tokens from a Kamino reserve (on-chain)
    Withdraw {
        /// Reserve public key
        #[arg(long)]
        reserve: String,
        /// Amount to withdraw (in token units)
        #[arg(long)]
        amount: String,
        /// Wallet public key (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
        /// Market public key (defaults to --market flag)
        #[arg(long)]
        market: Option<String>,
    },

    /// Borrow tokens from a Kamino reserve (on-chain)
    Borrow {
        /// Reserve public key
        #[arg(long)]
        reserve: String,
        /// Amount to borrow (in token units)
        #[arg(long)]
        amount: String,
        /// Wallet public key (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
        /// Market public key (defaults to --market flag)
        #[arg(long)]
        market: Option<String>,
    },

    /// Repay borrowed tokens to a Kamino reserve (on-chain)
    Repay {
        /// Reserve public key
        #[arg(long)]
        reserve: String,
        /// Amount to repay (in token units)
        #[arg(long)]
        amount: String,
        /// Wallet public key (defaults to logged-in onchainos wallet)
        #[arg(long)]
        wallet: Option<String>,
        /// Market public key (defaults to --market flag)
        #[arg(long)]
        market: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let client = reqwest::Client::builder()
        .user_agent("kamino-lend-cli/0.1.0")
        .build()?;

    match cli.command {
        Commands::Markets => {
            commands::markets::run(&client).await?;
        }

        Commands::Reserves { market } => {
            let mkt = market.as_deref().unwrap_or(&cli.market);
            commands::reserves::run(&client, Some(mkt)).await?;
        }

        Commands::Obligations { wallet, market } => {
            let mkt = market.as_deref().unwrap_or(&cli.market);
            commands::obligations::run(&client, wallet.as_deref(), Some(mkt)).await?;
        }

        Commands::Deposit {
            reserve,
            amount,
            wallet,
            market,
        } => {
            let mkt = market.as_deref().unwrap_or(&cli.market);
            commands::deposit::run(
                &client,
                &reserve,
                &amount,
                wallet.as_deref(),
                Some(mkt),
                cli.dry_run,
            )
            .await?;
        }

        Commands::Withdraw {
            reserve,
            amount,
            wallet,
            market,
        } => {
            let mkt = market.as_deref().unwrap_or(&cli.market);
            commands::withdraw::run(
                &client,
                &reserve,
                &amount,
                wallet.as_deref(),
                Some(mkt),
                cli.dry_run,
            )
            .await?;
        }

        Commands::Borrow {
            reserve,
            amount,
            wallet,
            market,
        } => {
            let mkt = market.as_deref().unwrap_or(&cli.market);
            commands::borrow::run(
                &client,
                &reserve,
                &amount,
                wallet.as_deref(),
                Some(mkt),
                cli.dry_run,
            )
            .await?;
        }

        Commands::Repay {
            reserve,
            amount,
            wallet,
            market,
        } => {
            let mkt = market.as_deref().unwrap_or(&cli.market);
            commands::repay::run(
                &client,
                &reserve,
                &amount,
                wallet.as_deref(),
                Some(mkt),
                cli.dry_run,
            )
            .await?;
        }
    }

    Ok(())
}

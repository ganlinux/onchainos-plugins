use clap::{Parser, Subcommand};

mod commands;
mod config;
mod onchainos;
mod rpc;

#[derive(Parser)]
#[command(
    name = "treehouse-protocol",
    about = "Treehouse Protocol — deposit ETH/WETH/stETH/wstETH for tETH yield on Ethereum, or AVAX/sAVAX for tAVAX on Avalanche"
)]
struct Cli {
    /// Simulate without broadcasting transactions (print calldata only)
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deposit ETH/WETH/stETH/wstETH to get tETH (Ethereum) or AVAX/sAVAX to get tAVAX (Avalanche)
    Deposit {
        /// Chain ID: 1 (Ethereum, default) or 43114 (Avalanche)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Token to deposit: ETH, WETH, stETH, wstETH (Ethereum) or AVAX, sAVAX (Avalanche)
        #[arg(long)]
        token: String,

        /// Amount to deposit (human-readable, e.g. "1.0" for 1 ETH)
        #[arg(long)]
        amount: String,

        /// Sender wallet address (optional; resolved from onchainos if omitted)
        #[arg(long)]
        from: Option<String>,
    },

    /// Query tETH (Ethereum) or tAVAX (Avalanche) balance and underlying value
    Balance {
        /// Chain ID: 1 (Ethereum, default) or 43114 (Avalanche)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Wallet address to query (optional; resolved from onchainos if omitted)
        #[arg(long)]
        account: Option<String>,
    },

    /// Get current tETH or tAVAX price (convertToAssets ratio)
    Price {
        /// Chain ID: 1 (Ethereum, default) or 43114 (Avalanche)
        #[arg(long, default_value = "1")]
        chain: u64,
    },

    /// Show full position: balance, underlying value, and APY from DeFiLlama
    Positions {
        /// Chain ID: 1 (Ethereum, default) or 43114 (Avalanche)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Wallet address to query (optional; resolved from onchainos if omitted)
        #[arg(long)]
        account: Option<String>,
    },

    /// Withdraw tETH → wstETH via Curve pool (Ethereum only; small amounts <= 200 wstETH)
    Withdraw {
        /// Chain ID: must be 1 (Ethereum); tAVAX withdrawal not supported
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Amount of tETH to redeem (e.g. "10.0")
        #[arg(long)]
        amount: String,

        /// Slippage tolerance in basis points (default: 100 = 1%)
        #[arg(long, default_value = "100")]
        slippage_bps: u32,

        /// Sender wallet address (optional; resolved from onchainos if omitted)
        #[arg(long)]
        from: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deposit { chain, token, amount, from } => {
            commands::deposit::run(chain, &token, &amount, from.as_deref(), cli.dry_run).await
        }
        Commands::Balance { chain, account } => {
            commands::balance::run(chain, account.as_deref()).await
        }
        Commands::Price { chain } => {
            commands::price::run(chain).await
        }
        Commands::Positions { chain, account } => {
            commands::positions::run(chain, account.as_deref()).await
        }
        Commands::Withdraw { chain, amount, slippage_bps, from } => {
            commands::withdraw::run(chain, &amount, slippage_bps, from.as_deref(), cli.dry_run)
                .await
        }
    }
}

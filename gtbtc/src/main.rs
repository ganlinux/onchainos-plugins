use clap::{Parser, Subcommand};

mod api;
mod commands;
mod config;
mod onchainos;
mod rpc;

#[derive(Parser)]
#[command(
    name = "gtbtc",
    about = "Gate Wrapped BTC (GTBTC) plugin — balance, price, APR, transfer, approve"
)]
struct Cli {
    /// Chain ID: 1=Ethereum, 56=BSC, 8453=Base, 501=Solana (default: 1)
    #[arg(long, global = true, default_value = "1")]
    chain: u64,

    /// Simulate without broadcasting on-chain transactions
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Query GTBTC balance for an address (EVM or Solana)
    Balance {
        /// Wallet address (EVM 0x... or Solana base58). Defaults to current onchainos wallet.
        #[arg(long)]
        address: Option<String>,
    },
    /// Get current GTBTC price from Gate.io
    Price,
    /// Get current GTBTC staking APR from Gate.io
    Apr,
    /// Transfer GTBTC to another address (EVM only in v1)
    Transfer {
        /// Recipient address (EVM 0x...)
        #[arg(long)]
        to: String,
        /// Amount in GTBTC (e.g. 0.001)
        #[arg(long)]
        amount: f64,
        /// Sender address (defaults to current onchainos wallet)
        #[arg(long)]
        from: Option<String>,
    },
    /// Approve a spender to use your GTBTC (EVM only)
    Approve {
        /// Spender address (e.g. DEX router 0x...)
        #[arg(long)]
        spender: String,
        /// Amount in GTBTC to approve (omit for unlimited)
        #[arg(long)]
        amount: Option<f64>,
        /// Owner address (defaults to current onchainos wallet)
        #[arg(long)]
        from: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Balance { address } => {
            commands::balance::run(&commands::balance::BalanceArgs {
                address,
                chain_id: cli.chain,
            })
            .await
        }
        Commands::Price => commands::price::run().await,
        Commands::Apr => commands::apr::run().await,
        Commands::Transfer { to, amount, from } => {
            commands::transfer::run(
                &commands::transfer::TransferArgs {
                    to,
                    amount,
                    from,
                    chain_id: cli.chain,
                },
                cli.dry_run,
            )
            .await
        }
        Commands::Approve {
            spender,
            amount,
            from,
        } => {
            commands::approve::run(
                &commands::approve::ApproveArgs {
                    spender,
                    amount,
                    from,
                    chain_id: cli.chain,
                },
                cli.dry_run,
            )
            .await
        }
    }
}

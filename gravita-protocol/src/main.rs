use clap::{Parser, Subcommand};

mod commands;
mod config;
mod onchainos;
mod rpc;

#[derive(Parser)]
#[command(
    name = "gravita-protocol",
    about = "Gravita Protocol — borrow GRAI stablecoin against LST collateral (wstETH, rETH, WETH)"
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
    /// Open a new Vessel: deposit collateral and borrow GRAI
    Open {
        /// Chain ID: 1 (Ethereum) or 59144 (Linea)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Collateral symbol: WETH, wstETH, rETH (Ethereum); wstETH (Linea)
        #[arg(long)]
        collateral: String,

        /// Amount of collateral to deposit (e.g. "1.0" for 1 wstETH)
        #[arg(long)]
        coll_amount: String,

        /// Amount of GRAI to borrow (minimum ~2000 GRAI, e.g. "2000.0")
        #[arg(long)]
        debt_amount: String,
    },

    /// Adjust an existing Vessel: add/withdraw collateral or borrow/repay GRAI
    Adjust {
        /// Chain ID: 1 (Ethereum) or 59144 (Linea)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Collateral symbol: WETH, wstETH, rETH
        #[arg(long)]
        collateral: String,

        /// Action: add-coll, withdraw-coll, borrow, repay
        #[arg(long)]
        action: String,

        /// Amount (collateral units for add-coll/withdraw-coll; GRAI for borrow/repay)
        #[arg(long)]
        amount: String,
    },

    /// Close a Vessel: repay all GRAI debt and recover collateral
    Close {
        /// Chain ID: 1 (Ethereum) or 59144 (Linea)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Collateral symbol: WETH, wstETH, rETH
        #[arg(long)]
        collateral: String,
    },

    /// Query Vessel position: debt, collateral, status, and risk parameters
    Position {
        /// Chain ID: 1 (Ethereum) or 59144 (Linea)
        #[arg(long, default_value = "1")]
        chain: u64,

        /// Collateral symbol: WETH, wstETH, rETH
        #[arg(long)]
        collateral: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Open { chain, collateral, coll_amount, debt_amount } => {
            commands::open::run(chain, &collateral, &coll_amount, &debt_amount, cli.dry_run).await
        }
        Commands::Adjust { chain, collateral, action, amount } => {
            let action_parsed: commands::adjust::AdjustAction = action.parse()?;
            commands::adjust::run(chain, &collateral, action_parsed, &amount, cli.dry_run).await
        }
        Commands::Close { chain, collateral } => {
            commands::close::run(chain, &collateral, cli.dry_run).await
        }
        Commands::Position { chain, collateral } => {
            commands::position::run(chain, &collateral).await
        }
    }
}

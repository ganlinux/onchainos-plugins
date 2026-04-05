mod commands;
mod config;
mod onchainos;
mod rpc;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "swellchain-staking",
    about = "Swell Network staking: swETH liquid staking, rswETH restaking, and Earn pool on Ethereum L1",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Stake ETH to receive swETH (liquid staking token)
    Stake(commands::stake::StakeArgs),

    /// Deposit swETH or rswETH into SimpleStakingERC20 Earn pool
    EarnDeposit(commands::earn_deposit::EarnDepositArgs),

    /// Withdraw swETH or rswETH from SimpleStakingERC20 Earn pool
    EarnWithdraw(commands::earn_withdraw::EarnWithdrawArgs),

    /// Create a swETH withdrawal request (swETH -> swEXIT NFT, redeemed for ETH after 1-12 days)
    RequestWithdrawal(commands::request_withdrawal::RequestWithdrawalArgs),

    /// Finalize a matured swEXIT withdrawal and receive ETH
    FinalizeWithdrawal(commands::finalize_withdrawal::FinalizeWithdrawalArgs),

    /// Query swETH and rswETH balances and exchange rates for an address
    Balance(commands::balance::BalanceArgs),

    /// Show all staking positions and pending withdrawals for an address
    Positions(commands::positions::PositionsArgs),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Stake(args) => commands::stake::run(args).await,
        Commands::EarnDeposit(args) => commands::earn_deposit::run(args).await,
        Commands::EarnWithdraw(args) => commands::earn_withdraw::run(args).await,
        Commands::RequestWithdrawal(args) => commands::request_withdrawal::run(args).await,
        Commands::FinalizeWithdrawal(args) => commands::finalize_withdrawal::run(args).await,
        Commands::Balance(args) => commands::balance::run(args).await,
        Commands::Positions(args) => commands::positions::run(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

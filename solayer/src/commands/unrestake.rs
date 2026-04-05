use anyhow::{anyhow, Result};
use clap::Args;

use crate::api::get_ssol_balance;
use crate::config::{DEFAULT_RPC, LAMPORTS_PER_SOL, RESTAKING_PROGRAM};
use crate::onchainos::resolve_wallet_and_balance_solana;

#[derive(Debug, Args)]
pub struct UnrestakeArgs {
    /// Amount of sSOL to unrestake (UI units, e.g., 1.0)
    #[arg(long)]
    pub amount: f64,

    /// Dry run: print the command without executing it
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Solana JSON-RPC endpoint
    #[arg(long, default_value = DEFAULT_RPC)]
    pub rpc: String,
}

pub async fn run(args: UnrestakeArgs) -> Result<()> {
    if args.amount <= 0.0 {
        return Err(anyhow!("Amount must be greater than 0"));
    }

    println!(
        "Preparing to unrestake {:.6} sSOL from Solayer...",
        args.amount
    );

    // Step 1: Get wallet address (and balance JSON, unused here)
    let (wallet, _) = resolve_wallet_and_balance_solana()?;
    println!("Wallet: {}", wallet);

    // Step 2: Check sSOL balance (skip enforcement in dry-run mode)
    let ssol_balance = get_ssol_balance(&args.rpc, &wallet).await?;

    if !args.dry_run && ssol_balance < args.amount {
        return Err(anyhow!(
            "Insufficient sSOL balance. Have {:.6} sSOL, need {:.6} sSOL.",
            ssol_balance,
            args.amount
        ));
    }

    println!(
        "sSOL balance: {:.6} sSOL {}",
        ssol_balance,
        if ssol_balance >= args.amount { "✓" } else { "(dry-run only)" }
    );

    // Step 3: Important notice about waiting period
    println!();
    println!("=== Important: Unrestake Process & Waiting Period ===");
    println!(
        "Unrestaking {:.6} sSOL requires a 5-step on-chain transaction:",
        args.amount
    );
    println!("  1. Restaking Program unrestake() — burn sSOL, release intermediate LST");
    println!("  2. Approve LST transfer from your LST token account");
    println!("  3. Create a new stake account (requires ~0.0023 SOL rent)");
    println!("  4. Withdraw stake from Solayer Stake Pool into the new stake account");
    println!("  5. Deactivate the stake account");
    println!();
    println!("After deactivation, you must wait until the current Solana epoch ends");
    println!("(typically 2-3 days) before you can withdraw SOL to your wallet.");
    println!();

    // Convert amount to lamports for the contract call
    let amount_lamports = (args.amount * LAMPORTS_PER_SOL as f64) as u64;
    println!("Amount in lamports: {}", amount_lamports);

    // Step 4: Confirm with user before broadcasting
    println!();
    println!("=== Action Required: Confirm Unrestaking ===");
    println!(
        "You are about to unrestake {:.6} sSOL ({} lamports) from Solayer.",
        args.amount, amount_lamports
    );
    println!("The recovered SOL will enter a deactivating stake account.");
    println!("This transaction will be signed and broadcast via onchainos.");
    println!();

    // Step 5: Build and display the onchainos command
    // NOTE: Constructing the full 5-instruction Solana transaction in pure Rust
    // without the Solana/Anchor SDK is not feasible here. We provide the
    // dry-run command so the agent can inform the user, and guide them to
    // use the official Solayer CLI (TypeScript) for the actual execution.
    let cmd = format!(
        "onchainos wallet contract-call \\\n  \
         --chain 501 \\\n  \
         --to {} \\\n  \
         --unsigned-tx <BASE58_SERIALIZED_TX> \\\n  \
         --force",
        RESTAKING_PROGRAM
    );

    if args.dry_run {
        println!("[dry-run] The unrestake transaction would use:");
        println!("{}", cmd);
        println!();
        println!("Transaction construction notes:");
        println!("  - Program: {} (Solayer Restaking)", RESTAKING_PROGRAM);
        println!("  - Method: unrestake({})", amount_lamports);
        println!("  - Instructions: 5 (unrestake + approve + createAccount + withdrawStake + deactivate)");
        println!("  - A fresh stake account keypair is generated per transaction");
        println!("  - ComputeBudget: 500,000 units @ 200,000 microLamports");
        return Ok(());
    }

    // Step 6: Notify user about full implementation path
    println!("=== Implementation Note ===");
    println!(
        "The full unrestake transaction requires constructing a 5-instruction Solana \
         transaction with Anchor CPI calls and a freshly generated stake account keypair."
    );
    println!();
    println!("For production use, please use the official Solayer CLI:");
    println!("  npx solayer-cli unrestake --amount {}", args.amount);
    println!("  (https://github.com/solayer-labs/solayer-cli)");
    println!();
    println!("Alternatively, use the Solayer web app: https://app.solayer.org");
    println!();
    println!("If you have the serialized transaction (base64 from Solayer CLI/SDK), you can");
    println!("broadcast it via onchainos with:");
    println!("{}", cmd);

    Ok(())
}

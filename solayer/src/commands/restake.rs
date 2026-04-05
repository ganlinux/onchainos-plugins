use anyhow::{anyhow, Result};
use clap::Args;

use crate::api::{parse_sol_balance, restake_ssol};
use crate::config::{DEFAULT_RPC, MIN_GAS_BUFFER_SOL, RESTAKING_PROGRAM};
use crate::onchainos::{
    extract_tx_hash, resolve_wallet_and_balance_solana, wallet_contract_call_solana,
};

#[derive(Debug, Args)]
pub struct RestakeArgs {
    /// Amount of SOL to restake (UI units, e.g., 1.0)
    #[arg(long)]
    pub amount: f64,

    /// Partner referrer wallet address (optional, for tracking)
    #[arg(long)]
    pub referrer: Option<String>,

    /// Dry run: print the command without executing it
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Solana JSON-RPC endpoint
    #[arg(long, default_value = DEFAULT_RPC)]
    pub rpc: String,
}

pub async fn run(args: RestakeArgs) -> Result<()> {
    if args.amount <= 0.0 {
        return Err(anyhow!("Amount must be greater than 0"));
    }

    println!(
        "Preparing to restake {:.4} SOL on Solayer...",
        args.amount
    );

    // Step 1: Get wallet address AND SOL balance in a single onchainos call
    let (wallet, sol_output) = resolve_wallet_and_balance_solana()?;
    println!("Wallet: {}", wallet);

    // Step 2: Parse SOL balance
    let sol_balance = parse_sol_balance(&sol_output);
    let required = args.amount + MIN_GAS_BUFFER_SOL;

    if !args.dry_run && sol_balance < required {
        return Err(anyhow!(
            "Insufficient SOL balance. Have {:.4} SOL, need at least {:.4} SOL ({:.4} SOL to restake + {:.4} SOL for gas).",
            sol_balance,
            required,
            args.amount,
            MIN_GAS_BUFFER_SOL
        ));
    }

    println!("SOL balance: {:.4} SOL {}", sol_balance, if sol_balance >= required { "✓" } else { "(dry-run only)" });

    // In dry-run mode: skip API call and just show what would happen
    if args.dry_run {
        println!();
        println!("[dry-run] Restake simulation — no API call or transaction will be made.");
        println!("dry_run: true");
        println!("action: restake");
        println!("amount_sol: {}", args.amount);
        println!("staker: {}", wallet);
        println!("program: {}", RESTAKING_PROGRAM);
        println!();
        println!("Steps that would execute:");
        println!("  1. GET https://app.solayer.org/api/partner/restake/ssol?staker={}&amount={}&referrerkey=<PARTNER>", wallet, args.amount);
        println!("  2. Convert API response base64 tx → base58");
        println!("  3. onchainos wallet contract-call --chain 501 --to {} --unsigned-tx <BASE58_TX> --force", RESTAKING_PROGRAM);
        println!();
        println!("[dry-run] serialized_tx: <would be populated from API response>");
        return Ok(());
    }

    // Step 3: Call Partner Restake API to get unsigned transaction
    println!("Calling Solayer Partner Restake API...");
    let amount_str = format!("{}", args.amount);
    // referrerkey is required by the API; fall back to staker's own wallet if not provided
    let referrer_owned;
    let referrer = match args.referrer.as_deref() {
        Some(r) if !r.is_empty() => Some(r),
        _ => {
            referrer_owned = wallet.clone();
            Some(referrer_owned.as_str())
        }
    };

    let api_resp = restake_ssol(&args.rpc, &wallet, &amount_str, referrer).await?;

    if let Some(ref msg) = api_resp.message {
        println!("API message: {}", msg);
    }

    // Step 4: Confirm with user before broadcasting
    println!();
    println!("=== Action Required: Confirm Restaking ===");
    println!(
        "You are about to restake {:.4} SOL to Solayer in exchange for sSOL.",
        args.amount
    );
    if let Some(ref msg) = api_resp.message {
        println!("Expected outcome: {}", msg);
    }
    println!("This transaction will be signed and broadcast via onchainos.");
    println!();

    // Step 5: Execute via onchainos (base64 tx from API → base58 for onchainos)
    println!("Broadcasting transaction via onchainos...");
    let result =
        wallet_contract_call_solana(RESTAKING_PROGRAM, &api_resp.transaction, false)?;

    // Step 6: Extract and display txHash
    match extract_tx_hash(&result) {
        Some(hash) => {
            println!();
            println!("=== Restaking Successful ===");
            println!("Transaction hash: {}", hash);
            println!(
                "Explorer: https://solscan.io/tx/{}",
                hash
            );
            println!();
            println!(
                "Your sSOL will appear in your wallet shortly. sSOL holders accumulate \
                 Solayer points and can delegate to AVS for additional yield."
            );
        }
        None => {
            println!("Transaction submitted. Raw output: {}", result);
        }
    }

    Ok(())
}

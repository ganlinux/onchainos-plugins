use crate::onchainos;
use anyhow::{anyhow, Result};

pub async fn run(amount_jitosol: f64, slippage: f64, dry_run: bool) -> Result<()> {
    if amount_jitosol <= 0.0 {
        return Err(anyhow!("Amount must be greater than 0"));
    }
    if slippage < 0.0 || slippage > 50.0 {
        return Err(anyhow!("Slippage must be between 0 and 50 (percent)"));
    }

    let wallet = onchainos::resolve_wallet_solana()?;
    println!("Wallet: {}", wallet);
    println!("Exchanging {} JitoSOL for SOL via DEX (instant)...", amount_jitosol);
    println!("Max slippage: {}%", slippage);

    if dry_run {
        println!("[dry-run] Would execute swap: {} JitoSOL -> SOL (slippage: {}%)", amount_jitosol, slippage);
        println!("[dry-run] Command would be:");
        println!("  onchainos swap execute --chain solana \\");
        println!("    --from J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn \\");
        println!("    --to So11111111111111111111111111111111111111112 \\");
        println!("    --readable-amount {} --wallet {} --slippage {}", amount_jitosol, wallet, slippage);
        return Ok(());
    }

    let result = onchainos::swap_execute_jitosol_to_sol(amount_jitosol, &wallet, slippage, false)?;
    let tx_hash = onchainos::extract_tx_hash(&result);
    println!("Unstake (instant swap) successful!");
    println!("txHash: {}", tx_hash);
    Ok(())
}

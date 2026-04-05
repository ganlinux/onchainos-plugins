use crate::onchainos;
use anyhow::{anyhow, Result};

const SPL_STAKE_POOL_PROGRAM: &str = "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy";

pub async fn run(amount_sol: f64, dry_run: bool) -> Result<()> {
    // Convert SOL to lamports (integer)
    let lamports = (amount_sol * 1_000_000_000.0) as u64;
    if lamports < 1_000_000 {
        return Err(anyhow!("Minimum stake amount is 0.001 SOL (got {:.6} SOL)", amount_sol));
    }

    // Get wallet address
    let wallet = onchainos::resolve_wallet_solana()?;
    println!("Wallet: {}", wallet);
    println!("Staking {} SOL ({} lamports) for JitoSOL...", amount_sol, lamports);

    // Get serialized tx from onchainos defi invest
    let invest_result = onchainos::defi_invest_jito(&wallet, lamports)?;
    let serialized_tx = invest_result["data"]["dataList"][0]["serializedData"]
        .as_str()
        .ok_or_else(|| anyhow!(
            "No serializedData in defi invest response.\nFull response: {}",
            serde_json::to_string_pretty(&invest_result).unwrap_or_default()
        ))?;

    let preview_len = serialized_tx.len().min(20);
    println!("Got serialized tx ({}...)", &serialized_tx[..preview_len]);

    if dry_run {
        println!("[dry-run] Would submit tx to {}", SPL_STAKE_POOL_PROGRAM);
        println!("[dry-run] serializedData length: {} chars", serialized_tx.len());
        println!("[dry-run] Command would be:");
        println!("  onchainos wallet contract-call --chain 501 --to {} --unsigned-tx {} --force",
            SPL_STAKE_POOL_PROGRAM, &serialized_tx[..preview_len]);
        return Ok(());
    }

    // Submit tx immediately (60s expiry window)
    println!("Broadcasting transaction...");
    let result = onchainos::wallet_contract_call_solana(SPL_STAKE_POOL_PROGRAM, serialized_tx, false)?;
    let tx_hash = onchainos::extract_tx_hash(&result);
    println!("Stake successful!");
    println!("txHash: {}", tx_hash);
    println!("You will receive JitoSOL in your wallet shortly.");
    Ok(())
}

/// closeVessel: approve GRAI + closeVessel(asset)
/// closeVessel selector: 0xe687854f
/// ABI: (address _asset)

use crate::config::{format_18, get_chain, resolve_collateral};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{get_vessel_debt, get_vessel_status};

/// Encode closeVessel calldata.
/// closeVessel(address _asset) — selector 0xe687854f
fn encode_close_vessel(asset: &str) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    let asset_padded = format!("{:0>64}", asset_stripped);
    format!("0xe687854f{}", asset_padded)
}

pub async fn run(
    chain_id: u64,
    collateral: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = get_chain(chain_id)?;
    let asset = resolve_collateral(chain_id, collateral)?;

    println!("=== Gravita Protocol — Close Vessel ===");
    println!("Chain:      {} ({})", chain_id, if chain_id == 1 { "Ethereum" } else { "Linea" });
    println!("Collateral: {} ({})", collateral, asset);
    println!();

    // Resolve wallet
    let wallet = resolve_wallet(chain_id)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on chain {}. Run: onchainos wallet login", chain_id);
    }
    println!("Wallet: {}", wallet);

    // Check vessel is active (skip status check in dry-run mode)
    let debt = if dry_run {
        // In dry-run mode, use a placeholder debt to show calldata
        let status = get_vessel_status(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await.unwrap_or(0);
        if status == 1 {
            get_vessel_debt(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await.unwrap_or(0)
        } else {
            // Placeholder: 2000 GRAI + 200 gas compensation
            2200_u128 * 1_000_000_000_000_000_000u128
        }
    } else {
        let status = get_vessel_status(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await?;
        if status != 1 {
            anyhow::bail!(
                "No active Vessel found for {} on chain {} (status={}). Nothing to close.",
                collateral, chain_id, status
            );
        }
        get_vessel_debt(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await?
    };
    println!("Current Vessel debt: {} GRAI (includes 200 GRAI gas compensation)", format_18(debt));
    println!("You must hold at least {} GRAI to close this Vessel.", format_18(debt));
    println!("GRAI token on chain {}: {}", chain_id, cfg.grai_token);
    println!();

    println!("--- Step 1: Approve GRAI -> BorrowerOperations ---");
    println!("  GRAI token:  {}", cfg.grai_token);
    println!("  Spender:     {}", cfg.borrower_operations);
    println!("  Amount:      {} GRAI", format_18(debt));
    println!();
    println!(">>> Please confirm Step 1 (approve GRAI for BorrowerOperations to repay full debt of {} GRAI). Proceed? [y/N]", format_18(debt));
    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let approve_result = erc20_approve(chain_id, cfg.grai_token, cfg.borrower_operations, debt, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);
    if !dry_run {
        println!("Waiting 5 seconds before closeVessel to avoid nonce conflict...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    println!();
    println!("--- Step 2: closeVessel ---");
    println!("  BorrowerOperations: {}", cfg.borrower_operations);
    println!("  Asset: {} ({})", collateral, asset);
    println!();
    println!(">>> Please confirm Step 2 (closeVessel — this will repay {} GRAI and return all {} collateral). Proceed? [y/N]", format_18(debt), collateral);
    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let calldata = encode_close_vessel(asset);
    let result = wallet_contract_call(chain_id, cfg.borrower_operations, &calldata, Some(&wallet), None, dry_run).await?;
    let tx_hash = extract_tx_hash(&result);
    println!("closeVessel tx: {}", tx_hash);

    if dry_run {
        println!();
        println!("[dry-run] closeVessel calldata: {}", calldata);
    } else {
        println!();
        println!("Vessel closed successfully! Collateral returned to your wallet.");
    }

    Ok(())
}

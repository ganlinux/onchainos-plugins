use crate::config::{format_18, get_chain, resolve_collateral, ETHEREUM_COLLATERALS, LINEA_COLLATERALS};
use crate::onchainos::resolve_wallet;
use crate::rpc::{get_entire_debt_and_coll, get_vessel_status, get_mcr, get_borrowing_fee, status_str};

pub async fn run(chain_id: u64, collateral: &str) -> anyhow::Result<()> {
    let cfg = get_chain(chain_id)?;
    let asset = resolve_collateral(chain_id, collateral)?;

    // Resolve wallet address
    let wallet = resolve_wallet(chain_id)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on chain {}. Run: onchainos wallet login", chain_id);
    }

    println!("Querying Vessel position on chain {} for wallet {}", chain_id, wallet);
    println!("Collateral: {} ({})", collateral, asset);
    println!();

    // Get vessel status
    let status = get_vessel_status(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await?;
    println!("Vessel Status: {} ({})", status, status_str(status));

    if status == 0 {
        println!("No active Vessel found for {} on chain {}.", collateral, chain_id);
        println!("Use 'gravita-protocol open' to create a new Vessel.");
        return Ok(());
    }

    // Get debt and collateral
    let (debt, coll, pending_debt, pending_coll) =
        get_entire_debt_and_coll(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await?;

    println!("Collateral locked: {} {} (+ pending: {})", format_18(coll), collateral, format_18(pending_coll));
    println!("Debt (GRAI):       {} (+ pending: {})", format_18(debt), format_18(pending_debt));

    // Calculate ICR
    if coll > 0 && debt > 0 {
        // ICR = coll / debt (both in the same denomination makes sense only with price)
        // Without price oracle, show ratio of coll/debt as a dimensionless ratio
        // We display for reference; full ICR needs price
        println!();
        println!("Note: ICR calculation requires collateral price. Query a price oracle for full risk assessment.");
    }

    // Get MCR and borrowing fee
    if let Ok(mcr) = get_mcr(cfg.rpc_url, cfg.admin_contract, asset).await {
        let mcr_pct = (mcr as f64) / 1e16; // convert 1e18 to percentage
        println!("MCR (min collateral ratio): {:.1}%", mcr_pct);
    }

    if let Ok(fee) = get_borrowing_fee(cfg.rpc_url, cfg.admin_contract, asset).await {
        let fee_pct = (fee as f64) / 1e16;
        println!("One-time borrowing fee: {:.2}%", fee_pct);
    }

    println!();
    println!("GRAI token (chain {}): {}", chain_id, cfg.grai_token);

    // Show available collaterals on this chain
    let collaterals = match chain_id {
        1     => ETHEREUM_COLLATERALS,
        59144 => LINEA_COLLATERALS,
        _     => &[],
    };
    println!();
    println!("Supported collaterals on chain {}:", chain_id);
    for c in collaterals {
        println!("  {} ({}) — max LTV: {:.0}%", c.symbol, c.address, c.max_ltv * 100.0);
    }

    Ok(())
}

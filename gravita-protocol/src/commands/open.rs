/// openVessel: approve collateral + openVessel(asset, assetAmount, debtAmount, 0x0, 0x0)
/// Selector: 0xd92ff442
/// ABI: (address _asset, uint256 _assetAmount, uint256 _debtTokenAmount, address _upperHint, address _lowerHint)

use crate::config::{format_18, get_chain, parse_18, resolve_collateral};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{get_min_net_debt, get_vessel_status};

const ZERO_ADDRESS: &str = "0000000000000000000000000000000000000000";

/// Encode openVessel calldata.
/// openVessel(address _asset, uint256 _assetAmount, uint256 _debtTokenAmount, address _upperHint, address _lowerHint)
/// selector: 0xd92ff442
fn encode_open_vessel(asset: &str, asset_amount: u128, debt_amount: u128) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    let asset_padded   = format!("{:0>64}", asset_stripped);
    let amount_hex     = format!("{:064x}", asset_amount);
    let debt_hex       = format!("{:064x}", debt_amount);
    let upper_hint     = format!("{:0>64}", ZERO_ADDRESS);
    let lower_hint     = format!("{:0>64}", ZERO_ADDRESS);
    format!("0xd92ff442{}{}{}{}{}", asset_padded, amount_hex, debt_hex, upper_hint, lower_hint)
}

pub async fn run(
    chain_id: u64,
    collateral: &str,
    coll_amount: &str,
    debt_amount: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = get_chain(chain_id)?;
    let asset = resolve_collateral(chain_id, collateral)?;

    let coll_wei: u128 = parse_18(coll_amount)?;
    let debt_wei: u128 = parse_18(debt_amount)?;

    println!("=== Gravita Protocol — Open Vessel ===");
    println!("Chain:      {} ({})", chain_id, if chain_id == 1 { "Ethereum" } else { "Linea" });
    println!("Collateral: {} ({}) = {} tokens", collateral, asset, format_18(coll_wei));
    println!("Borrow:     {} GRAI", format_18(debt_wei));
    println!();

    // Resolve wallet
    let wallet = resolve_wallet(chain_id)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on chain {}. Run: onchainos wallet login", chain_id);
    }
    println!("Wallet: {}", wallet);

    // Check vessel not already active
    let status = get_vessel_status(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await?;
    if status == 1 {
        anyhow::bail!(
            "Vessel already active for {} on chain {}. Use 'adjust' to modify the existing Vessel.",
            collateral, chain_id
        );
    }

    // Check min net debt
    let min_debt = get_min_net_debt(cfg.rpc_url, cfg.admin_contract, asset).await.unwrap_or(0);
    if min_debt > 0 && debt_wei < min_debt {
        anyhow::bail!(
            "Debt amount {} GRAI is below minimum {} GRAI for {} on chain {}.",
            format_18(debt_wei), format_18(min_debt), collateral, chain_id
        );
    }

    println!();
    println!("--- Step 1: Approve {} -> BorrowerOperations ---", collateral);
    println!("  Token:   {}", asset);
    println!("  Spender: {}", cfg.borrower_operations);
    println!("  Amount:  {} {}", format_18(coll_wei), collateral);
    println!();
    println!(">>> Please confirm Step 1 (ERC-20 approve {} for BorrowerOperations). Proceed? [y/N]", collateral);
    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let approve_result = erc20_approve(chain_id, asset, cfg.borrower_operations, coll_wei, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);
    if !dry_run {
        println!("Waiting 5 seconds before openVessel to avoid nonce conflict...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    println!();
    println!("--- Step 2: openVessel ---");
    println!("  BorrowerOperations: {}", cfg.borrower_operations);
    println!("  Asset:        {} ({})", collateral, asset);
    println!("  Coll amount:  {} {}", format_18(coll_wei), collateral);
    println!("  Debt amount:  {} GRAI", format_18(debt_wei));
    println!("  Hints:        address(0) / address(0)");
    println!();
    println!(">>> Please confirm Step 2 (openVessel on Gravita Protocol). This will lock {} {} and mint {} GRAI. Proceed? [y/N]", format_18(coll_wei), collateral, format_18(debt_wei));
    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let calldata = encode_open_vessel(asset, coll_wei, debt_wei);
    let result = wallet_contract_call(chain_id, cfg.borrower_operations, &calldata, Some(&wallet), None, dry_run).await?;
    let tx_hash = extract_tx_hash(&result);
    println!("openVessel tx: {}", tx_hash);

    if dry_run {
        println!();
        println!("[dry-run] openVessel calldata: {}", calldata);
    } else {
        println!();
        println!("Vessel opened successfully!");
        println!("Note: Total debt includes 200 GRAI gas compensation locked by the protocol.");
    }

    Ok(())
}

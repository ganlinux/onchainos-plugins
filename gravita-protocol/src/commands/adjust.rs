/// Vessel adjustment operations:
///   add-coll:          addColl(asset, assetSent, upperHint, lowerHint)        selector 0x48a4a39d
///   withdraw-coll:     withdrawColl(asset, collWithdrawal, upperHint, lowerHint) selector 0x49b010c5
///   borrow:            withdrawDebtTokens(asset, debtAmount, upperHint, lowerHint) selector 0xb5c5c9fc
///   repay:             repayDebtTokens(asset, debtAmount, upperHint, lowerHint) selector 0x7703d730

use crate::config::{format_18, get_chain, parse_18, resolve_collateral};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::get_vessel_status;

const ZERO_ADDRESS: &str = "0000000000000000000000000000000000000000";

fn encode_add_coll(asset: &str, amount: u128) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    format!(
        "0x48a4a39d{:0>64}{:064x}{:0>64}{:0>64}",
        asset_stripped, amount, ZERO_ADDRESS, ZERO_ADDRESS
    )
}

fn encode_withdraw_coll(asset: &str, amount: u128) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    format!(
        "0x49b010c5{:0>64}{:064x}{:0>64}{:0>64}",
        asset_stripped, amount, ZERO_ADDRESS, ZERO_ADDRESS
    )
}

fn encode_withdraw_debt(asset: &str, amount: u128) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    format!(
        "0xb5c5c9fc{:0>64}{:064x}{:0>64}{:0>64}",
        asset_stripped, amount, ZERO_ADDRESS, ZERO_ADDRESS
    )
}

fn encode_repay_debt(asset: &str, amount: u128) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    format!(
        "0x7703d730{:0>64}{:064x}{:0>64}{:0>64}",
        asset_stripped, amount, ZERO_ADDRESS, ZERO_ADDRESS
    )
}

#[derive(Debug, Clone, PartialEq)]
pub enum AdjustAction {
    AddColl,
    WithdrawColl,
    Borrow,
    Repay,
}

impl std::str::FromStr for AdjustAction {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "add-coll" | "addcoll" | "add_coll" => Ok(AdjustAction::AddColl),
            "withdraw-coll" | "withdrawcoll" | "withdraw_coll" => Ok(AdjustAction::WithdrawColl),
            "borrow" | "withdraw-debt" => Ok(AdjustAction::Borrow),
            "repay" | "repay-debt" => Ok(AdjustAction::Repay),
            _ => anyhow::bail!(
                "Unknown action '{}'. Valid: add-coll, withdraw-coll, borrow, repay",
                s
            ),
        }
    }
}

pub async fn run(
    chain_id: u64,
    collateral: &str,
    action: AdjustAction,
    amount: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    let cfg = get_chain(chain_id)?;
    let asset = resolve_collateral(chain_id, collateral)?;
    let amount_wei: u128 = parse_18(amount)?;

    let action_label = match action {
        AdjustAction::AddColl      => "Add Collateral",
        AdjustAction::WithdrawColl => "Withdraw Collateral",
        AdjustAction::Borrow       => "Borrow More GRAI",
        AdjustAction::Repay        => "Repay GRAI",
    };

    println!("=== Gravita Protocol — Adjust Vessel: {} ===", action_label);
    println!("Chain:      {} ({})", chain_id, if chain_id == 1 { "Ethereum" } else { "Linea" });
    println!("Collateral: {} ({})", collateral, asset);
    println!("Amount:     {}", format_18(amount_wei));
    println!();

    let wallet = resolve_wallet(chain_id)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on chain {}. Run: onchainos wallet login", chain_id);
    }
    println!("Wallet: {}", wallet);

    // Check vessel is active
    let status = get_vessel_status(cfg.rpc_url, cfg.vessel_manager, asset, &wallet).await?;
    if status != 1 {
        anyhow::bail!(
            "No active Vessel found for {} on chain {} (status={}). Use 'open' to create a Vessel first.",
            collateral, chain_id, status
        );
    }

    match action {
        AdjustAction::AddColl => {
            // Requires approve collateral token first
            println!("--- Step 1: Approve {} -> BorrowerOperations ---", collateral);
            println!("  Token:   {}", asset);
            println!("  Spender: {}", cfg.borrower_operations);
            println!("  Amount:  {} {}", format_18(amount_wei), collateral);
            println!();
            println!(">>> Please confirm Step 1 (approve {} {} for BorrowerOperations). Proceed? [y/N]", format_18(amount_wei), collateral);
            if !dry_run {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted by user.");
                    return Ok(());
                }
            }
            let approve_result = erc20_approve(chain_id, asset, cfg.borrower_operations, amount_wei, Some(&wallet), dry_run).await?;
            println!("Approve tx: {}", extract_tx_hash(&approve_result));
            if !dry_run {
                println!("Waiting 5 seconds before addColl...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }

            println!();
            println!("--- Step 2: addColl ---");
            println!(">>> Please confirm Step 2 (addColl: deposit {} {} into your Vessel). Proceed? [y/N]", format_18(amount_wei), collateral);
            if !dry_run {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted by user.");
                    return Ok(());
                }
            }
            let calldata = encode_add_coll(asset, amount_wei);
            let result = wallet_contract_call(chain_id, cfg.borrower_operations, &calldata, Some(&wallet), None, dry_run).await?;
            println!("addColl tx: {}", extract_tx_hash(&result));
            if dry_run {
                println!("[dry-run] calldata: {}", calldata);
            }
        }

        AdjustAction::WithdrawColl => {
            // No approve needed — protocol pushes tokens to user
            println!("--- withdrawColl (no approve needed) ---");
            println!("  Withdraw: {} {}", format_18(amount_wei), collateral);
            println!();
            println!(">>> Please confirm withdrawColl: remove {} {} from your Vessel. This increases liquidation risk. Proceed? [y/N]", format_18(amount_wei), collateral);
            if !dry_run {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted by user.");
                    return Ok(());
                }
            }
            let calldata = encode_withdraw_coll(asset, amount_wei);
            let result = wallet_contract_call(chain_id, cfg.borrower_operations, &calldata, Some(&wallet), None, dry_run).await?;
            println!("withdrawColl tx: {}", extract_tx_hash(&result));
            if dry_run {
                println!("[dry-run] calldata: {}", calldata);
            }
        }

        AdjustAction::Borrow => {
            // No approve needed — protocol mints GRAI to user
            println!("--- withdrawDebtTokens / Borrow more GRAI (no approve needed) ---");
            println!("  Borrow: {} GRAI", format_18(amount_wei));
            println!();
            println!(">>> Please confirm borrow: mint {} additional GRAI against your Vessel. This increases debt and liquidation risk. Proceed? [y/N]", format_18(amount_wei));
            if !dry_run {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted by user.");
                    return Ok(());
                }
            }
            let calldata = encode_withdraw_debt(asset, amount_wei);
            let result = wallet_contract_call(chain_id, cfg.borrower_operations, &calldata, Some(&wallet), None, dry_run).await?;
            println!("withdrawDebtTokens tx: {}", extract_tx_hash(&result));
            if dry_run {
                println!("[dry-run] calldata: {}", calldata);
            }
        }

        AdjustAction::Repay => {
            // Requires approve GRAI first
            println!("--- Step 1: Approve GRAI -> BorrowerOperations ---");
            println!("  GRAI:    {}", cfg.grai_token);
            println!("  Spender: {}", cfg.borrower_operations);
            println!("  Amount:  {} GRAI", format_18(amount_wei));
            println!();
            println!(">>> Please confirm Step 1 (approve {} GRAI for BorrowerOperations). Proceed? [y/N]", format_18(amount_wei));
            if !dry_run {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted by user.");
                    return Ok(());
                }
            }
            let approve_result = erc20_approve(chain_id, cfg.grai_token, cfg.borrower_operations, amount_wei, Some(&wallet), dry_run).await?;
            println!("Approve tx: {}", extract_tx_hash(&approve_result));
            if !dry_run {
                println!("Waiting 5 seconds before repayDebtTokens...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }

            println!();
            println!("--- Step 2: repayDebtTokens ---");
            println!(">>> Please confirm Step 2 (repayDebtTokens: repay {} GRAI to your Vessel). Proceed? [y/N]", format_18(amount_wei));
            if !dry_run {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Aborted by user.");
                    return Ok(());
                }
            }
            let calldata = encode_repay_debt(asset, amount_wei);
            let result = wallet_contract_call(chain_id, cfg.borrower_operations, &calldata, Some(&wallet), None, dry_run).await?;
            println!("repayDebtTokens tx: {}", extract_tx_hash(&result));
            if dry_run {
                println!("[dry-run] calldata: {}", calldata);
            }
        }
    }

    Ok(())
}

// positions — fetch user's account summary and per-asset balances
// Uses:
//   Pool.getUserAccountData(address) — selector 0xbf92857c
//   ProtocolDataProvider.getUserReserveData(asset, user) — selector 0x28dd0f6e
// Market list fetched from REST API to get asset addresses.

use crate::config::{API_MARKETS, CHAIN_ID, POOL, PROTOCOL_DATA_PROVIDER};
use crate::onchainos::resolve_wallet;
use crate::rpc::{build_client, get_user_account_data, get_user_reserve_data};
use anyhow::anyhow;
use clap::Args;
use serde_json::Value;

#[derive(Args, Debug)]
pub struct PositionsArgs {
    /// Wallet address to query (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &PositionsArgs) -> anyhow::Result<()> {
    let wallet = args
        .from
        .clone()
        .unwrap_or_else(|| resolve_wallet(CHAIN_ID).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Pass --from or ensure onchainos is logged in.");
    }

    // Fetch account summary
    let account_data = get_user_account_data(POOL, &wallet).await?;

    // Fetch market list to get asset addresses
    let client = build_client();
    let markets_resp: Value = client
        .get(API_MARKETS)
        .send()
        .await?
        .json()
        .await?;
    let reserves = markets_resp["reserves"]
        .as_array()
        .ok_or_else(|| anyhow!("No 'reserves' array in API response"))?;

    // Check per-asset balances for non-zero positions
    let mut supplied = Vec::new();
    let mut borrowed = Vec::new();

    for r in reserves {
        let asset = r["underlyingAsset"].as_str().unwrap_or("");
        let symbol = r["symbol"].as_str().unwrap_or("?");
        let decimals = r["decimals"]
            .as_str()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(18);

        if asset.is_empty() {
            continue;
        }

        match get_user_reserve_data(PROTOCOL_DATA_PROVIDER, asset, &wallet).await {
            Ok(reserve_data) => {
                let a_token_balance = reserve_data["currentATokenBalance"]
                    .as_str()
                    .and_then(|s| s.parse::<u128>().ok())
                    .unwrap_or(0);
                let variable_debt = reserve_data["currentVariableDebt"]
                    .as_str()
                    .and_then(|s| s.parse::<u128>().ok())
                    .unwrap_or(0);
                let use_as_collateral = reserve_data["usageAsCollateralEnabled"]
                    .as_bool()
                    .unwrap_or(false);

                let dec_factor = 10u128.pow(decimals);

                if a_token_balance > 0 {
                    supplied.push(serde_json::json!({
                        "symbol": symbol,
                        "asset": asset,
                        "balance": a_token_balance.to_string(),
                        "balanceHuman": format!("{:.6}", (a_token_balance as f64) / (dec_factor as f64)),
                        "usageAsCollateral": use_as_collateral
                    }));
                }
                if variable_debt > 0 {
                    borrowed.push(serde_json::json!({
                        "symbol": symbol,
                        "asset": asset,
                        "debt": variable_debt.to_string(),
                        "debtHuman": format!("{:.6}", (variable_debt as f64) / (dec_factor as f64)),
                        "interestRateMode": "variable"
                    }));
                }
            }
            Err(_) => {
                // Skip assets that fail (may not be in user's portfolio)
            }
        }
    }

    let health_factor = account_data["healthFactor"].as_f64().unwrap_or(0.0);
    let hf_status = if health_factor <= 0.0 {
        "no-debt"
    } else if health_factor < 1.1 {
        "CRITICAL"
    } else if health_factor < 1.5 {
        "WARNING"
    } else {
        "safe"
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "wallet": wallet,
            "protocol": "HyperLend Core Pools (HyperEVM 999)",
            "accountSummary": {
                "totalCollateralUsd": account_data["totalCollateralUsd"],
                "totalDebtUsd": account_data["totalDebtUsd"],
                "availableBorrowsUsd": account_data["availableBorrowsUsd"],
                "healthFactor": format!("{:.4}", health_factor),
                "healthFactorStatus": hf_status,
                "ltv": account_data["ltv"],
                "liquidationThreshold": account_data["currentLiquidationThreshold"]
            },
            "supplied": supplied,
            "borrowed": borrowed
        }))?
    );
    Ok(())
}

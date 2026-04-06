use crate::rpc::build_client;
use crate::config::API_MARKETS;
use anyhow::anyhow;
use clap::Args;
use serde_json::Value;

#[derive(Args, Debug)]
pub struct GetMarketsArgs {
    /// Only show active and non-frozen markets
    #[arg(long)]
    pub active_only: bool,
}

pub async fn execute(args: &GetMarketsArgs) -> anyhow::Result<()> {
    let client = build_client();
    let resp: Value = client
        .get(API_MARKETS)
        .send()
        .await?
        .json()
        .await?;

    let reserves = resp["reserves"]
        .as_array()
        .ok_or_else(|| anyhow!("No 'reserves' array in API response"))?;

    let mut markets = Vec::new();
    for r in reserves {
        let is_active = r["isActive"].as_bool().unwrap_or(false);
        let is_frozen = r["isFrozen"].as_bool().unwrap_or(true);
        if args.active_only && (!is_active || is_frozen) {
            continue;
        }

        let symbol = r["symbol"].as_str().unwrap_or("?");
        let decimals = r["decimals"]
            .as_str()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(18);
        let underlying_asset = r["underlyingAsset"].as_str().unwrap_or("");

        // Rates are in ray (1e27); convert to APY %
        let supply_rate_raw = r["liquidityRate"]
            .as_str()
            .and_then(|s| s.parse::<u128>().ok())
            .unwrap_or(0);
        let borrow_rate_raw = r["variableBorrowRate"]
            .as_str()
            .and_then(|s| s.parse::<u128>().ok())
            .unwrap_or(0);
        let supply_apy = (supply_rate_raw as f64) / 1e25;
        let borrow_apy = (borrow_rate_raw as f64) / 1e25;

        // Liquidity and debt (raw units, need decimals to convert)
        let avail_liq = r["availableLiquidity"]
            .as_str()
            .and_then(|s| s.parse::<u128>().ok())
            .unwrap_or(0);
        let total_debt = r["totalScaledVariableDebt"]
            .as_str()
            .and_then(|s| s.parse::<u128>().ok())
            .unwrap_or(0);
        let dec_factor = 10u128.pow(decimals);
        let avail_liq_human = (avail_liq as f64) / (dec_factor as f64);
        let total_debt_human = (total_debt as f64) / (dec_factor as f64);
        let total_supply_human = avail_liq_human + total_debt_human;
        let utilization = if total_supply_human > 0.0 {
            total_debt_human / total_supply_human * 100.0
        } else {
            0.0
        };

        let ltv = r["baseLTVasCollateral"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
            / 100.0;
        let liq_threshold = r["reserveLiquidationThreshold"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
            / 100.0;
        let borrow_enabled = r["borrowingEnabled"].as_bool().unwrap_or(false);

        markets.push(serde_json::json!({
            "symbol": symbol,
            "underlyingAsset": underlying_asset,
            "decimals": decimals,
            "supplyApy": format!("{:.2}%", supply_apy),
            "borrowApy": format!("{:.2}%", borrow_apy),
            "totalSupply": format!("{:.4}", total_supply_human),
            "availableLiquidity": format!("{:.4}", avail_liq_human),
            "totalVariableDebt": format!("{:.4}", total_debt_human),
            "utilizationRate": format!("{:.1}%", utilization),
            "ltv": format!("{:.1}%", ltv),
            "liquidationThreshold": format!("{:.1}%", liq_threshold),
            "isActive": is_active,
            "isFrozen": is_frozen,
            "borrowingEnabled": borrow_enabled,
            "aTokenAddress": r["aTokenAddress"].as_str().unwrap_or(""),
            "variableDebtTokenAddress": r["variableDebtTokenAddress"].as_str().unwrap_or("")
        }));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "chain": "HyperEVM (999)",
            "protocol": "HyperLend Core Pools",
            "marketCount": markets.len(),
            "markets": markets
        }))?
    );
    Ok(())
}

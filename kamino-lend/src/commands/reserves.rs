use anyhow::Result;
use serde_json::Value;

use crate::api;
use crate::config::MAIN_MARKET;

pub async fn run(client: &reqwest::Client, market: Option<&str>) -> Result<()> {
    let market_pubkey = market.unwrap_or(MAIN_MARKET);
    println!("Fetching reserve metrics for market: {market_pubkey}");

    let metrics: Value = api::get_reserves_metrics(client, market_pubkey).await?;

    let display = if metrics.is_array() {
        metrics.clone()
    } else if let Some(arr) = metrics["data"].as_array() {
        Value::Array(arr.clone())
    } else {
        metrics.clone()
    };

    if let Some(arr) = display.as_array() {
        println!("Reserves ({} total):", arr.len());
        for reserve in arr {
            let pubkey = reserve["reserve"]
                .as_str()
                .or_else(|| reserve["pubkey"].as_str())
                .unwrap_or("unknown");
            let symbol = reserve["symbol"]
                .as_str()
                .or_else(|| reserve["tokenSymbol"].as_str())
                .or_else(|| reserve["liquidityToken"].as_str())
                .unwrap_or("?");
            let supply_apy = reserve["supplyInterestAPY"]
                .as_f64()
                .or_else(|| reserve["supplyApy"].as_f64())
                .or_else(|| {
                    reserve["supplyApy"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                })
                .unwrap_or(0.0);
            let borrow_apy = reserve["borrowInterestAPY"]
                .as_f64()
                .or_else(|| reserve["borrowApy"].as_f64())
                .or_else(|| {
                    reserve["borrowApy"]
                        .as_str()
                        .and_then(|s| s.parse::<f64>().ok())
                })
                .unwrap_or(0.0);
            println!(
                "  {symbol} ({pubkey:.12}...)\n    Supply APY: {:.4}%  Borrow APY: {:.4}%",
                supply_apy * 100.0,
                borrow_apy * 100.0
            );
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&display)?);
    }

    Ok(())
}

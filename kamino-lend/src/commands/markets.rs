use anyhow::Result;
use serde_json::Value;

use crate::api;

pub async fn run(client: &reqwest::Client) -> Result<()> {
    let markets: Value = api::get_markets(client).await?;

    // Pretty-print the top-level list or data field
    let display = if markets.is_array() {
        markets.clone()
    } else if let Some(arr) = markets["data"].as_array() {
        Value::Array(arr.clone())
    } else {
        markets.clone()
    };

    if let Some(arr) = display.as_array() {
        println!("Kamino Markets ({} total):", arr.len());
        for (i, market) in arr.iter().enumerate() {
            let pubkey = market["lendingMarket"]
                .as_str()
                .or_else(|| market["pubkey"].as_str())
                .or_else(|| market["address"].as_str())
                .unwrap_or("unknown");
            let name = market["name"]
                .as_str()
                .unwrap_or("unnamed");
            println!("  [{i}] {name} — {pubkey}");
        }
    } else {
        println!("{}", serde_json::to_string_pretty(&display)?);
    }

    Ok(())
}

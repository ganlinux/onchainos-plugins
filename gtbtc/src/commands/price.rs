use crate::api::build_client;

pub async fn run() -> anyhow::Result<()> {
    let client = build_client();
    let ticker = crate::api::get_price(&client).await?;

    let last: f64 = ticker.last.parse().unwrap_or(0.0);
    let change: f64 = ticker.change_percentage.parse().unwrap_or(0.0);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "data": {
                "pair": "GTBTC_USDT",
                "price_usd": last,
                "price_usd_str": ticker.last,
                "change_24h_pct": change,
                "high_24h": ticker.high_24h,
                "low_24h": ticker.low_24h,
                "base_volume_24h": ticker.base_volume,
                "quote_volume_24h": ticker.quote_volume,
                "source": "gate.io"
            }
        })
    );
    Ok(())
}

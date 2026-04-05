use crate::api::build_client;

pub async fn run() -> anyhow::Result<()> {
    let client = build_client();
    let apr_data = crate::api::get_apr(&client).await?;

    match apr_data {
        Some(data) => {
            let min_rate = data.min_lend_rate.as_deref().unwrap_or("N/A");
            let max_rate = data.max_lend_rate.as_deref().unwrap_or("N/A");
            let avg_rate = data.avg_lend_rate.as_deref().unwrap_or("N/A");

            // Rates from Gate API are daily rates; multiply by 365 for APR
            let min_apr: Option<f64> = data
                .min_lend_rate
                .as_deref()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|r| r * 365.0 * 100.0);
            let max_apr: Option<f64> = data
                .max_lend_rate
                .as_deref()
                .and_then(|s| s.parse::<f64>().ok())
                .map(|r| r * 365.0 * 100.0);

            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "data": {
                        "token": "GTBTC",
                        "min_lend_rate_daily": min_rate,
                        "max_lend_rate_daily": max_rate,
                        "avg_lend_rate_daily": avg_rate,
                        "min_apr_pct": min_apr,
                        "max_apr_pct": max_apr,
                        "note": "APR shown is Gate Flex Earn lending rate for GTBTC. GTBTC native BTC staking yield accrues automatically via NAV growth.",
                        "staking_url": "https://www.gate.com/staking/BTC",
                        "source": "gate.io"
                    }
                })
            );
        }
        None => {
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "data": {
                        "token": "GTBTC",
                        "min_apr_pct": null,
                        "max_apr_pct": null,
                        "note": "Gate Flex Earn APR for GTBTC is currently unavailable. GTBTC earns BTC staking yield automatically via NAV growth (approx 3-5% APR). Check https://www.gate.com/staking/BTC for current rates.",
                        "staking_url": "https://www.gate.com/staking/BTC",
                        "source": "gate.io"
                    }
                })
            );
        }
    }
    Ok(())
}

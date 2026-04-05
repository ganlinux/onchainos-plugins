use crate::{api, onchainos};
use anyhow::Result;

const JITOSOL_MINT: &str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

pub async fn run() -> Result<()> {
    let wallet = onchainos::resolve_wallet_solana()?;
    println!("Wallet: {}", wallet);

    // Primary: onchainos wallet balance with token-address filter
    let balance_result = onchainos::wallet_balance_solana_token(JITOSOL_MINT)
        .unwrap_or(serde_json::json!({}));

    // Try to parse JitoSOL balance from onchainos response
    let jitosol_from_onchainos = balance_result["data"]["details"]
        .as_array()
        .and_then(|details| {
            details.iter().find_map(|d| {
                d["tokenAssets"].as_array()?.iter().find_map(|a| {
                    if a["address"].as_str() == Some(JITOSOL_MINT) {
                        a["balance"].as_str()?.parse::<f64>().ok()
                    } else {
                        None
                    }
                })
            })
        });

    // Fallback to Solana RPC if onchainos parse fails
    let jitosol_balance = match jitosol_from_onchainos {
        Some(b) if b > 0.0 => b,
        _ => api::get_user_jitosol_balance(&wallet).await.unwrap_or(0.0),
    };

    // Get APY and SOL price from onchainos defi detail
    let detail = onchainos::defi_detail_jito().unwrap_or(serde_json::json!({}));
    let apy: f64 = detail["data"]["baseRate"].as_str().unwrap_or("0").parse().unwrap_or(0.0);
    let sol_price: f64 = detail["data"]["aboutToken"][0]["price"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);

    println!();
    println!("Your Jito Positions");
    println!("===================");
    println!("JitoSOL Balance:  {:.6} JitoSOL", jitosol_balance);
    if sol_price > 0.0 && jitosol_balance > 0.0 {
        println!("USD Value:        ${:.2}", jitosol_balance * sol_price);
    }
    if apy > 0.0 {
        println!("APY:              {:.2}%", apy * 100.0);
    }
    if jitosol_balance == 0.0 {
        println!();
        println!("No JitoSOL holdings. Use 'jito stake --amount <SOL>' to start earning.");
    }
    Ok(())
}

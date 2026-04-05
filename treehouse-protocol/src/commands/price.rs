use crate::config::{format_18, AVAX_CHAIN_ID, ETH_CHAIN_ID, TAVAX_TOKEN, TETH_TOKEN};
use crate::rpc;

/// Get tETH or tAVAX price (convertToAssets ratio for 1 share).
pub async fn run(chain_id: u64) -> anyhow::Result<()> {
    let (token_addr, token_symbol, underlying_symbol) = match chain_id {
        ETH_CHAIN_ID => (TETH_TOKEN, "tETH", "wstETH"),
        AVAX_CHAIN_ID => (TAVAX_TOKEN, "tAVAX", "sAVAX"),
        _ => anyhow::bail!(
            "Unsupported chain_id: {}. Supported: 1 (Ethereum), 43114 (Avalanche)",
            chain_id
        ),
    };

    // 1 share = 1e18
    let one_share: u128 = 1_000_000_000_000_000_000u128;
    let underlying_per_share =
        rpc::erc4626_convert_to_assets(token_addr, one_share, chain_id).await?;

    let result = serde_json::json!({
        "ok": true,
        "data": {
            "chain_id": chain_id,
            "token": token_symbol,
            "underlying_symbol": underlying_symbol,
            "price": format_18(underlying_per_share),
            "price_raw": underlying_per_share.to_string(),
            "description": format!("1 {} = {} {}", token_symbol, format_18(underlying_per_share), underlying_symbol)
        }
    });

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

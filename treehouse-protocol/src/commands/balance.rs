use crate::config::{format_18, AVAX_CHAIN_ID, ETH_CHAIN_ID, TAVAX_TOKEN, TETH_TOKEN};
use crate::rpc;

/// Query tETH or tAVAX balance and its value in underlying assets.
pub async fn run(chain_id: u64, account: Option<&str>) -> anyhow::Result<()> {
    let (token_addr, token_symbol, underlying_symbol) = match chain_id {
        ETH_CHAIN_ID => (TETH_TOKEN, "tETH", "wstETH"),
        AVAX_CHAIN_ID => (TAVAX_TOKEN, "tAVAX", "sAVAX"),
        _ => anyhow::bail!(
            "Unsupported chain_id: {}. Supported: 1 (Ethereum), 43114 (Avalanche)",
            chain_id
        ),
    };

    // Resolve account address
    let wallet_addr;
    let address = match account {
        Some(a) => a.to_string(),
        None => {
            wallet_addr = crate::onchainos::resolve_wallet(chain_id)?;
            if wallet_addr.is_empty() {
                anyhow::bail!(
                    "Cannot resolve wallet address. Please pass --account or log in via onchainos."
                );
            }
            wallet_addr
        }
    };

    // Query balance
    let balance_raw = rpc::erc20_balance_of(token_addr, &address, chain_id).await?;

    // Query underlying value
    let underlying_raw = if balance_raw > 0 {
        rpc::erc4626_convert_to_assets(token_addr, balance_raw, chain_id).await?
    } else {
        0u128
    };

    let result = serde_json::json!({
        "ok": true,
        "data": {
            "account": address,
            "chain_id": chain_id,
            "token": token_symbol,
            "balance": format_18(balance_raw),
            "balance_raw": balance_raw.to_string(),
            "underlying_symbol": underlying_symbol,
            "underlying_value": format_18(underlying_raw),
            "underlying_value_raw": underlying_raw.to_string()
        }
    });

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

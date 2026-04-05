use clap::Args;
use serde_json::json;

use crate::{
    config::{RSWETH_PROXY, SWETH_PROXY},
    rpc::{
        balance_of, eth_to_rsweth_rate, eth_to_sweth_rate, format_eth, format_rate,
        print_json, rsweth_to_eth_rate, sweth_to_eth_rate,
    },
};

#[derive(Args, Debug)]
pub struct BalanceArgs {
    /// Wallet address to query
    #[arg(long)]
    pub address: String,
}

/// balance — query swETH and rswETH balances plus current exchange rates.
pub async fn run(args: BalanceArgs) -> anyhow::Result<()> {
    let addr = &args.address;

    // Fetch balances and rates concurrently (sequential for simplicity)
    let sweth_balance = balance_of(SWETH_PROXY, addr).await.unwrap_or(0);
    let rsweth_balance = balance_of(RSWETH_PROXY, addr).await.unwrap_or(0);

    let sweth_to_eth = sweth_to_eth_rate(SWETH_PROXY).await.unwrap_or(0);
    let eth_to_sweth = eth_to_sweth_rate(SWETH_PROXY).await.unwrap_or(0);
    let rsweth_to_eth = rsweth_to_eth_rate(RSWETH_PROXY).await.unwrap_or(0);
    let eth_to_rsweth = eth_to_rsweth_rate(RSWETH_PROXY).await.unwrap_or(0);

    // Calculate ETH equivalent of holdings
    let sweth_eth_value = if sweth_to_eth > 0 {
        (sweth_balance as u128)
            .checked_mul(sweth_to_eth)
            .unwrap_or(0)
            / 1_000_000_000_000_000_000u128
    } else {
        0
    };
    let rsweth_eth_value = if rsweth_to_eth > 0 {
        (rsweth_balance as u128)
            .checked_mul(rsweth_to_eth)
            .unwrap_or(0)
            / 1_000_000_000_000_000_000u128
    } else {
        0
    };

    print_json(&json!({
        "ok": true,
        "address": addr,
        "swETH": {
            "contract": SWETH_PROXY,
            "balance_wei": sweth_balance.to_string(),
            "balance": format_eth(sweth_balance),
            "eth_equivalent": format_eth(sweth_eth_value),
            "swETHToETHRate": format_rate(sweth_to_eth),
            "ethToSwETHRate": format_rate(eth_to_sweth)
        },
        "rswETH": {
            "contract": RSWETH_PROXY,
            "balance_wei": rsweth_balance.to_string(),
            "balance": format_eth(rsweth_balance),
            "eth_equivalent": format_eth(rsweth_eth_value),
            "rswETHToETHRate": format_rate(rsweth_to_eth),
            "ethToRswETHRate": format_rate(eth_to_rsweth)
        }
    }));
    Ok(())
}

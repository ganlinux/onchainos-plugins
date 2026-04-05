use clap::Args;
use serde_json::json;

use crate::{
    config::{RSWETH_PROXY, SIMPLE_STAKING_ERC20, SWEXIT_PROXY, SWETH_PROXY},
    rpc::{
        balance_of, format_eth, format_rate, get_last_token_id_created,
        get_last_token_id_processed, get_processed_rate_for_token_id, print_json,
        rsweth_to_eth_rate, sweth_to_eth_rate,
    },
};

#[derive(Args, Debug)]
pub struct PositionsArgs {
    /// Wallet address to query
    #[arg(long)]
    pub address: String,
}

/// positions — show complete staking positions and pending withdrawals.
pub async fn run(args: PositionsArgs) -> anyhow::Result<()> {
    let addr = &args.address;

    // 1. Balances
    let sweth_balance = balance_of(SWETH_PROXY, addr).await.unwrap_or(0);
    let rsweth_balance = balance_of(RSWETH_PROXY, addr).await.unwrap_or(0);

    // 2. Rates
    let sweth_to_eth = sweth_to_eth_rate(SWETH_PROXY).await.unwrap_or(0);
    let rsweth_to_eth = rsweth_to_eth_rate(RSWETH_PROXY).await.unwrap_or(0);

    // 3. swEXIT withdrawal status
    let last_created = get_last_token_id_created(SWEXIT_PROXY)
        .await
        .unwrap_or(0);
    let last_processed = get_last_token_id_processed(SWEXIT_PROXY)
        .await
        .unwrap_or(0);

    let pending_count = if last_created > last_processed {
        last_created - last_processed
    } else {
        0
    };

    // Check most recent pending withdrawal status
    let withdrawal_status = if last_created > 0 {
        let (is_processed, rate) =
            get_processed_rate_for_token_id(SWEXIT_PROXY, last_created)
                .await
                .unwrap_or((false, 0));
        json!({
            "last_token_id_created": last_created.to_string(),
            "last_token_id_processed": last_processed.to_string(),
            "pending_requests": pending_count.to_string(),
            "latest_request_processed": is_processed,
            "latest_request_processed_rate": rate.to_string()
        })
    } else {
        json!({
            "last_token_id_created": "0",
            "last_token_id_processed": "0",
            "pending_requests": "0"
        })
    };

    // ETH values
    let sweth_eth = if sweth_to_eth > 0 {
        (sweth_balance as u128)
            .checked_mul(sweth_to_eth)
            .unwrap_or(0)
            / 1_000_000_000_000_000_000u128
    } else {
        0
    };
    let rsweth_eth = if rsweth_to_eth > 0 {
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
        "liquid_staking": {
            "swETH": {
                "balance": format_eth(sweth_balance),
                "balance_wei": sweth_balance.to_string(),
                "eth_value": format_eth(sweth_eth),
                "rate_sweth_per_eth": format_rate(sweth_to_eth),
                "apr_note": "~3% APR (repricing token)"
            },
            "rswETH": {
                "balance": format_eth(rsweth_balance),
                "balance_wei": rsweth_balance.to_string(),
                "eth_value": format_eth(rsweth_eth),
                "rate_rsweth_per_eth": format_rate(rsweth_to_eth),
                "apr_note": "~2.63% APR + EigenLayer restaking rewards"
            }
        },
        "earn_pool": {
            "contract": SIMPLE_STAKING_ERC20,
            "note": "Use 'balance --address <addr>' to check on-chain deposit balances. SimpleStakingERC20 does not expose per-user view functions."
        },
        "withdrawals": withdrawal_status
    }));
    Ok(())
}

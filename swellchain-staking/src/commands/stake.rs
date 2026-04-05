use clap::Args;
use serde_json::json;

use crate::{
    config::{ETHEREUM_CHAIN_ID, SWETH_PROXY},
    onchainos,
    rpc::{self, print_json},
};

#[derive(Args, Debug)]
pub struct StakeArgs {
    /// Amount of ETH to stake in wei (e.g. 500000000000000000 for 0.5 ETH)
    #[arg(long)]
    pub amt: u128,

    /// Override sender address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,

    /// Simulate without broadcasting
    #[arg(long)]
    pub dry_run: bool,
}

/// stake — deposit ETH -> swETH via deposit() payable on swETH proxy.
/// Selector: deposit() = 0xd0e30db0
pub async fn run(args: StakeArgs) -> anyhow::Result<()> {
    if args.dry_run {
        let calldata = "0xd0e30db0";
        print_json(&json!({
            "ok": true,
            "dry_run": true,
            "action": "stake",
            "contract": SWETH_PROXY,
            "calldata": calldata,
            "amt_wei": args.amt.to_string(),
            "description": "deposit() payable — ETH -> swETH"
        }));
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(ETHEREUM_CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Please pass --from or ensure onchainos is logged in.");
    }
    let from_addr = args.from.as_deref().unwrap_or(&wallet);

    // Query current exchange rate for user information
    let eth_to_sweth = rpc::eth_to_sweth_rate(SWETH_PROXY).await.unwrap_or(0);
    let expected_sweth = if eth_to_sweth > 0 {
        (args.amt as u128)
            .checked_mul(eth_to_sweth)
            .unwrap_or(0)
            / 1_000_000_000_000_000_000u128
    } else {
        0
    };

    // deposit() has no parameters — calldata is just the 4-byte selector
    let calldata = "0xd0e30db0";

    let result = onchainos::wallet_contract_call(
        ETHEREUM_CHAIN_ID,
        SWETH_PROXY,
        calldata,
        Some(from_addr),
        Some(args.amt),
        false,
    )
    .await?;

    let tx_hash = onchainos::extract_tx_hash(&result);
    print_json(&json!({
        "ok": true,
        "action": "stake",
        "contract": SWETH_PROXY,
        "eth_staked_wei": args.amt.to_string(),
        "eth_staked": rpc::format_eth(args.amt),
        "expected_sweth": rpc::format_eth(expected_sweth),
        "txHash": tx_hash,
        "raw": result
    }));
    Ok(())
}

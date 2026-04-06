use clap::Args;
use serde_json::json;

use crate::{
    config::{ETHEREUM_CHAIN_ID, SWEXIT_PROXY, SWETH_PROXY},
    onchainos,
    rpc::{self, print_json},
};

#[derive(Args, Debug)]
pub struct RequestWithdrawalArgs {
    /// Amount of swETH to withdraw in wei
    #[arg(long)]
    pub amt: u128,

    /// Override sender address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,

    /// Simulate without broadcasting
    #[arg(long)]
    pub dry_run: bool,
}

/// request-withdrawal — approve swEXIT + createWithdrawRequest(uint256).
/// Creates a swEXIT NFT representing the withdrawal request.
/// Selector: createWithdrawRequest(uint256) = 0x74dc9d1a
pub async fn run(args: RequestWithdrawalArgs) -> anyhow::Result<()> {
    if args.dry_run {
        let approve_calldata = build_approve_calldata(SWEXIT_PROXY, args.amt);
        let withdraw_calldata = build_create_withdraw_request_calldata(args.amt);
        print_json(&json!({
            "ok": true,
            "dry_run": true,
            "action": "request-withdrawal",
            "sweth_contract": SWETH_PROXY,
            "swexit_contract": SWEXIT_PROXY,
            "amt_wei": args.amt.to_string(),
            "step1_approve_calldata": approve_calldata,
            "step2_create_withdraw_request_calldata": withdraw_calldata,
            "note": "Withdrawal takes 1-12 days. Use finalize-withdrawal with the NFT tokenId once processed."
        }));
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(ETHEREUM_CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Please pass --from or ensure onchainos is logged in.");
    }
    let from_addr = args.from.as_deref().unwrap_or(&wallet);

    // Check swETH balance
    let balance = rpc::balance_of(SWETH_PROXY, from_addr).await.unwrap_or(0);
    if balance < args.amt {
        anyhow::bail!(
            "Insufficient swETH balance. Have: {} wei, need: {} wei",
            balance,
            args.amt
        );
    }

    // Step 1: approve swEXIT to spend swETH
    let approve_result = onchainos::erc20_approve(
        ETHEREUM_CHAIN_ID,
        SWETH_PROXY,
        SWEXIT_PROXY,
        args.amt,
        Some(from_addr),
        false,
    )
    .await?;
    if approve_result["ok"].as_bool() != Some(true) {
        anyhow::bail!("approve failed: {}", approve_result["error"].as_str().unwrap_or("unknown error"));
    }
    let approve_tx = onchainos::extract_tx_hash(&approve_result);
    tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;

    // Step 2: createWithdrawRequest(amount)
    let withdraw_calldata = build_create_withdraw_request_calldata(args.amt);
    let withdraw_result = onchainos::wallet_contract_call_force(
        ETHEREUM_CHAIN_ID,
        SWEXIT_PROXY,
        &withdraw_calldata,
        Some(from_addr),
        None,
        true,
        Some(200_000),
        false,
    )
    .await?;
    if withdraw_result["ok"].as_bool() != Some(true) {
        anyhow::bail!("request-withdrawal failed: {}", withdraw_result["error"].as_str().unwrap_or("unknown error"));
    }
    let withdraw_tx = onchainos::extract_tx_hash(&withdraw_result);

    print_json(&json!({
        "ok": true,
        "action": "request-withdrawal",
        "sweth_contract": SWETH_PROXY,
        "swexit_contract": SWEXIT_PROXY,
        "amt_wei": args.amt.to_string(),
        "amt_sweth": rpc::format_eth(args.amt),
        "approve_txHash": approve_tx,
        "withdraw_request_txHash": withdraw_tx,
        "note": "Withdrawal NFT (swEXIT) created. Check status with 'positions' command. Finalize after 1-12 days with 'finalize-withdrawal --token-id <id>'.",
    }));
    Ok(())
}

/// Build approve(address,uint256) calldata for swEXIT as spender.
fn build_approve_calldata(spender: &str, amount: u128) -> String {
    let spender_stripped = spender.strip_prefix("0x").unwrap_or(spender);
    let spender_padded = format!("{:0>64}", spender_stripped);
    let amount_hex = format!("{:064x}", amount);
    format!("0x095ea7b3{}{}", spender_padded, amount_hex)
}

/// Build createWithdrawRequest(uint256) calldata.
/// Selector: 0x74dc9d1a
fn build_create_withdraw_request_calldata(amount: u128) -> String {
    let amount_hex = format!("{:064x}", amount);
    format!("0x74dc9d1a{}", amount_hex)
}

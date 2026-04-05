use clap::Args;
use serde_json::json;

use crate::{
    config::{ETHEREUM_CHAIN_ID, RSWETH_PROXY, SIMPLE_STAKING_ERC20, SWETH_PROXY},
    onchainos,
    rpc::print_json,
};

#[derive(Args, Debug)]
pub struct EarnWithdrawArgs {
    /// Token to withdraw: "swETH" or "rswETH"
    #[arg(long, default_value = "swETH")]
    pub token: String,

    /// Amount to withdraw in wei
    #[arg(long)]
    pub amt: u128,

    /// Override sender/receiver address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,

    /// Simulate without broadcasting
    #[arg(long)]
    pub dry_run: bool,
}

/// earn-withdraw — withdraw token from SimpleStakingERC20.
/// withdraw(address,uint256,address) selector: 0x69328dec
pub async fn run(args: EarnWithdrawArgs) -> anyhow::Result<()> {
    let token_addr = resolve_token_address(&args.token)?;

    if args.dry_run {
        let calldata = build_withdraw_calldata(token_addr, args.amt, "0x<wallet>");
        print_json(&json!({
            "ok": true,
            "dry_run": true,
            "action": "earn-withdraw",
            "token": args.token,
            "token_addr": token_addr,
            "staking_contract": SIMPLE_STAKING_ERC20,
            "amt_wei": args.amt.to_string(),
            "calldata": calldata,
        }));
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(ETHEREUM_CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Please pass --from or ensure onchainos is logged in.");
    }
    let from_addr = args.from.as_deref().unwrap_or(&wallet);

    // withdraw(token, amount, receiver)
    let calldata = build_withdraw_calldata(token_addr, args.amt, from_addr);
    let result = onchainos::wallet_contract_call(
        ETHEREUM_CHAIN_ID,
        SIMPLE_STAKING_ERC20,
        &calldata,
        Some(from_addr),
        None,
        false,
    )
    .await?;
    let tx_hash = onchainos::extract_tx_hash(&result);

    print_json(&json!({
        "ok": true,
        "action": "earn-withdraw",
        "token": args.token,
        "token_addr": token_addr,
        "staking_contract": SIMPLE_STAKING_ERC20,
        "amt_wei": args.amt.to_string(),
        "txHash": tx_hash,
        "raw": result
    }));
    Ok(())
}

fn resolve_token_address(token: &str) -> anyhow::Result<&'static str> {
    match token.to_lowercase().as_str() {
        "sweth" => Ok(SWETH_PROXY),
        "rsweth" => Ok(RSWETH_PROXY),
        _ => anyhow::bail!("Unsupported token '{}'. Use 'swETH' or 'rswETH'.", token),
    }
}

/// Build withdraw(address,uint256,address) calldata.
/// Selector: 0x69328dec
fn build_withdraw_calldata(token: &str, amount: u128, receiver: &str) -> String {
    let token_stripped = token.strip_prefix("0x").unwrap_or(token);
    let token_padded = format!("{:0>64}", token_stripped);
    let amount_hex = format!("{:064x}", amount);
    let receiver_stripped = receiver.strip_prefix("0x").unwrap_or(receiver);
    let receiver_padded = format!("{:0>64}", receiver_stripped);
    format!("0x69328dec{}{}{}", token_padded, amount_hex, receiver_padded)
}

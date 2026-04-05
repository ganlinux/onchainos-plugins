use clap::Args;
use serde_json::json;

use crate::{
    config::{ETHEREUM_CHAIN_ID, RSWETH_PROXY, SIMPLE_STAKING_ERC20, SWETH_PROXY},
    onchainos,
    rpc::print_json,
};

#[derive(Args, Debug)]
pub struct EarnDepositArgs {
    /// Token to deposit: "swETH" or "rswETH"
    #[arg(long, default_value = "swETH")]
    pub token: String,

    /// Amount to deposit in wei
    #[arg(long)]
    pub amt: u128,

    /// Override sender/receiver address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,

    /// Simulate without broadcasting
    #[arg(long)]
    pub dry_run: bool,
}

/// earn-deposit — approve + deposit token into SimpleStakingERC20.
/// deposit(address,uint256,address) selector: 0xf45346dc
pub async fn run(args: EarnDepositArgs) -> anyhow::Result<()> {
    let token_addr = resolve_token_address(&args.token)?;

    if args.dry_run {
        let approve_calldata = build_approve_calldata(SIMPLE_STAKING_ERC20, args.amt);
        let deposit_calldata = build_deposit_calldata(token_addr, args.amt, "0x<wallet>");
        print_json(&json!({
            "ok": true,
            "dry_run": true,
            "action": "earn-deposit",
            "token": args.token,
            "token_addr": token_addr,
            "staking_contract": SIMPLE_STAKING_ERC20,
            "amt_wei": args.amt.to_string(),
            "step1_approve_calldata": approve_calldata,
            "step2_deposit_calldata": deposit_calldata,
        }));
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(ETHEREUM_CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Please pass --from or ensure onchainos is logged in.");
    }
    let from_addr = args.from.as_deref().unwrap_or(&wallet);

    // Step 1: approve SimpleStakingERC20 to spend the token
    let approve_result = onchainos::erc20_approve(
        ETHEREUM_CHAIN_ID,
        token_addr,
        SIMPLE_STAKING_ERC20,
        args.amt,
        Some(from_addr),
        false,
    )
    .await?;
    let approve_tx = onchainos::extract_tx_hash(&approve_result);

    // Step 2: deposit(token, amount, receiver)
    let deposit_calldata = build_deposit_calldata(token_addr, args.amt, from_addr);
    let deposit_result = onchainos::wallet_contract_call(
        ETHEREUM_CHAIN_ID,
        SIMPLE_STAKING_ERC20,
        &deposit_calldata,
        Some(from_addr),
        None,
        false,
    )
    .await?;
    let deposit_tx = onchainos::extract_tx_hash(&deposit_result);

    print_json(&json!({
        "ok": true,
        "action": "earn-deposit",
        "token": args.token,
        "token_addr": token_addr,
        "staking_contract": SIMPLE_STAKING_ERC20,
        "amt_wei": args.amt.to_string(),
        "approve_txHash": approve_tx,
        "deposit_txHash": deposit_tx,
        "approve_raw": approve_result,
        "deposit_raw": deposit_result
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

/// Build approve(address,uint256) calldata for SIMPLE_STAKING_ERC20 as spender.
fn build_approve_calldata(spender: &str, amount: u128) -> String {
    let spender_stripped = spender.strip_prefix("0x").unwrap_or(spender);
    let spender_padded = format!("{:0>64}", spender_stripped);
    let amount_hex = format!("{:064x}", amount);
    format!("0x095ea7b3{}{}", spender_padded, amount_hex)
}

/// Build deposit(address,uint256,address) calldata.
/// Selector: 0xf45346dc
fn build_deposit_calldata(token: &str, amount: u128, receiver: &str) -> String {
    let token_stripped = token.strip_prefix("0x").unwrap_or(token);
    let token_padded = format!("{:0>64}", token_stripped);
    let amount_hex = format!("{:064x}", amount);
    let receiver_stripped = receiver.strip_prefix("0x").unwrap_or(receiver);
    let receiver_padded = format!("{:0>64}", receiver_stripped);
    format!("0xf45346dc{}{}{}", token_padded, amount_hex, receiver_padded)
}

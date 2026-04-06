// repay — ERC-20 approve + Pool.repay(asset, amount, interestRateMode, onBehalfOf)
// Selector: 0x573ade81
// Note: Do NOT use uint256.max for repay-all. Use actual token balance as amount.
// Interest accrues per-second; uint256.max will try to pull debtAmount which may
// exceed wallet balance and revert.

use crate::config::{CHAIN_ID, POOL, SEL_REPAY};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use clap::Args;

#[derive(Args, Debug)]
pub struct RepayArgs {
    /// ERC-20 token address (same as borrowed asset)
    #[arg(long)]
    pub asset: String,

    /// Amount in raw token units. Use your actual token balance for repay-all
    /// (do NOT use u128::MAX — interest accrues and the pool may revert).
    #[arg(long)]
    pub amount: u128,

    /// Repayer wallet address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &RepayArgs, dry_run: bool) -> anyhow::Result<()> {
    let asset_padded = format!("{:0>64}", args.asset.trim_start_matches("0x"));
    let amount_padded = format!("{:064x}", args.amount);
    let interest_rate_mode = format!("{:064x}", 2u64); // 2 = variable

    if dry_run {
        let placeholder_wallet = "0000000000000000000000000000000000000000";
        let repay_calldata = format!(
            "{}{}{}{}{}",
            SEL_REPAY,
            asset_padded,
            amount_padded,
            interest_rate_mode,
            format!("{:0>64}", placeholder_wallet)
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "dry_run": true,
                "steps": [
                    {
                        "step": 1,
                        "action": "ERC-20 approve",
                        "to": args.asset,
                        "calldata": format!("0x095ea7b3{:0>64}{:064x}", POOL.trim_start_matches("0x"), args.amount)
                    },
                    {
                        "step": 2,
                        "action": "Pool.repay",
                        "to": POOL,
                        "calldata": repay_calldata
                    }
                ],
                "note": "interestRateMode=2 (variable). Use actual token balance for repay-all, not u128::MAX."
            }))?
        );
        return Ok(());
    }

    let wallet = args
        .from
        .clone()
        .unwrap_or_else(|| resolve_wallet(CHAIN_ID).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Pass --from or ensure onchainos is logged in.");
    }

    // Step 1: approve
    println!("Step 1/2: Approving Pool to spend {} raw units of {}...", args.amount, args.asset);
    let approve_result =
        erc20_approve(CHAIN_ID, &args.asset, POOL, args.amount, Some(&wallet), false).await?;
    let approve_tx = extract_tx_hash(&approve_result);
    println!("  approve txHash: {}", approve_tx);

    // Wait 3 seconds
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Step 2: Pool.repay(asset, amount, interestRateMode, onBehalfOf)
    let wallet_padded = format!("{:0>64}", wallet.trim_start_matches("0x"));
    let calldata = format!(
        "{}{}{}{}{}",
        SEL_REPAY, asset_padded, amount_padded, interest_rate_mode, wallet_padded
    );

    println!("Step 2/2: Calling Pool.repay...");
    let result =
        wallet_contract_call(CHAIN_ID, POOL, &calldata, Some(&wallet), None, false).await?;
    let tx_hash = extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "action": "repay",
            "asset": args.asset,
            "amount": args.amount.to_string(),
            "interestRateMode": "variable (2)",
            "onBehalfOf": wallet,
            "approveTxHash": approve_tx,
            "repayTxHash": tx_hash
        }))?
    );
    Ok(())
}

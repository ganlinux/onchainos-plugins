// withdraw — Pool.withdraw(asset, amount, to)
// Selector: 0x69328dec
// No approval needed. Pool burns hTokens directly.
// Use u128::MAX (type(uint256).max) to withdraw all.
// WARNING: withdraw-all reverts if user has outstanding debt (HF would drop to 0).

use crate::config::{CHAIN_ID, POOL, SEL_WITHDRAW};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use clap::Args;

#[derive(Args, Debug)]
pub struct WithdrawArgs {
    /// ERC-20 token address to withdraw
    #[arg(long)]
    pub asset: String,

    /// Amount in raw token units. Pass 0 to withdraw all (uses uint256.max internally).
    /// WARNING: withdraw-all (amount=0) reverts if you have outstanding debt.
    #[arg(long)]
    pub amount: u128,

    /// Recipient address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &WithdrawArgs, dry_run: bool) -> anyhow::Result<()> {
    // amount=0 means "withdraw all" → use uint256.max
    let actual_amount: u128 = if args.amount == 0 {
        u128::MAX
    } else {
        args.amount
    };

    let asset_padded = format!("{:0>64}", args.asset.trim_start_matches("0x"));
    let amount_padded = format!("{:064x}", actual_amount);

    if dry_run {
        let placeholder_wallet = "0000000000000000000000000000000000000000";
        let calldata = format!(
            "{}{}{}{}",
            SEL_WITHDRAW,
            asset_padded,
            amount_padded,
            format!("{:0>64}", placeholder_wallet)
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "dry_run": true,
                "action": "withdraw",
                "to": POOL,
                "calldata": calldata,
                "withdrawAll": args.amount == 0,
                "note": "If amount=0, uses uint256.max (withdraw all). Reverts if outstanding debt exists."
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

    let wallet_padded = format!("{:0>64}", wallet.trim_start_matches("0x"));
    let calldata = format!(
        "{}{}{}{}",
        SEL_WITHDRAW, asset_padded, amount_padded, wallet_padded
    );

    let result =
        wallet_contract_call(CHAIN_ID, POOL, &calldata, Some(&wallet), None, false).await?;
    let tx_hash = extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "action": "withdraw",
            "asset": args.asset,
            "amount": actual_amount.to_string(),
            "withdrawAll": args.amount == 0,
            "to": wallet,
            "txHash": tx_hash
        }))?
    );
    Ok(())
}

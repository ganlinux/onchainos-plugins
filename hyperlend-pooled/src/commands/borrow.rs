// borrow — Pool.borrow(asset, amount, interestRateMode, referralCode, onBehalfOf)
// Selector: 0xa415bcad
// No ERC-20 approval needed. interestRateMode=2 (variable only in V3.2)

use crate::config::{CHAIN_ID, POOL, SEL_BORROW};
use crate::onchainos::{extract_tx_hash, resolve_wallet, wallet_contract_call};
use clap::Args;

#[derive(Args, Debug)]
pub struct BorrowArgs {
    /// ERC-20 token address to borrow
    #[arg(long)]
    pub asset: String,

    /// Amount in raw token units
    #[arg(long)]
    pub amount: u128,

    /// Borrower wallet address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &BorrowArgs, dry_run: bool) -> anyhow::Result<()> {
    // borrow(asset, amount, interestRateMode=2, referralCode=0, onBehalfOf)
    let asset_padded = format!("{:0>64}", args.asset.trim_start_matches("0x"));
    let amount_padded = format!("{:064x}", args.amount);
    let interest_rate_mode = format!("{:064x}", 2u64); // 2 = variable
    let referral_code = format!("{:064x}", 0u64);

    if dry_run {
        let placeholder_wallet = "0000000000000000000000000000000000000000";
        let calldata = format!(
            "{}{}{}{}{}{}",
            SEL_BORROW,
            asset_padded,
            amount_padded,
            interest_rate_mode,
            referral_code,
            format!("{:0>64}", placeholder_wallet)
        );
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": true,
                "dry_run": true,
                "action": "borrow",
                "to": POOL,
                "calldata": calldata,
                "note": "interestRateMode=2 (variable). No approval needed."
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
        "{}{}{}{}{}{}",
        SEL_BORROW,
        asset_padded,
        amount_padded,
        interest_rate_mode,
        referral_code,
        wallet_padded
    );

    let result =
        wallet_contract_call(CHAIN_ID, POOL, &calldata, Some(&wallet), None, false).await?;
    let tx_hash = extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "action": "borrow",
            "asset": args.asset,
            "amount": args.amount.to_string(),
            "interestRateMode": "variable (2)",
            "onBehalfOf": wallet,
            "txHash": tx_hash,
            "warning": "Maintain health factor > 1.5 to avoid liquidation risk."
        }))?
    );
    Ok(())
}

// supply — ERC-20 approve + Pool.supply(asset, amount, onBehalfOf, referralCode)
// Selector: 0x617ba037
// Step 1: approve Pool to spend token
// Step 2 (after 3s): call Pool.supply

use crate::config::{CHAIN_ID, POOL, SEL_SUPPLY};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use clap::Args;

#[derive(Args, Debug)]
pub struct SupplyArgs {
    /// ERC-20 token address to supply
    #[arg(long)]
    pub asset: String,

    /// Amount in raw token units (e.g. 1000000 for 1 USDC with 6 decimals)
    #[arg(long)]
    pub amount: u128,

    /// Recipient wallet address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &SupplyArgs, dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        let asset_padded = format!("{:0>64}", args.asset.trim_start_matches("0x"));
        let amount_padded = format!("{:064x}", args.amount);
        let on_behalf_padded = format!("{:0>64}", "0000000000000000000000000000000000000000");
        let ref_code_padded = format!("{:064x}", 0u64);
        let supply_calldata = format!(
            "{}{}{}{}{}",
            SEL_SUPPLY, asset_padded, amount_padded, on_behalf_padded, ref_code_padded
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
                        "action": "Pool.supply",
                        "to": POOL,
                        "calldata": supply_calldata
                    }
                ]
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
    println!("Step 1/2: Approving {} to spend {} raw units of {}...", POOL, args.amount, args.asset);
    let approve_result =
        erc20_approve(CHAIN_ID, &args.asset, POOL, args.amount, Some(&wallet), false).await?;
    let approve_tx = extract_tx_hash(&approve_result);
    println!("  approve txHash: {}", approve_tx);

    // Wait 3 seconds to avoid nonce collision
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Step 2: Pool.supply(asset, amount, onBehalfOf, referralCode)
    let asset_padded = format!("{:0>64}", args.asset.trim_start_matches("0x"));
    let amount_padded = format!("{:064x}", args.amount);
    let wallet_padded = format!("{:0>64}", wallet.trim_start_matches("0x"));
    let ref_code_padded = format!("{:064x}", 0u64);
    let calldata = format!(
        "{}{}{}{}{}",
        SEL_SUPPLY, asset_padded, amount_padded, wallet_padded, ref_code_padded
    );

    println!("Step 2/2: Calling Pool.supply...");
    let result =
        wallet_contract_call(CHAIN_ID, POOL, &calldata, Some(&wallet), None, false).await?;
    let tx_hash = extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ok": true,
            "action": "supply",
            "asset": args.asset,
            "amount": args.amount.to_string(),
            "onBehalfOf": wallet,
            "approveTxHash": approve_tx,
            "supplyTxHash": tx_hash,
            "note": "You will receive hTokens (yield-bearing) in your wallet."
        }))?
    );
    Ok(())
}

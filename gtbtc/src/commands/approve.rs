use crate::config::{gtbtc_to_atomic, GTBTC_TOKEN_ADDRESS};
use crate::onchainos;

pub struct ApproveArgs {
    pub spender: String,
    pub amount: Option<f64>, // None = unlimited (u128::MAX)
    pub from: Option<String>,
    pub chain_id: u64,
}

pub async fn run(args: &ApproveArgs, dry_run: bool) -> anyhow::Result<()> {
    let atomic: u128 = match args.amount {
        Some(amt) => gtbtc_to_atomic(amt) as u128,
        None => u128::MAX, // unlimited approval
    };

    // Build EVM calldata: approve(address,uint256)
    // selector = 0x095ea7b3 (verified via cast sig)
    let spender_padded = format!("{:0>64}", args.spender.trim_start_matches("0x"));
    let amount_hex = format!("{:064x}", atomic);
    let calldata = format!("0x095ea7b3{}{}", spender_padded, amount_hex);

    let amount_display = match args.amount {
        Some(a) => format!("{} GTBTC", a),
        None => "unlimited".to_string(),
    };

    if dry_run {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "data": {
                    "action": "approve",
                    "token": "GTBTC",
                    "spender": args.spender,
                    "amount": amount_display,
                    "amount_atomic": atomic.to_string(),
                    "decimals": 8,
                    "chain_id": args.chain_id,
                    "contract": GTBTC_TOKEN_ADDRESS,
                    "calldata": calldata
                }
            })
        );
        return Ok(());
    }

    let from = match &args.from {
        Some(f) => f.clone(),
        None => {
            let addr = onchainos::resolve_wallet(args.chain_id)?;
            if addr.is_empty() {
                anyhow::bail!(
                    "Cannot resolve wallet address for chain {}. Pass --from or ensure onchainos is logged in.",
                    args.chain_id
                );
            }
            addr
        }
    };

    let result = onchainos::wallet_contract_call(
        args.chain_id,
        GTBTC_TOKEN_ADDRESS,
        &calldata,
        Some(&from),
        None,
        false,
    )
    .await?;

    let tx_hash = onchainos::extract_tx_hash(&result);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "data": {
                "action": "approve",
                "token": "GTBTC",
                "owner": from,
                "spender": args.spender,
                "amount": amount_display,
                "amount_atomic": atomic.to_string(),
                "chain_id": args.chain_id,
                "tx_hash": tx_hash,
                "raw": result
            }
        })
    );
    Ok(())
}

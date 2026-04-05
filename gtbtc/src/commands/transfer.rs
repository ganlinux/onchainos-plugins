use crate::config::{gtbtc_to_atomic, GTBTC_TOKEN_ADDRESS, SOLANA_CHAIN_ID};
use crate::onchainos;

pub struct TransferArgs {
    pub to: String,
    pub amount: f64,
    pub from: Option<String>,
    pub chain_id: u64,
}

pub async fn run(args: &TransferArgs, dry_run: bool) -> anyhow::Result<()> {
    let atomic = gtbtc_to_atomic(args.amount);

    if args.chain_id == SOLANA_CHAIN_ID {
        // Solana SPL transfer — not fully implemented in v1
        println!(
            "{}",
            serde_json::json!({
                "ok": false,
                "error": "Solana SPL transfer for GTBTC is not supported in v1. Use 'onchainos swap execute' to swap GTBTC on Solana DEX.",
                "hint": "For Solana GTBTC transfers, use onchainos swap execute --chain 501 --from gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So"
            })
        );
        return Ok(());
    }

    // Build EVM calldata: transfer(address,uint256)
    // selector = 0xa9059cbb (verified via cast sig)
    let to_padded = format!("{:0>64}", args.to.trim_start_matches("0x"));
    let amount_hex = format!("{:064x}", atomic);
    let calldata = format!("0xa9059cbb{}{}", to_padded, amount_hex);

    if dry_run {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "data": {
                    "action": "transfer",
                    "token": "GTBTC",
                    "to": args.to,
                    "amount_gtbtc": args.amount,
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
                "action": "transfer",
                "token": "GTBTC",
                "from": from,
                "to": args.to,
                "amount_gtbtc": args.amount,
                "amount_atomic": atomic.to_string(),
                "chain_id": args.chain_id,
                "tx_hash": tx_hash,
                "raw": result
            }
        })
    );
    Ok(())
}

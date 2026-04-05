use crate::config::{atomic_to_gtbtc, SOLANA_CHAIN_ID};
use crate::rpc;

pub struct BalanceArgs {
    pub address: Option<String>,
    pub chain_id: u64,
}

pub async fn run(args: &BalanceArgs) -> anyhow::Result<()> {
    // Determine wallet address
    let address = match &args.address {
        Some(a) => a.clone(),
        None => {
            if args.chain_id == SOLANA_CHAIN_ID {
                let addr = crate::onchainos::resolve_wallet_solana()?;
                if addr.is_empty() {
                    anyhow::bail!("Cannot resolve Solana wallet address. Ensure onchainos is logged in.");
                }
                addr
            } else {
                let addr = crate::onchainos::resolve_wallet(args.chain_id)?;
                if addr.is_empty() {
                    anyhow::bail!(
                        "Cannot resolve wallet address for chain {}. Ensure onchainos is logged in.",
                        args.chain_id
                    );
                }
                addr
            }
        }
    };

    let (atomic, chain_label) = if args.chain_id == SOLANA_CHAIN_ID {
        let bal = rpc::get_solana_balance(&address).await?;
        (bal, "solana")
    } else {
        let bal = rpc::get_evm_balance(&address, args.chain_id).await?;
        let label = match args.chain_id {
            1 => "ethereum",
            56 => "bsc",
            8453 => "base",
            _ => "evm",
        };
        (bal, label)
    };

    let human = atomic_to_gtbtc(atomic);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "data": {
                "address": address,
                "chain": chain_label,
                "chain_id": args.chain_id,
                "balance_atomic": atomic.to_string(),
                "balance_gtbtc": format!("{:.8}", human),
                "token": "GTBTC",
                "decimals": 8,
                "contract": crate::config::GTBTC_TOKEN_ADDRESS
            }
        })
    );
    Ok(())
}

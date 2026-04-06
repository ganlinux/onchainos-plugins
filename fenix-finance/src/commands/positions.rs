use clap::Args;

use crate::config::NFPM;
use crate::{api, onchainos, rpc};

#[derive(Args, Debug)]
pub struct PositionsArgs {
    /// Wallet address to query (defaults to logged-in wallet)
    #[arg(long)]
    pub owner: Option<String>,

    /// Use on-chain query instead of subgraph
    #[arg(long)]
    pub onchain: bool,
}

pub async fn execute(args: &PositionsArgs, chain_id: u64) -> anyhow::Result<()> {
    let owner = match &args.owner {
        Some(o) => o.clone(),
        None => {
            let w = onchainos::resolve_wallet(chain_id)?;
            if w.is_empty() {
                anyhow::bail!("Cannot resolve wallet. Pass --owner or ensure onchainos is logged in.");
            }
            w
        }
    };

    if !args.onchain {
        // Try subgraph first
        match api::get_user_positions(&owner).await {
            Ok(positions) => {
                if positions.is_empty() {
                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "owner": owner,
                            "positions": [],
                            "count": 0
                        })
                    );
                    return Ok(());
                }
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "owner": owner,
                        "positions": positions,
                        "count": positions.len(),
                        "source": "subgraph"
                    })
                );
                return Ok(());
            }
            Err(e) => {
                eprintln!("Subgraph query failed ({}), falling back to on-chain", e);
            }
        }
    }

    // On-chain fallback
    let count = rpc::nfpm_balance_of(NFPM, &owner).await?;
    if count == 0 {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "owner": owner,
                "positions": [],
                "count": 0,
                "source": "onchain"
            })
        );
        return Ok(());
    }

    let mut positions_out = Vec::new();
    for i in 0..count {
        let token_id = rpc::nfpm_token_of_owner_by_index(NFPM, &owner, i).await?;
        match rpc::nfpm_positions(NFPM, token_id).await {
            Ok(pos) => {
                positions_out.push(serde_json::json!({
                    "token_id": token_id,
                    "token0": pos.token0,
                    "token1": pos.token1,
                    "tick_lower": pos.tick_lower,
                    "tick_upper": pos.tick_upper,
                    "liquidity": pos.liquidity.to_string(),
                    "tokens_owed0": pos.tokens_owed0.to_string(),
                    "tokens_owed1": pos.tokens_owed1.to_string()
                }));
            }
            Err(e) => {
                positions_out.push(serde_json::json!({
                    "token_id": token_id,
                    "error": e.to_string()
                }));
            }
        }
    }

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "owner": owner,
            "positions": positions_out,
            "count": positions_out.len(),
            "source": "onchain"
        })
    );
    Ok(())
}

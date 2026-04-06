use anyhow::Context;
use clap::Args;

use crate::config::NFPM;
use crate::{onchainos, rpc};

#[derive(Args, Debug)]
pub struct RemoveLiquidityArgs {
    /// LP NFT token ID
    #[arg(long)]
    pub token_id: u64,

    /// Override sender address
    #[arg(long)]
    pub from: Option<String>,

    /// Deadline in seconds from now (default 300)
    #[arg(long, default_value = "300")]
    pub deadline_secs: u64,
}

pub async fn execute(args: &RemoveLiquidityArgs, dry_run: bool, chain_id: u64) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "description": "Would execute decreaseLiquidity + collect on Fenix NFPM",
                "token_id": args.token_id,
                "nfpm": NFPM,
                "selectors": {
                    "decreaseLiquidity": "0x0c49ccbe",
                    "collect": "0xfc6f7865"
                }
            })
        );
        return Ok(());
    }

    // Resolve wallet
    let wallet = args
        .from
        .clone()
        .unwrap_or_else(|| onchainos::resolve_wallet(chain_id).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Pass --from or ensure onchainos is logged in.");
    }

    // Fetch position data
    let pos = rpc::nfpm_positions(NFPM, args.token_id)
        .await
        .context("fetch position")?;

    let deadline = rpc::get_block_timestamp().await.unwrap_or(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    ) + args.deadline_secs;

    let mut decrease_tx = None;

    if pos.liquidity > 0 {
        // Build decreaseLiquidity calldata
        // decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))
        // Selector: 0x0c49ccbe
        let calldata = format!(
            "0x0c49ccbe{:064x}{:064x}{:064x}{:064x}{:064x}",
            args.token_id,
            pos.liquidity,
            0u128, // amount0Min = 0
            0u128, // amount1Min = 0
            deadline
        );

        let result = onchainos::wallet_contract_call(
            chain_id,
            NFPM,
            &calldata,
            Some(&wallet),
            None,
            true,
            false,
        )
        .await
        .context("decreaseLiquidity")?;

        decrease_tx = Some(onchainos::extract_tx_hash(&result));
        eprintln!("decreaseLiquidity tx: {:?}", decrease_tx);

        // Wait 5 seconds before collect
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    } else {
        eprintln!("Position has no liquidity; proceeding to collect fees only.");
    }

    // Build collect calldata
    // collect((uint256,address,uint128,uint128))
    // Selector: 0xfc6f7865
    let recipient_clean = wallet.trim_start_matches("0x");
    let collect_calldata = format!(
        "0xfc6f7865{:064x}{:0>64}{:064x}{:064x}",
        args.token_id,
        recipient_clean,
        u128::MAX, // amount0Max
        u128::MAX  // amount1Max
    );

    let collect_result = onchainos::wallet_contract_call(
        chain_id,
        NFPM,
        &collect_calldata,
        Some(&wallet),
        None,
        true,
        false,
    )
    .await
    .context("collect")?;

    let collect_tx = onchainos::extract_tx_hash(&collect_result);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "token_id": args.token_id,
            "decrease_liquidity_tx": decrease_tx,
            "collect_tx": collect_tx,
            "token0": pos.token0,
            "token1": pos.token1,
            "liquidity_removed": pos.liquidity.to_string(),
            "nfpm": NFPM
        })
    );
    Ok(())
}

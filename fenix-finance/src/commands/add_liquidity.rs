use anyhow::Context;
use clap::Args;

use crate::config::{self, NFPM};
use crate::{onchainos, rpc};

#[derive(Args, Debug)]
pub struct AddLiquidityArgs {
    /// First token (symbol or address)
    #[arg(long)]
    pub token0: String,

    /// Second token (symbol or address)
    #[arg(long)]
    pub token1: String,

    /// Desired amount of token0 (human-readable, e.g. "100")
    #[arg(long)]
    pub amount0: String,

    /// Desired amount of token1 (human-readable, e.g. "0.05")
    #[arg(long)]
    pub amount1: String,

    /// Tick lower bound (default -887220 for full range)
    #[arg(long, default_value = "-887220", allow_hyphen_values = true)]
    pub tick_lower: i32,

    /// Tick upper bound (default 887220 for full range)
    #[arg(long, default_value = "887220", allow_hyphen_values = true)]
    pub tick_upper: i32,

    /// Slippage in basis points (500 = 5%, default 50 = 0.5%)
    #[arg(long, default_value = "50")]
    pub slippage_bps: u32,

    /// Deadline in seconds from now (default 300)
    #[arg(long, default_value = "300")]
    pub deadline_secs: u64,

    /// Override sender address
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &AddLiquidityArgs, dry_run: bool, chain_id: u64) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "description": "Would mint new LP position on Fenix NFPM",
                "token0": args.token0,
                "token1": args.token1,
                "amount0": args.amount0,
                "amount1": args.amount1,
                "tick_lower": args.tick_lower,
                "tick_upper": args.tick_upper,
                "nfpm": NFPM,
                "selector": "0x9cc1a283"
            })
        );
        return Ok(());
    }

    let addr0 = config::resolve_token_address(&args.token0);
    let addr1 = config::resolve_token_address(&args.token1);
    let dec0 = config::resolve_token_decimals(&args.token0);
    let dec1 = config::resolve_token_decimals(&args.token1);

    // Ensure token0 < token1 (address ordering required by NFPM)
    let (token0_addr, token1_addr, amount0_str, amount1_str, dec0_final, dec1_final) =
        if addr0.to_lowercase() < addr1.to_lowercase() {
            (addr0, addr1, &args.amount0, &args.amount1, dec0, dec1)
        } else {
            (addr1, addr0, &args.amount1, &args.amount0, dec1, dec0)
        };

    let amount0_f: f64 = amount0_str.parse().context("invalid amount0")?;
    let amount1_f: f64 = amount1_str.parse().context("invalid amount1")?;
    let amount0_desired = (amount0_f * 10f64.powi(dec0_final as i32)) as u128;
    let amount1_desired = (amount1_f * 10f64.powi(dec1_final as i32)) as u128;
    let amount0_min = amount0_desired * (10000 - args.slippage_bps as u128) / 10000;
    let amount1_min = amount1_desired * (10000 - args.slippage_bps as u128) / 10000;

    // Resolve wallet
    let wallet = args
        .from
        .clone()
        .unwrap_or_else(|| onchainos::resolve_wallet(chain_id).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Pass --from or ensure onchainos is logged in.");
    }

    // Approve token0 -> NFPM
    let allowance0 = rpc::erc20_allowance(&token0_addr, &wallet, NFPM).await?;
    if allowance0 < amount0_desired {
        eprintln!("Approving token0 for NFPM...");
        let r0 = onchainos::erc20_approve(chain_id, &token0_addr, NFPM, u128::MAX, Some(&wallet), false)
            .await
            .context("approve token0")?;
        if r0["ok"].as_bool() != Some(true) {
            anyhow::bail!("approve token0 failed: {}", r0["error"].as_str().unwrap_or("unknown error"));
        }
        eprintln!("Approve token0 tx: {}", onchainos::extract_tx_hash(&r0));
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    // Approve token1 -> NFPM
    let allowance1 = rpc::erc20_allowance(&token1_addr, &wallet, NFPM).await?;
    if allowance1 < amount1_desired {
        eprintln!("Approving token1 for NFPM...");
        let r1 = onchainos::erc20_approve(chain_id, &token1_addr, NFPM, u128::MAX, Some(&wallet), false)
            .await
            .context("approve token1")?;
        if r1["ok"].as_bool() != Some(true) {
            anyhow::bail!("approve token1 failed: {}", r1["error"].as_str().unwrap_or("unknown error"));
        }
        eprintln!("Approve token1 tx: {}", onchainos::extract_tx_hash(&r1));
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    // Build mint calldata
    // mint((address,address,int24,int24,uint256,uint256,uint256,uint256,address,uint256))
    // Selector: 0x9cc1a283
    let deadline = rpc::get_block_timestamp().await.unwrap_or(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    ) + args.deadline_secs;

    let t0_clean = token0_addr.trim_start_matches("0x");
    let t1_clean = token1_addr.trim_start_matches("0x");
    let recipient_clean = wallet.trim_start_matches("0x");

    // int24 tick values encoded as 32-byte ABI (int256, two's complement)
    let tick_lower_enc = encode_int256(args.tick_lower as i128);
    let tick_upper_enc = encode_int256(args.tick_upper as i128);

    let calldata = format!(
        "0x9cc1a283{:0>64}{:0>64}{}{}{:064x}{:064x}{:064x}{:064x}{:0>64}{:064x}",
        t0_clean,
        t1_clean,
        tick_lower_enc,
        tick_upper_enc,
        amount0_desired,
        amount1_desired,
        amount0_min,
        amount1_min,
        recipient_clean,
        deadline
    );

    let result = onchainos::wallet_contract_call_gas(
        chain_id,
        NFPM,
        &calldata,
        Some(&wallet),
        None,
        true,
        Some(600_000),
        false,
    )
    .await
    .context("wallet contract-call mint")?;

    if result["ok"].as_bool() != Some(true) {
        anyhow::bail!("mint failed: {}", result["error"].as_str().unwrap_or("unknown error"));
    }
    let tx_hash = onchainos::extract_tx_hash(&result);
    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "tx_hash": tx_hash,
            "token0": token0_addr,
            "token1": token1_addr,
            "amount0_desired": amount0_desired.to_string(),
            "amount1_desired": amount1_desired.to_string(),
            "tick_lower": args.tick_lower,
            "tick_upper": args.tick_upper,
            "nfpm": NFPM
        })
    );
    Ok(())
}

/// Encode i128 as ABI int256 (32 bytes, two's complement).
fn encode_int256(val: i128) -> String {
    // Two's complement: negative values have all upper bits set.
    let unsigned = val as u128; // wraps correctly for negative values
    if val >= 0 {
        format!("{:064x}", unsigned)
    } else {
        // Upper 128 bits (as int256) are all 0xFF
        format!("ffffffffffffffffffffffffffffffff{:032x}", unsigned)
    }
}

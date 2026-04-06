use anyhow::Context;
use clap::Args;

use crate::config::{self, ALGEBRA_FACTORY, QUOTER_V2, SWAP_ROUTER};
use crate::{onchainos, rpc};

#[derive(Args, Debug)]
pub struct SwapArgs {
    /// Input token (symbol like WETH, USDB, FNX, or raw address)
    #[arg(long)]
    pub token_in: String,

    /// Output token (symbol or address)
    #[arg(long)]
    pub token_out: String,

    /// Human-readable amount to swap (e.g. "0.1")
    #[arg(long)]
    pub amount: String,

    /// Slippage in basis points (100 = 1%, default 50 = 0.5%)
    #[arg(long, default_value = "50")]
    pub slippage_bps: u32,

    /// Deadline in seconds from now (default 300)
    #[arg(long, default_value = "300")]
    pub deadline_secs: u64,

    /// Override sender address (wallet address)
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(args: &SwapArgs, dry_run: bool, chain_id: u64) -> anyhow::Result<()> {
    if dry_run {
        println!(
            "{}",
            serde_json::json!({
                "ok": true,
                "dry_run": true,
                "description": "Would execute exactInputSingle on Fenix SwapRouter",
                "token_in": args.token_in,
                "token_out": args.token_out,
                "amount": args.amount,
                "slippage_bps": args.slippage_bps,
                "swap_router": SWAP_ROUTER,
                "selector": "0xbc651188"
            })
        );
        return Ok(());
    }

    let token_in_addr = config::resolve_token_address(&args.token_in);
    let token_out_addr = config::resolve_token_address(&args.token_out);
    let decimals_in = config::resolve_token_decimals(&args.token_in);

    let amount_f: f64 = args.amount.parse().context("invalid amount")?;
    let amount_in = (amount_f * 10f64.powi(decimals_in as i32)) as u128;

    // Step 1: Verify pool exists
    let pool_addr = rpc::factory_pool_by_pair(ALGEBRA_FACTORY, &token_in_addr, &token_out_addr)
        .await
        .context("factory_pool_by_pair")?;
    if pool_addr == "0x0000000000000000000000000000000000000000" {
        anyhow::bail!("Pool does not exist for this pair");
    }

    // Step 2: Get quote
    let amount_out = rpc::quoter_quote_exact_input_single(
        QUOTER_V2,
        &token_in_addr,
        &token_out_addr,
        amount_in,
    )
    .await
    .context("quoteExactInputSingle")?;

    let amount_out_minimum = amount_out * (10000 - args.slippage_bps as u128) / 10000;

    // Step 3: Resolve wallet
    let wallet = args
        .from
        .clone()
        .unwrap_or_else(|| onchainos::resolve_wallet(chain_id).unwrap_or_default());
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Pass --from or ensure onchainos is logged in.");
    }

    // Step 4: Check and approve if needed
    let allowance = rpc::erc20_allowance(&token_in_addr, &wallet, SWAP_ROUTER)
        .await
        .context("allowance check")?;
    if allowance < amount_in {
        eprintln!("Approving {} for SwapRouter...", args.token_in);
        let approve_result = onchainos::erc20_approve(
            chain_id,
            &token_in_addr,
            SWAP_ROUTER,
            u128::MAX,
            Some(&wallet),
            false,
        )
        .await
        .context("erc20_approve")?;
        let approve_hash = onchainos::extract_tx_hash(&approve_result);
        eprintln!("Approve tx: {}", approve_hash);
        // Wait 3 seconds after approve
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

    // Step 5: Build exactInputSingle calldata
    // ExactInputSingleParams: (tokenIn, tokenOut, recipient, deadline, amountIn, amountOutMinimum, limitSqrtPrice)
    // selector: 0xbc651188
    let deadline = rpc::get_block_timestamp().await.unwrap_or(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    ) + args.deadline_secs;

    let token_in_clean = token_in_addr.trim_start_matches("0x");
    let token_out_clean = token_out_addr.trim_start_matches("0x");
    let recipient_clean = wallet.trim_start_matches("0x");

    let calldata = format!(
        "0xbc651188{:0>64}{:0>64}{:0>64}{:064x}{:064x}{:064x}{:064x}",
        token_in_clean,
        token_out_clean,
        recipient_clean,
        deadline,
        amount_in,
        amount_out_minimum,
        0u128 // limitSqrtPrice = 0
    );

    // Step 6: Execute swap (requires --force for DEX operations)
    let result = onchainos::wallet_contract_call(
        chain_id,
        SWAP_ROUTER,
        &calldata,
        Some(&wallet),
        None,
        true, // --force required for DEX swap
        false,
    )
    .await
    .context("wallet contract-call swap")?;

    let tx_hash = onchainos::extract_tx_hash(&result);
    let decimals_out = config::resolve_token_decimals(&args.token_out);
    let amount_out_human = amount_out as f64 / 10f64.powi(decimals_out as i32);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "tx_hash": tx_hash,
            "token_in": args.token_in,
            "token_out": args.token_out,
            "amount_in": args.amount,
            "estimated_amount_out": format!("{:.6}", amount_out_human),
            "amount_out_minimum_raw": amount_out_minimum.to_string(),
            "slippage_bps": args.slippage_bps,
            "swap_router": SWAP_ROUTER
        })
    );
    Ok(())
}

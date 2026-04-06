use anyhow::Context;
use clap::Args;

use crate::config::{self, QUOTER_V2};
use crate::rpc;

#[derive(Args, Debug)]
pub struct PriceArgs {
    /// Input token (symbol like WETH, USDB, FNX, or raw address)
    #[arg(long)]
    pub token_in: String,

    /// Output token (symbol or address)
    #[arg(long)]
    pub token_out: String,

    /// Human-readable amount to quote (e.g. "1.0")
    #[arg(long, default_value = "1")]
    pub amount: String,
}

pub async fn execute(args: &PriceArgs) -> anyhow::Result<()> {
    let token_in_addr = config::resolve_token_address(&args.token_in);
    let token_out_addr = config::resolve_token_address(&args.token_out);
    let decimals_in = config::resolve_token_decimals(&args.token_in);

    let amount_f: f64 = args.amount.parse().context("invalid amount")?;
    let amount_in = (amount_f * 10f64.powi(decimals_in as i32)) as u128;

    // Verify pool exists
    let pool_addr = rpc::factory_pool_by_pair(
        config::ALGEBRA_FACTORY,
        &token_in_addr,
        &token_out_addr,
    )
    .await?;
    if pool_addr == "0x0000000000000000000000000000000000000000" {
        println!(
            "{}",
            serde_json::json!({
                "ok": false,
                "error": "Pool does not exist for this pair"
            })
        );
        return Ok(());
    }

    let amount_out = rpc::quoter_quote_exact_input_single(
        QUOTER_V2,
        &token_in_addr,
        &token_out_addr,
        amount_in,
    )
    .await?;

    let decimals_out = config::resolve_token_decimals(&args.token_out);
    let amount_out_human = amount_out as f64 / 10f64.powi(decimals_out as i32);

    println!(
        "{}",
        serde_json::json!({
            "ok": true,
            "token_in": args.token_in,
            "token_out": args.token_out,
            "amount_in": args.amount,
            "amount_out_raw": amount_out.to_string(),
            "amount_out_human": format!("{:.6}", amount_out_human),
            "pool": pool_addr
        })
    );
    Ok(())
}

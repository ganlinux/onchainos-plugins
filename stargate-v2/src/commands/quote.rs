use anyhow::Context;
use clap::Args;

use crate::config::{
    chain_name, default_rpc, format_amount, parse_amount, parse_chain, pool_address, token_decimals,
    chain_id_to_eid,
};
use crate::rpc::{address_to_bytes32, quote_oft, quote_send};

#[derive(Args, Debug)]
pub struct QuoteArgs {
    /// Source chain (name or chain ID, e.g. "ethereum", "arbitrum", 42161)
    #[arg(long)]
    pub src_chain: String,

    /// Destination chain (name or chain ID)
    #[arg(long)]
    pub dst_chain: String,

    /// Token symbol: ETH, USDC, USDT
    #[arg(long)]
    pub token: String,

    /// Amount to bridge (human-readable, e.g. "100.5")
    #[arg(long)]
    pub amount: String,

    /// Receiver address on destination chain (default: same as sender)
    #[arg(long)]
    pub receiver: Option<String>,

    /// Transfer mode: taxi (fast) or bus (cheap)
    #[arg(long, default_value = "taxi")]
    pub mode: String,

    /// RPC endpoint for source chain (optional, uses public default if not set)
    #[arg(long)]
    pub rpc: Option<String>,
}

pub async fn run(args: QuoteArgs) -> anyhow::Result<()> {
    let src_chain_id = parse_chain(&args.src_chain)?;
    let dst_chain_id = parse_chain(&args.dst_chain)?;
    let token = args.token.to_uppercase();
    let decimals = token_decimals(&token);
    let amount_ld = parse_amount(&args.amount, decimals)
        .context("Failed to parse amount")?;

    let dst_eid = chain_id_to_eid(dst_chain_id)
        .ok_or_else(|| anyhow::anyhow!("Unsupported destination chain: {}", args.dst_chain))?;

    let (pool_addr, _is_native) = pool_address(src_chain_id, &token)
        .ok_or_else(|| anyhow::anyhow!("No Stargate pool for {} on chain {}", token, src_chain_id))?;

    let rpc_url = args.rpc.as_deref().unwrap_or_else(|| default_rpc(src_chain_id));
    let bus_mode = args.mode.to_lowercase() == "bus";

    // Use a placeholder receiver for quote (doesn't affect fees significantly)
    let receiver_raw = args
        .receiver
        .as_deref()
        .unwrap_or("0x0000000000000000000000000000000000000001");
    let to_bytes32 = address_to_bytes32(receiver_raw);

    println!(
        "Querying quote: {} {} from {} -> {} (mode: {})",
        args.amount,
        token,
        chain_name(src_chain_id),
        chain_name(dst_chain_id),
        args.mode
    );

    // Step 1: quoteOFT
    let (amount_sent_ld, amount_received_ld) = quote_oft(
        rpc_url,
        pool_addr,
        dst_eid,
        &to_bytes32,
        amount_ld,
        bus_mode,
    )
    .await
    .context("quoteOFT failed")?;

    // Step 2: quoteSend with updated min_amount
    let (native_fee, lz_token_fee) = quote_send(
        rpc_url,
        pool_addr,
        dst_eid,
        &to_bytes32,
        amount_ld,
        amount_received_ld,
        bus_mode,
    )
    .await
    .context("quoteSend failed")?;

    println!();
    println!("=== Stargate V2 Quote ===");
    println!("  Source chain   : {} (chain ID {})", chain_name(src_chain_id), src_chain_id);
    println!("  Destination    : {} (chain ID {}, EID {})", chain_name(dst_chain_id), dst_chain_id, dst_eid);
    println!("  Token          : {}", token);
    println!("  Send amount    : {} {}", args.amount, token);
    println!(
        "  Amount sent    : {} {}",
        format_amount(amount_sent_ld, decimals),
        token
    );
    println!(
        "  Amount received: {} {}  (after protocol fee)",
        format_amount(amount_received_ld, decimals),
        token
    );
    println!(
        "  LayerZero fee  : {} wei (native)",
        native_fee
    );
    if lz_token_fee > 0 {
        println!("  LZ token fee   : {} wei", lz_token_fee);
    }
    println!("  Mode           : {} ({})",
        args.mode,
        if bus_mode { "batch, cheaper, ~5-20 min" } else { "immediate, ~1-3 min" }
    );

    Ok(())
}

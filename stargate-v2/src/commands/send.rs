use anyhow::Context;
use clap::Args;

use crate::config::{
    chain_name, default_rpc, format_amount, parse_amount, parse_chain, pool_address, token_decimals,
    chain_id_to_eid,
};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{address_to_bytes32, encode_send_token, get_allowance, get_pool_token, quote_oft, quote_send};

#[derive(Args, Debug)]
pub struct SendArgs {
    /// Source chain (name or chain ID)
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

    /// Transfer mode: taxi (fast, default) or bus (cheap)
    #[arg(long, default_value = "taxi")]
    pub mode: String,

    /// Slippage tolerance in basis points (1 bps = 0.01%), default 50 = 0.5%
    #[arg(long, default_value = "50")]
    pub slippage_bps: u32,

    /// RPC endpoint for source chain (optional)
    #[arg(long)]
    pub rpc: Option<String>,

    /// Dry run: build and print calldata without submitting
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn run(args: SendArgs) -> anyhow::Result<()> {
    let src_chain_id = parse_chain(&args.src_chain)?;
    let dst_chain_id = parse_chain(&args.dst_chain)?;
    let token = args.token.to_uppercase();
    let decimals = token_decimals(&token);
    let amount_ld = parse_amount(&args.amount, decimals)
        .context("Failed to parse amount")?;

    let dst_eid = chain_id_to_eid(dst_chain_id)
        .ok_or_else(|| anyhow::anyhow!("Unsupported destination chain: {}", args.dst_chain))?;

    let (pool_addr, is_native) = pool_address(src_chain_id, &token)
        .ok_or_else(|| anyhow::anyhow!("No Stargate pool for {} on chain {}", token, src_chain_id))?;

    let rpc_url = args.rpc.as_deref().unwrap_or_else(|| default_rpc(src_chain_id));
    let bus_mode = args.mode.to_lowercase() == "bus";

    // Resolve sender wallet address
    let sender = if args.dry_run {
        "0x0000000000000000000000000000000000000001".to_string()
    } else {
        resolve_wallet(src_chain_id).context("Failed to resolve wallet address")?
    };

    let receiver = args
        .receiver
        .as_deref()
        .unwrap_or(&sender)
        .to_string();
    let to_bytes32 = address_to_bytes32(&receiver);

    println!(
        "Preparing to bridge {} {} from {} -> {}",
        args.amount,
        token,
        chain_name(src_chain_id),
        chain_name(dst_chain_id)
    );

    // Step 1: quoteOFT to get expected received amount
    println!("  [1/4] Querying quoteOFT...");
    let (_amount_sent_ld, amount_received_ld) = quote_oft(
        rpc_url,
        pool_addr,
        dst_eid,
        &to_bytes32,
        amount_ld,
        bus_mode,
    )
    .await
    .context("quoteOFT failed")?;

    // Apply slippage to minAmountLD
    let min_amount_ld = amount_received_ld
        .saturating_mul(10_000 - args.slippage_bps as u128)
        / 10_000;

    println!(
        "     Expected received: {} {} (min with slippage: {})",
        format_amount(amount_received_ld, decimals),
        token,
        format_amount(min_amount_ld, decimals)
    );

    // Step 2: quoteSend to get LayerZero messaging fee
    println!("  [2/4] Querying quoteSend...");
    let (native_fee, _lz_token_fee) = quote_send(
        rpc_url,
        pool_addr,
        dst_eid,
        &to_bytes32,
        amount_ld,
        min_amount_ld,
        bus_mode,
    )
    .await
    .context("quoteSend failed")?;

    println!("     LayerZero messaging fee: {} wei", native_fee);

    // Step 3: ERC-20 approve if needed
    if !is_native {
        println!("  [3/4] Checking ERC-20 allowance...");
        let token_addr = get_pool_token(rpc_url, pool_addr)
            .await
            .context("Failed to get pool token address")?;

        let zero_addr = "0x0000000000000000000000000000000000000000";
        if token_addr.to_lowercase() == zero_addr {
            // Shouldn't happen for non-native pools, but be safe
            println!("     Pool reports native token, skipping approve.");
        } else {
            let allowance = get_allowance(rpc_url, &token_addr, &sender, pool_addr)
                .await
                .context("Failed to check allowance")?;

            if allowance < amount_ld {
                println!(
                    "     Allowance {} < required {}, executing approve...",
                    allowance, amount_ld
                );

                // ask user to confirm the ERC-20 approve transaction
                println!(
                    "     ACTION REQUIRED: Please confirm the ERC-20 approve transaction:"
                );
                println!("       Token:   {}", token_addr);
                println!("       Spender: {}", pool_addr);
                println!("       Amount:  {} {} ({})", args.amount, token, amount_ld);

                let approve_result = erc20_approve(
                    src_chain_id,
                    &token_addr,
                    pool_addr,
                    amount_ld,
                    Some(&sender),
                    args.dry_run,
                )
                .await
                .context("ERC-20 approve failed")?;

                let approve_hash = extract_tx_hash(&approve_result);
                println!("     Approve tx: {}", approve_hash);
            } else {
                println!("     Allowance sufficient ({} >= {}), skipping approve.", allowance, amount_ld);
            }
        }
    } else {
        println!("  [3/4] Native token pool — skipping ERC-20 approve.");
    }

    // Step 4: Execute sendToken
    println!("  [4/4] Executing sendToken...");

    // msg.value: native pool = nativeFee + amountLD, ERC-20 pool = nativeFee only
    let msg_value: u128 = if is_native {
        native_fee + amount_ld
    } else {
        native_fee
    };

    let calldata = encode_send_token(
        dst_eid,
        &to_bytes32,
        amount_ld,
        min_amount_ld,
        native_fee,
        &sender,
        bus_mode,
    );

    println!();
    println!("=== sendToken Transaction Summary ===");
    println!("  Pool contract  : {}", pool_addr);
    println!("  Source chain   : {} ({})", chain_name(src_chain_id), src_chain_id);
    println!("  Destination    : {} (EID {})", chain_name(dst_chain_id), dst_eid);
    println!("  Receiver       : {}", receiver);
    println!("  Send amount    : {} {}", args.amount, token);
    println!("  Min received   : {} {}", format_amount(min_amount_ld, decimals), token);
    println!("  Native fee     : {} wei", native_fee);
    println!("  msg.value      : {} wei", msg_value);
    println!("  Mode           : {}", args.mode);
    println!("  Calldata       : {}", &calldata[..std::cmp::min(calldata.len(), 80)]);

    // ask user to confirm the cross-chain sendToken transaction before submitting
    println!();
    println!("ACTION REQUIRED: Please confirm the cross-chain bridge transaction:");
    println!("  This will send {} {} from {} to {}.", args.amount, token, chain_name(src_chain_id), chain_name(dst_chain_id));
    println!("  msg.value = {} wei (includes LayerZero fee{})",
        msg_value,
        if is_native { " + bridge amount" } else { "" }
    );

    let result = wallet_contract_call(
        src_chain_id,
        pool_addr,
        &calldata,
        Some(&sender),
        Some(msg_value),
        args.dry_run,
    )
    .await
    .context("sendToken contract call failed")?;

    let tx_hash = extract_tx_hash(&result);
    println!();
    println!("=== Transaction Submitted ===");
    println!("  tx hash: {}", tx_hash);
    println!();
    println!(
        "Track status: stargate-v2 status --tx-hash {}",
        tx_hash
    );
    if bus_mode {
        println!("  Note: Bus mode — expect ~5-20 minutes for batch aggregation.");
    } else {
        println!("  Note: Taxi mode — expect ~1-3 minutes for delivery.");
    }

    Ok(())
}

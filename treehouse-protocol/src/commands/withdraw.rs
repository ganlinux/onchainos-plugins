use serde_json::json;

use crate::config::{
    parse_18, AVAX_CHAIN_ID, ETH_CHAIN_ID, CURVE_POOL, TETH_TOKEN, SEL_EXCHANGE,
};
use crate::onchainos;
use crate::rpc;

/// Withdraw (redeem) tETH → wstETH via Curve StableSwap pool (Ethereum only).
///
/// Limited to small redemptions (<=200 wstETH equivalent).
/// Avalanche tAVAX redemption is NOT supported in this version.
pub async fn run(
    chain_id: u64,
    amount: &str,
    slippage_bps: u32,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    match chain_id {
        ETH_CHAIN_ID => withdraw_teth(amount, slippage_bps, from, dry_run).await,
        AVAX_CHAIN_ID => {
            anyhow::bail!(
                "tAVAX withdrawal is not supported in this version. \
                 tAVAX redemption requires a 7-day waiting period via the standard \
                 redemption flow, which is not implemented here."
            )
        }
        _ => anyhow::bail!(
            "Unsupported chain_id: {}. Withdraw is only available on Ethereum (chain 1).",
            chain_id
        ),
    }
}

async fn withdraw_teth(
    amount: &str,
    slippage_bps: u32,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let amount_wei = parse_18(amount)?;

    // Estimate expected wstETH output via Curve get_dy(0, 1, dx)
    // tETH = coin[0], wstETH = coin[1]
    let expected_out = rpc::curve_get_dy(CURVE_POOL, 0, 1, amount_wei)
        .await
        .unwrap_or(0);

    // Apply slippage protection: min_dy = expected * (10000 - slippage_bps) / 10000
    let min_dy = if expected_out > 0 {
        expected_out * (10000 - slippage_bps as u128) / 10000
    } else {
        // If get_dy failed, use 99% of input as conservative estimate
        amount_wei * 99 / 100
    };

    // Encode exchange(int128,int128,uint256,uint256) calldata
    // i=0 (tETH), j=1 (wstETH), dx=amount_wei, min_dy
    let calldata = encode_exchange(0i128, 1i128, amount_wei, min_dy);

    if dry_run {
        let output = json!({
            "ok": true,
            "dry_run": true,
            "chain_id": ETH_CHAIN_ID,
            "operation": "withdraw_teth_via_curve",
            "amount_teth": amount,
            "amount_teth_raw": amount_wei.to_string(),
            "expected_wsteth": crate::config::format_18(expected_out),
            "min_wsteth": crate::config::format_18(min_dy),
            "slippage_bps": slippage_bps,
            "step1_approve": {
                "to": TETH_TOKEN,
                "note": "Approve tETH to Curve pool",
                "calldata": encode_approve_calldata(CURVE_POOL, amount_wei)
            },
            "step2_exchange": {
                "to": CURVE_POOL,
                "calldata": calldata,
                "note": "exchange(0, 1, tETH_amount, min_wstETH)"
            },
            "warning": "Only suitable for <= 200 wstETH redemptions (Curve Redemption Band)"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(ETH_CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Please log in via onchainos.");
    }
    let sender = from.unwrap_or(&wallet);

    // Step 1: Approve tETH → Curve pool
    eprintln!("Step 1/2: Approving tETH to Curve pool...");
    let approve_result = onchainos::erc20_approve(
        ETH_CHAIN_ID,
        TETH_TOKEN,
        CURVE_POOL,
        amount_wei,
        Some(sender),
        false,
    )
    .await?;
    let approve_tx = onchainos::extract_tx_hash(&approve_result);
    eprintln!("  approve tx: {}", approve_tx);

    // Step 2: Curve exchange
    eprintln!(
        "Step 2/2: Calling exchange(0, 1, {}, {}) on Curve pool...",
        amount,
        crate::config::format_18(min_dy)
    );
    let exchange_result = onchainos::wallet_contract_call(
        ETH_CHAIN_ID,
        CURVE_POOL,
        &calldata,
        Some(sender),
        None,
        false,
    )
    .await?;
    let exchange_tx = onchainos::extract_tx_hash(&exchange_result);

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "ok": true,
            "operation": "withdraw_teth_via_curve",
            "chain_id": ETH_CHAIN_ID,
            "amount_teth": amount,
            "expected_wsteth": crate::config::format_18(expected_out),
            "min_wsteth": crate::config::format_18(min_dy),
            "approve_tx": approve_tx,
            "exchange_tx": exchange_tx,
            "curve_pool": CURVE_POOL,
            "explorer": format!("https://etherscan.io/tx/{}", exchange_tx)
        }))?
    );

    Ok(())
}

/// Encode exchange(int128,int128,uint256,uint256) calldata.
/// Selector: 0x3df02124
fn encode_exchange(i: i128, j: i128, dx: u128, min_dy: u128) -> String {
    let i_enc = encode_int128(i);
    let j_enc = encode_int128(j);
    let dx_enc = format!("{:064x}", dx);
    let min_dy_enc = format!("{:064x}", min_dy);
    format!("{}{}{}{}{}", SEL_EXCHANGE, i_enc, j_enc, dx_enc, min_dy_enc)
}

fn encode_int128(val: i128) -> String {
    if val >= 0 {
        format!("{:064x}", val as u128)
    } else {
        // Two's complement for 32-byte word
        let abs = (-val) as u128;
        let twos = u128::MAX - abs + 1;
        format!("ffffffffffffffffffffffffffffffff{:032x}", twos)
    }
}

fn encode_approve_calldata(spender: &str, amount: u128) -> String {
    let spender_padded = format!(
        "{:0>64}",
        spender.strip_prefix("0x").unwrap_or(spender)
    );
    let amount_hex = format!("{:064x}", amount);
    format!("0x095ea7b3{}{}", spender_padded, amount_hex)
}

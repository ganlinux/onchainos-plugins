/// mint: USDC -> rUSD via Credit Enforcer mintStablecoin
/// Step 1: ERC-20 approve USDC -> Credit Enforcer (6 decimals)
/// Step 2: Credit Enforcer.mintStablecoin(uint256 usdcAmount) selector 0xa0b4dbb1 (6 decimals)
/// NOTE: mintStablecoin parameter is in USDC 6-decimal units, NOT 18 decimals.

use std::time::Duration;
use crate::config::{format_18, format_6, parse_6, CREDIT_ENFORCER, USDC, RPC_URL};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::balance_of;

/// Encode mintStablecoin(uint256) calldata — selector 0xa0b4dbb1
/// amount is in USDC 6-decimal units.
fn encode_mint_stablecoin(usdc_amount_6dec: u128) -> String {
    format!("0xa0b4dbb1{:064x}", usdc_amount_6dec)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    // Parse amount as USDC (6 decimals)
    let usdc_amount_6dec = parse_6(amount)?;
    if usdc_amount_6dec == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    println!("=== Reservoir Protocol — Mint rUSD ===");
    println!("Chain:  1 (Ethereum Mainnet)");
    println!("Input:  {} USDC", format_6(usdc_amount_6dec));
    println!("Output: ~{} rUSD (1:1 via PSM)", format_6(usdc_amount_6dec));
    println!();

    // Resolve wallet
    let wallet = resolve_wallet(1)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on Ethereum mainnet. Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // Check USDC balance
    let usdc_bal = balance_of(RPC_URL, USDC, &wallet).await.unwrap_or(0);
    println!("USDC balance: {} USDC", format_6(usdc_bal));
    if !dry_run && usdc_bal < usdc_amount_6dec {
        anyhow::bail!(
            "Insufficient USDC balance. Have: {} USDC, Need: {} USDC",
            format_6(usdc_bal),
            format_6(usdc_amount_6dec)
        );
    }

    // The rUSD minted will be 1:1 in 18-decimal form
    let rusd_amount_18dec = (usdc_amount_6dec as u128) * 1_000_000_000_000u128;

    println!();
    println!("--- Step 1: Approve USDC -> Credit Enforcer ---");
    println!("  Token:   {} (USDC)", USDC);
    println!("  Spender: {} (Credit Enforcer)", CREDIT_ENFORCER);
    println!("  Amount:  {} USDC ({} raw units, 6 decimals)", format_6(usdc_amount_6dec), usdc_amount_6dec);
    println!();
    println!(">>> Please confirm Step 1: approve {} USDC for Credit Enforcer. Proceed? [y/N]", format_6(usdc_amount_6dec));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let approve_result = erc20_approve(1, USDC, CREDIT_ENFORCER, usdc_amount_6dec, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);

    if !dry_run {
        println!("Waiting 3 seconds before mintStablecoin to avoid nonce conflict...");
        std::thread::sleep(Duration::from_secs(3));
    }

    println!();
    println!("--- Step 2: mintStablecoin ---");
    println!("  Contract: {} (Credit Enforcer)", CREDIT_ENFORCER);
    println!("  Function: mintStablecoin(uint256)  selector: 0xa0b4dbb1");
    println!("  Amount:   {} USDC ({} raw, 6 decimals)", format_6(usdc_amount_6dec), usdc_amount_6dec);
    println!("  Expected rUSD received: ~{} rUSD", format_18(rusd_amount_18dec));
    println!();
    println!(">>> Please confirm Step 2: mint {} rUSD by depositing {} USDC. Proceed? [y/N]", format_18(rusd_amount_18dec), format_6(usdc_amount_6dec));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let calldata = encode_mint_stablecoin(usdc_amount_6dec);
    let result = wallet_contract_call(1, CREDIT_ENFORCER, &calldata, Some(&wallet), None, dry_run).await?;
    let tx_hash = extract_tx_hash(&result);
    println!("mintStablecoin tx: {}", tx_hash);

    if dry_run {
        println!();
        println!("[dry-run] mintStablecoin calldata: {}", calldata);
        println!("[dry-run] Note: mintStablecoin uses USDC 6-decimal units ({} = {} USDC)", usdc_amount_6dec, format_6(usdc_amount_6dec));
    } else {
        println!();
        println!("Mint successful! You received ~{} rUSD.", format_18(rusd_amount_18dec));
        println!("Tip: deposit rUSD into srUSD with: reservoir-protocol save --amount <amount>");
    }

    Ok(())
}

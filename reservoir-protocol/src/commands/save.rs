/// save: rUSD -> srUSD via Credit Enforcer mintSavingcoin
/// Step 1: ERC-20 approve rUSD -> Credit Enforcer (18 decimals)
/// Step 2: Credit Enforcer.mintSavingcoin(uint256 rUSDAmount) selector 0x660cf34e (18 decimals)

use std::time::Duration;
use crate::config::{format_18, parse_18, CREDIT_ENFORCER, RUSD, RPC_URL, SAVING_MODULE};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{balance_of, current_price, preview_mint};

/// Encode mintSavingcoin(uint256) calldata — selector 0x660cf34e
/// amount is in rUSD 18-decimal units.
fn encode_mint_savingcoin(rusd_amount_18dec: u128) -> String {
    format!("0x660cf34e{:064x}", rusd_amount_18dec)
}

pub async fn run(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    // Parse amount as rUSD (18 decimals)
    let rusd_amount = parse_18(amount)?;
    if rusd_amount == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    println!("=== Reservoir Protocol — Save rUSD (mint srUSD) ===");
    println!("Chain:  1 (Ethereum Mainnet)");
    println!("Input:  {} rUSD", format_18(rusd_amount));
    println!();

    // Resolve wallet
    let wallet = resolve_wallet(1)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on Ethereum mainnet. Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // Check rUSD balance
    let rusd_bal = balance_of(RPC_URL, RUSD, &wallet).await.unwrap_or(0);
    println!("rUSD balance: {} rUSD", format_18(rusd_bal));
    if !dry_run && rusd_bal < rusd_amount {
        anyhow::bail!(
            "Insufficient rUSD balance. Have: {} rUSD, Need: {} rUSD",
            format_18(rusd_bal),
            format_18(rusd_amount)
        );
    }

    // Preview: query srUSD exchange rate and expected srUSD received
    let price = current_price(RPC_URL, SAVING_MODULE).await.unwrap_or(0);
    let price_display = price as f64 / 1e8;
    let expected_srusd = preview_mint(RPC_URL, SAVING_MODULE, rusd_amount).await.unwrap_or(0);
    println!("srUSD current price: {:.8} rUSD/srUSD", price_display);
    println!("Expected srUSD received: ~{} srUSD", format_18(expected_srusd));
    println!();

    println!("--- Step 1: Approve rUSD -> Credit Enforcer ---");
    println!("  Token:   {} (rUSD)", RUSD);
    println!("  Spender: {} (Credit Enforcer)", CREDIT_ENFORCER);
    println!("  Amount:  {} rUSD ({} raw units, 18 decimals)", format_18(rusd_amount), rusd_amount);
    println!();
    println!(">>> Please confirm Step 1: approve {} rUSD for Credit Enforcer. Proceed? [y/N]", format_18(rusd_amount));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let approve_result = erc20_approve(1, RUSD, CREDIT_ENFORCER, rusd_amount, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);

    if !dry_run {
        println!("Waiting 3 seconds before mintSavingcoin to avoid nonce conflict...");
        std::thread::sleep(Duration::from_secs(3));
    }

    println!();
    println!("--- Step 2: mintSavingcoin ---");
    println!("  Contract: {} (Credit Enforcer)", CREDIT_ENFORCER);
    println!("  Function: mintSavingcoin(uint256)  selector: 0x660cf34e");
    println!("  Amount:   {} rUSD ({} raw, 18 decimals)", format_18(rusd_amount), rusd_amount);
    println!("  Expected srUSD: ~{}", format_18(expected_srusd));
    println!();
    println!(">>> Please confirm Step 2: deposit {} rUSD to mint ~{} srUSD. Proceed? [y/N]", format_18(rusd_amount), format_18(expected_srusd));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let calldata = encode_mint_savingcoin(rusd_amount);
    let result = wallet_contract_call(1, CREDIT_ENFORCER, &calldata, Some(&wallet), None, dry_run).await?;
    let tx_hash = extract_tx_hash(&result);
    println!("mintSavingcoin tx: {}", tx_hash);

    if dry_run {
        println!();
        println!("[dry-run] mintSavingcoin calldata: {}", calldata);
        println!("[dry-run] Note: mintSavingcoin uses rUSD 18-decimal units ({} = {} rUSD)", rusd_amount, format_18(rusd_amount));
    } else {
        println!();
        println!("Saving successful! You received ~{} srUSD.", format_18(expected_srusd));
        println!("srUSD accrues yield (~7-8% APY) automatically. Redeem anytime with: reservoir-protocol redeem-srusd --amount <amount>");
    }

    Ok(())
}

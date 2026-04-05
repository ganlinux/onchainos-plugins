/// redeem: two sub-operations
/// 1. redeem-rusd: rUSD -> USDC via PSM.redeem (approve rUSD + PSM.redeem, 18 decimals)
/// 2. redeem-srusd: srUSD -> rUSD via SavingModule.redeem (no approve needed, 18 decimals)

use std::time::Duration;
use crate::config::{format_18, format_6, parse_18, PSM_USDC, RUSD, SAVING_MODULE, SRUSD, RPC_URL};
use crate::onchainos::{erc20_approve, extract_tx_hash, resolve_wallet, wallet_contract_call};
use crate::rpc::{balance_of, preview_redeem, psm_underlying_balance};

/// Encode PSM.redeem(uint256) calldata — selector 0xdb006a75
/// amount is rUSD in 18-decimal units.
fn encode_psm_redeem(rusd_amount_18dec: u128) -> String {
    format!("0xdb006a75{:064x}", rusd_amount_18dec)
}

/// Encode SavingModule.redeem(uint256) calldata — selector 0xdb006a75
/// amount is srUSD in 18-decimal units.
fn encode_saving_redeem(srusd_amount_18dec: u128) -> String {
    format!("0xdb006a75{:064x}", srusd_amount_18dec)
}

/// Redeem rUSD -> USDC via PSM (approve + redeem, 2 steps)
pub async fn run_rusd(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let rusd_amount = parse_18(amount)?;
    if rusd_amount == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }
    // rUSD (18 dec) -> USDC (6 dec): divide by 1e12
    let expected_usdc_6dec = rusd_amount / 1_000_000_000_000u128;

    println!("=== Reservoir Protocol — Redeem rUSD -> USDC ===");
    println!("Chain:  1 (Ethereum Mainnet)");
    println!("Input:  {} rUSD", format_18(rusd_amount));
    println!("Output: ~{} USDC (1:1 via PSM)", format_6(expected_usdc_6dec));
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

    // Check PSM liquidity
    let psm_usdc = psm_underlying_balance(RPC_URL, PSM_USDC).await.unwrap_or(0);
    println!("PSM USDC liquidity: {} USDC", format_6(psm_usdc));
    if !dry_run && psm_usdc < expected_usdc_6dec {
        anyhow::bail!(
            "Insufficient PSM USDC liquidity. Available: {} USDC, Needed: {} USDC",
            format_6(psm_usdc),
            format_6(expected_usdc_6dec)
        );
    }

    println!();
    println!("--- Step 1: Approve rUSD -> PSM (USDC) ---");
    println!("  Token:   {} (rUSD)", RUSD);
    println!("  Spender: {} (PSM USDC)", PSM_USDC);
    println!("  Amount:  {} rUSD ({} raw, 18 decimals)", format_18(rusd_amount), rusd_amount);
    println!();
    println!(">>> Please confirm Step 1: approve {} rUSD for PSM. Proceed? [y/N]", format_18(rusd_amount));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let approve_result = erc20_approve(1, RUSD, PSM_USDC, rusd_amount, Some(&wallet), dry_run).await?;
    let approve_hash = extract_tx_hash(&approve_result);
    println!("Approve tx: {}", approve_hash);

    if !dry_run {
        println!("Waiting 3 seconds before PSM redeem to avoid nonce conflict...");
        std::thread::sleep(Duration::from_secs(3));
    }

    println!();
    println!("--- Step 2: PSM redeem ---");
    println!("  Contract: {} (PSM USDC)", PSM_USDC);
    println!("  Function: redeem(uint256)  selector: 0xdb006a75");
    println!("  Amount:   {} rUSD ({} raw, 18 decimals)", format_18(rusd_amount), rusd_amount);
    println!("  Expected USDC: ~{} USDC", format_6(expected_usdc_6dec));
    println!();
    println!(">>> Please confirm Step 2: redeem {} rUSD for ~{} USDC via PSM. Proceed? [y/N]", format_18(rusd_amount), format_6(expected_usdc_6dec));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let calldata = encode_psm_redeem(rusd_amount);
    let result = wallet_contract_call(1, PSM_USDC, &calldata, Some(&wallet), None, dry_run).await?;
    let tx_hash = extract_tx_hash(&result);
    println!("PSM redeem tx: {}", tx_hash);

    if dry_run {
        println!();
        println!("[dry-run] PSM redeem calldata: {}", calldata);
        println!("[dry-run] Note: PSM.redeem uses rUSD 18-decimal units ({} = {} rUSD)", rusd_amount, format_18(rusd_amount));
    } else {
        println!();
        println!("Redeem successful! You received ~{} USDC.", format_6(expected_usdc_6dec));
    }

    Ok(())
}

/// Redeem srUSD -> rUSD via SavingModule (no approve needed — contract burns caller's srUSD directly)
pub async fn run_srusd(amount: &str, dry_run: bool) -> anyhow::Result<()> {
    let srusd_amount = parse_18(amount)?;
    if srusd_amount == 0 {
        anyhow::bail!("Amount must be greater than 0");
    }

    println!("=== Reservoir Protocol — Redeem srUSD -> rUSD ===");
    println!("Chain:  1 (Ethereum Mainnet)");
    println!("Input:  {} srUSD", format_18(srusd_amount));
    println!();

    // Resolve wallet
    let wallet = resolve_wallet(1)?;
    if wallet.is_empty() {
        anyhow::bail!("No wallet found on Ethereum mainnet. Run: onchainos wallet login");
    }
    println!("Wallet: {}", wallet);

    // Check srUSD balance
    let srusd_bal = balance_of(RPC_URL, SRUSD, &wallet).await.unwrap_or(0);
    println!("srUSD balance: {} srUSD", format_18(srusd_bal));
    if !dry_run && srusd_bal < srusd_amount {
        anyhow::bail!(
            "Insufficient srUSD balance. Have: {} srUSD, Need: {} srUSD",
            format_18(srusd_bal),
            format_18(srusd_amount)
        );
    }

    // Preview: how much rUSD will be received
    let expected_rusd = preview_redeem(RPC_URL, SAVING_MODULE, srusd_amount).await.unwrap_or(0);
    println!("Expected rUSD received: ~{} rUSD (includes accumulated yield, minus redeemFee)", format_18(expected_rusd));
    println!();
    println!("Note: No ERC-20 approve needed. Saving Module burns srUSD directly from your wallet.");
    println!();

    println!("--- Step 1: SavingModule redeem (single step) ---");
    println!("  Contract: {} (Saving Module)", SAVING_MODULE);
    println!("  Function: redeem(uint256)  selector: 0xdb006a75");
    println!("  Amount:   {} srUSD ({} raw, 18 decimals)", format_18(srusd_amount), srusd_amount);
    println!("  Expected: ~{} rUSD", format_18(expected_rusd));
    println!();
    println!(">>> Please confirm: redeem {} srUSD for ~{} rUSD via Saving Module. Proceed? [y/N]", format_18(srusd_amount), format_18(expected_rusd));

    if !dry_run {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted by user.");
            return Ok(());
        }
    }

    let calldata = encode_saving_redeem(srusd_amount);
    let result = wallet_contract_call(1, SAVING_MODULE, &calldata, Some(&wallet), None, dry_run).await?;
    let tx_hash = extract_tx_hash(&result);
    println!("SavingModule redeem tx: {}", tx_hash);

    if dry_run {
        println!();
        println!("[dry-run] SavingModule redeem calldata: {}", calldata);
        println!("[dry-run] Note: SavingModule.redeem uses srUSD 18-decimal units ({} = {} srUSD)", srusd_amount, format_18(srusd_amount));
    } else {
        println!();
        println!("Redeem successful! You received ~{} rUSD.", format_18(expected_rusd));
        println!("Tip: You can further redeem rUSD -> USDC with: reservoir-protocol redeem-rusd --amount <amount>");
    }

    Ok(())
}

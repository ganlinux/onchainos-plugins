use serde_json::json;

use crate::config::{
    parse_18, resolve_eth_token, AVAX_CHAIN_ID, ETH_CHAIN_ID, SAVAX, TAVAX_ROUTER, TETH_ROUTER,
    SEL_DEPOSIT_AVAX, SEL_DEPOSIT_ETH, SEL_DEPOSIT_TOKEN,
};
use crate::onchainos;

/// Deposit ETH/WETH/stETH/wstETH → tETH (Ethereum)
/// or AVAX/sAVAX → tAVAX (Avalanche).
///
/// token: "ETH" | "WETH" | "stETH" | "wstETH" for Ethereum
///        "AVAX" | "sAVAX" for Avalanche
pub async fn run(
    chain_id: u64,
    token: &str,
    amount: &str,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    match chain_id {
        ETH_CHAIN_ID => deposit_ethereum(token, amount, from, dry_run).await,
        AVAX_CHAIN_ID => deposit_avalanche(token, amount, from, dry_run).await,
        _ => anyhow::bail!(
            "Unsupported chain_id: {}. Supported: 1 (Ethereum), 43114 (Avalanche)",
            chain_id
        ),
    }
}

async fn deposit_ethereum(
    token: &str,
    amount: &str,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let amount_wei = parse_18(amount)?;

    match token.to_uppercase().as_str() {
        "ETH" => {
            // depositETH() — send ETH as msg.value, selector 0xf6326fb3
            if dry_run {
                let output = json!({
                    "ok": true,
                    "dry_run": true,
                    "chain_id": ETH_CHAIN_ID,
                    "operation": "depositETH",
                    "router": TETH_ROUTER,
                    "amount_eth": amount,
                    "amount_wei": amount_wei.to_string(),
                    "calldata": SEL_DEPOSIT_ETH,
                    "note": "Sends ETH as msg.value; no ERC-20 approve needed"
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(());
            }

            let wallet = onchainos::resolve_wallet(ETH_CHAIN_ID)?;
            if wallet.is_empty() {
                anyhow::bail!("Cannot resolve wallet address. Please log in via onchainos.");
            }
            let sender = from.unwrap_or(&wallet);

            // Safety: u128 amount_wei fits in u64 for reasonable ETH amounts
            let amt_u64 = amount_wei as u64;
            let result = onchainos::wallet_contract_call(
                ETH_CHAIN_ID,
                TETH_ROUTER,
                SEL_DEPOSIT_ETH,
                Some(sender),
                Some(amt_u64),
                false,
            )
            .await?;

            if result["ok"].as_bool() != Some(true) {
                anyhow::bail!("depositETH failed: {}", result["error"].as_str().unwrap_or("unknown error"));
            }
            let tx_hash = onchainos::extract_tx_hash(&result);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "operation": "depositETH",
                    "chain_id": ETH_CHAIN_ID,
                    "amount_eth": amount,
                    "router": TETH_ROUTER,
                    "tx_hash": tx_hash,
                    "explorer": format!("https://etherscan.io/tx/{}", tx_hash)
                }))?
            );
        }

        "WETH" | "STETH" | "WSTETH" => {
            // deposit(address,uint256) — ERC-20 deposit; requires approve first
            let token_addr = resolve_eth_token(token)?;

            if dry_run {
                // Encode deposit(address,uint256) calldata
                let calldata = encode_deposit_token(token_addr, amount_wei);
                let approve_calldata = encode_approve(TETH_ROUTER, amount_wei);
                let output = json!({
                    "ok": true,
                    "dry_run": true,
                    "chain_id": ETH_CHAIN_ID,
                    "operation": "deposit_token",
                    "token": token,
                    "token_address": token_addr,
                    "amount": amount,
                    "amount_raw": amount_wei.to_string(),
                    "step1_approve": {
                        "to": token_addr,
                        "calldata": approve_calldata,
                        "note": "Approve tETH Router to spend your token"
                    },
                    "step2_deposit": {
                        "to": TETH_ROUTER,
                        "calldata": calldata,
                        "note": "deposit(address,uint256) on tETH Router"
                    }
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(());
            }

            let wallet = onchainos::resolve_wallet(ETH_CHAIN_ID)?;
            if wallet.is_empty() {
                anyhow::bail!("Cannot resolve wallet address. Please log in via onchainos.");
            }
            let sender = from.unwrap_or(&wallet);

            // Step 1: Approve
            eprintln!("Step 1/2: Approving {} to tETH Router...", token);
            let approve_result = onchainos::erc20_approve(
                ETH_CHAIN_ID,
                token_addr,
                TETH_ROUTER,
                amount_wei,
                Some(sender),
                false,
            )
            .await?;
            if approve_result["ok"].as_bool() != Some(true) {
                anyhow::bail!("approve failed: {}", approve_result["error"].as_str().unwrap_or("unknown error"));
            }
            let approve_tx = onchainos::extract_tx_hash(&approve_result);
            eprintln!("  approve tx: {}", approve_tx);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            // Step 2: Deposit
            eprintln!("Step 2/2: Calling deposit({}, {}) on tETH Router...", token, amount);
            let calldata = encode_deposit_token(token_addr, amount_wei);
            let deposit_result = onchainos::wallet_contract_call(
                ETH_CHAIN_ID,
                TETH_ROUTER,
                &calldata,
                Some(sender),
                None,
                false,
            )
            .await?;
            if deposit_result["ok"].as_bool() != Some(true) {
                anyhow::bail!("deposit failed: {}", deposit_result["error"].as_str().unwrap_or("unknown error"));
            }
            let deposit_tx = onchainos::extract_tx_hash(&deposit_result);

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "operation": "deposit_token",
                    "chain_id": ETH_CHAIN_ID,
                    "token": token,
                    "token_address": token_addr,
                    "amount": amount,
                    "approve_tx": approve_tx,
                    "deposit_tx": deposit_tx,
                    "router": TETH_ROUTER,
                    "explorer": format!("https://etherscan.io/tx/{}", deposit_tx)
                }))?
            );
        }

        other => {
            anyhow::bail!(
                "Unsupported token '{}' for Ethereum deposit. Supported: ETH, WETH, stETH, wstETH",
                other
            );
        }
    }

    Ok(())
}

async fn deposit_avalanche(
    token: &str,
    amount: &str,
    from: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let amount_wei = parse_18(amount)?;

    match token.to_uppercase().as_str() {
        "AVAX" => {
            // depositAVAX() — send AVAX as msg.value, selector 0xa0d065c3
            if dry_run {
                let output = json!({
                    "ok": true,
                    "dry_run": true,
                    "chain_id": AVAX_CHAIN_ID,
                    "operation": "depositAVAX",
                    "router": TAVAX_ROUTER,
                    "amount_avax": amount,
                    "amount_wei": amount_wei.to_string(),
                    "calldata": SEL_DEPOSIT_AVAX,
                    "note": "Sends AVAX as msg.value; no ERC-20 approve needed"
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(());
            }

            let wallet = onchainos::resolve_wallet(AVAX_CHAIN_ID)?;
            if wallet.is_empty() {
                anyhow::bail!("Cannot resolve wallet address. Please log in via onchainos.");
            }
            let sender = from.unwrap_or(&wallet);

            let amt_u64 = amount_wei as u64;
            let result = onchainos::wallet_contract_call(
                AVAX_CHAIN_ID,
                TAVAX_ROUTER,
                SEL_DEPOSIT_AVAX,
                Some(sender),
                Some(amt_u64),
                false,
            )
            .await?;

            if result["ok"].as_bool() != Some(true) {
                anyhow::bail!("depositAVAX failed: {}", result["error"].as_str().unwrap_or("unknown error"));
            }
            let tx_hash = onchainos::extract_tx_hash(&result);
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "operation": "depositAVAX",
                    "chain_id": AVAX_CHAIN_ID,
                    "amount_avax": amount,
                    "router": TAVAX_ROUTER,
                    "tx_hash": tx_hash,
                    "explorer": format!("https://snowtrace.io/tx/{}", tx_hash)
                }))?
            );
        }

        "SAVAX" => {
            // deposit(address,uint256) with sAVAX — requires approve first
            let token_addr = SAVAX;

            if dry_run {
                let calldata = encode_deposit_token(token_addr, amount_wei);
                let approve_calldata = encode_approve(TAVAX_ROUTER, amount_wei);
                let output = json!({
                    "ok": true,
                    "dry_run": true,
                    "chain_id": AVAX_CHAIN_ID,
                    "operation": "deposit_savax",
                    "token": "sAVAX",
                    "token_address": token_addr,
                    "amount": amount,
                    "amount_raw": amount_wei.to_string(),
                    "step1_approve": {
                        "to": token_addr,
                        "calldata": approve_calldata,
                        "note": "Approve tAVAX Router to spend your sAVAX"
                    },
                    "step2_deposit": {
                        "to": TAVAX_ROUTER,
                        "calldata": calldata,
                        "note": "deposit(address,uint256) on tAVAX Router"
                    }
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
                return Ok(());
            }

            let wallet = onchainos::resolve_wallet(AVAX_CHAIN_ID)?;
            if wallet.is_empty() {
                anyhow::bail!("Cannot resolve wallet address. Please log in via onchainos.");
            }
            let sender = from.unwrap_or(&wallet);

            // Step 1: Approve sAVAX → tAVAX Router
            eprintln!("Step 1/2: Approving sAVAX to tAVAX Router...");
            let approve_result = onchainos::erc20_approve(
                AVAX_CHAIN_ID,
                token_addr,
                TAVAX_ROUTER,
                amount_wei,
                Some(sender),
                false,
            )
            .await?;
            if approve_result["ok"].as_bool() != Some(true) {
                anyhow::bail!("approve failed: {}", approve_result["error"].as_str().unwrap_or("unknown error"));
            }
            let approve_tx = onchainos::extract_tx_hash(&approve_result);
            eprintln!("  approve tx: {}", approve_tx);
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            // Step 2: Deposit
            eprintln!("Step 2/2: Calling deposit(sAVAX, {}) on tAVAX Router...", amount);
            let calldata = encode_deposit_token(token_addr, amount_wei);
            let deposit_result = onchainos::wallet_contract_call(
                AVAX_CHAIN_ID,
                TAVAX_ROUTER,
                &calldata,
                Some(sender),
                None,
                false,
            )
            .await?;
            if deposit_result["ok"].as_bool() != Some(true) {
                anyhow::bail!("deposit_savax failed: {}", deposit_result["error"].as_str().unwrap_or("unknown error"));
            }
            let deposit_tx = onchainos::extract_tx_hash(&deposit_result);

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "ok": true,
                    "operation": "deposit_savax",
                    "chain_id": AVAX_CHAIN_ID,
                    "token": "sAVAX",
                    "token_address": token_addr,
                    "amount": amount,
                    "approve_tx": approve_tx,
                    "deposit_tx": deposit_tx,
                    "router": TAVAX_ROUTER,
                    "explorer": format!("https://snowtrace.io/tx/{}", deposit_tx)
                }))?
            );
        }

        other => {
            anyhow::bail!(
                "Unsupported token '{}' for Avalanche deposit. Supported: AVAX, sAVAX",
                other
            );
        }
    }

    Ok(())
}

/// Encode deposit(address,uint256) calldata.
/// Selector: 0x47e7ef24
fn encode_deposit_token(token_addr: &str, amount: u128) -> String {
    let addr_padded = format!(
        "{:0>64}",
        token_addr.strip_prefix("0x").unwrap_or(token_addr)
    );
    let amount_hex = format!("{:064x}", amount);
    format!("{}{}{}", SEL_DEPOSIT_TOKEN, addr_padded, amount_hex)
}

/// Encode approve(address,uint256) calldata.
/// Selector: 0x095ea7b3
fn encode_approve(spender: &str, amount: u128) -> String {
    let spender_padded = format!(
        "{:0>64}",
        spender.strip_prefix("0x").unwrap_or(spender)
    );
    let amount_hex = format!("{:064x}", amount);
    format!("0x095ea7b3{}{}", spender_padded, amount_hex)
}

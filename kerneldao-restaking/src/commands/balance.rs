use anyhow::Result;

use crate::config::{asset_info, format_amount, BSC_CHAIN_ID, KNOWN_ASSETS, STAKER_GATEWAY};
use crate::onchainos::resolve_wallet;
use crate::rpc::{decode_uint256, eth_call};

/// Selector: balanceOf(address asset, address owner) => 0xf7888aec
const BALANCE_OF_SELECTOR: &str = "f7888aec";

/// Build calldata for balanceOf(asset, owner).
fn balance_of_calldata(asset: &str, owner: &str) -> String {
    let asset_stripped = asset.strip_prefix("0x").unwrap_or(asset);
    let owner_stripped = owner.strip_prefix("0x").unwrap_or(owner);
    format!(
        "0x{}{:0>64}{:0>64}",
        BALANCE_OF_SELECTOR,
        asset_stripped.to_lowercase(),
        owner_stripped.to_lowercase()
    )
}

/// Query balance for a single asset. Returns (raw_amount, symbol, decimals).
async fn query_balance(
    asset_addr: &str,
    owner: &str,
) -> Result<(u128, String, u32)> {
    let calldata = balance_of_calldata(asset_addr, owner);
    let hex = eth_call(STAKER_GATEWAY, &calldata).await?;
    let raw = decode_uint256(&hex);

    let (sym, dec) = asset_info(asset_addr)
        .unwrap_or(("UNKNOWN", 18));

    Ok((raw, sym.to_string(), dec))
}

pub async fn run(asset: Option<&str>) -> Result<()> {
    let owner = resolve_wallet(BSC_CHAIN_ID)?;
    println!("Wallet: {}", owner);
    println!("Chain: BSC ({})", BSC_CHAIN_ID);
    println!();

    if let Some(input) = asset {
        // Resolve symbol to address if needed
        let addr = if input.starts_with("0x") || input.starts_with("0X") {
            input.to_string()
        } else {
            // Look up by symbol (case-insensitive)
            let upper = input.to_uppercase();
            KNOWN_ASSETS
                .iter()
                .find(|(sym, _, _)| sym.to_uppercase() == upper)
                .map(|(_, a, _)| a.to_string())
                .ok_or_else(|| anyhow::anyhow!(
                    "Unknown asset symbol '{}'. Use a contract address (0x...) or one of: {}",
                    input,
                    KNOWN_ASSETS.iter().map(|(s, _, _)| *s).collect::<Vec<_>>().join(", ")
                ))?
        };
        // Single asset query
        let (raw, sym, dec) = query_balance(&addr, &owner).await?;
        let human = format_amount(raw, dec);
        println!("Asset:  {} ({})", sym, addr);
        println!("Staked: {} {} ({} wei)", human, sym, raw);
    } else {
        // Query all known assets
        println!("Querying all known KernelDAO assets on BSC...");
        println!();
        let mut any_found = false;
        for (sym, addr, dec) in KNOWN_ASSETS {
            let calldata = balance_of_calldata(addr, &owner);
            match eth_call(STAKER_GATEWAY, &calldata).await {
                Ok(hex) => {
                    let raw = decode_uint256(&hex);
                    if raw > 0 {
                        let human = format_amount(raw, *dec);
                        println!(
                            "  {:12} {:>20} {} ({})",
                            sym,
                            human,
                            sym,
                            addr
                        );
                        any_found = true;
                    }
                }
                Err(e) => {
                    eprintln!("  [warn] Could not query {} ({}): {}", sym, addr, e);
                }
            }
        }
        if !any_found {
            println!("  No staked positions found.");
        }
        println!();
        println!("To stake, run:");
        println!("  kerneldao-restaking stake --asset <TOKEN_ADDRESS> --amount <AMOUNT>");
        println!("  kerneldao-restaking stake-native --amount <BNB_AMOUNT>");
    }

    Ok(())
}

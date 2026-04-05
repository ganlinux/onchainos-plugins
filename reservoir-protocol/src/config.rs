/// Reservoir Protocol contract addresses — Ethereum Mainnet (chain 1).

pub const _CHAIN_ID: u64 = 1;
pub const RPC_URL: &str = "https://eth.llamarpc.com";
pub const RPC_FALLBACK: &str = "https://ethereum.publicnode.com";

// Token addresses
pub const RUSD: &str = "0x09D4214C03D01F49544C0448DBE3A27f768F2b34";
pub const SRUSD: &str = "0x738d1115B90efa71AE468F1287fc864775e23a31";
pub const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

// Core protocol contracts
pub const CREDIT_ENFORCER: &str = "0x04716DB62C085D9e08050fcF6F7D775A03d07720";
pub const PSM_USDC: &str = "0x4809010926aec940b550D34a46A52739f996D75D";
pub const SAVING_MODULE: &str = "0x5475611Dffb8ef4d697Ae39df9395513b6E947d7";

/// Format a raw uint256 (18 decimals) to human-readable string with 6 decimal places.
pub fn format_18(raw: u128) -> String {
    let whole = raw / 1_000_000_000_000_000_000u128;
    let frac = (raw % 1_000_000_000_000_000_000u128) / 1_000_000_000_000u128;
    format!("{}.{:06}", whole, frac)
}

/// Format a raw uint256 (6 decimals) to human-readable string.
pub fn format_6(raw: u128) -> String {
    let whole = raw / 1_000_000u128;
    let frac = raw % 1_000_000u128;
    format!("{}.{:06}", whole, frac)
}

/// Parse a human-readable decimal string to u128 with 18 decimals.
pub fn parse_18(s: &str) -> anyhow::Result<u128> {
    let s = s.trim();
    if let Some((whole, frac)) = s.split_once('.') {
        let whole_val: u128 = whole.parse().map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        let frac_padded = format!("{:0<18}", frac);
        // Pad or truncate to exactly 18 chars
        let frac_str18 = if frac_padded.len() >= 18 {
            &frac_padded[..18]
        } else {
            &frac_padded
        };
        let frac_val: u128 = frac_str18.parse().map_err(|_| anyhow::anyhow!("invalid fraction: {}", s))?;
        Ok(whole_val * 1_000_000_000_000_000_000u128 + frac_val)
    } else {
        let whole_val: u128 = s.parse().map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        Ok(whole_val * 1_000_000_000_000_000_000u128)
    }
}

/// Parse a human-readable decimal string to u128 with 6 decimals (USDC).
pub fn parse_6(s: &str) -> anyhow::Result<u128> {
    let s = s.trim();
    if let Some((whole, frac)) = s.split_once('.') {
        let whole_val: u128 = whole.parse().map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        let frac_padded = format!("{:0<6}", frac);
        let frac_str = if frac_padded.len() >= 6 { &frac_padded[..6] } else { &frac_padded };
        let frac_val: u128 = frac_str.parse().map_err(|_| anyhow::anyhow!("invalid fraction: {}", s))?;
        Ok(whole_val * 1_000_000u128 + frac_val)
    } else {
        let whole_val: u128 = s.parse().map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        Ok(whole_val * 1_000_000u128)
    }
}

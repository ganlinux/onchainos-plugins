/// Lista DAO CDP — contract addresses and configuration for BSC Mainnet (chain 56)

pub const CHAIN_ID: u64 = 56;
/// Fallback RPC endpoints for BSC
pub const BSC_FALLBACK_RPCS: &[&str] = &[
    "https://bsc-rpc.publicnode.com",
    "https://bsc-dataseed.binance.org",
    "https://bsc.rpc.blxrbdn.com",
];

/// Lista CDP Interaction contract — unified CDP entry point (deposit/withdraw/borrow/payback)
pub const INTERACTION: &str = "0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4";

/// StakeManager — BNB → slisBNB liquid staking
pub const STAKE_MANAGER: &str = "0x1adB950d8bB3dA4bE104211D5AB038628e477fE6";

/// slisBNB token (Lista Liquid Staking BNB), 18 decimals
pub const SLISBNB: &str = "0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B";

/// lisUSD token (Lista stablecoin), 18 decimals
pub const LISUSD: &str = "0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5";

/// slisBNBProvider — HelioProvider contract; slisBNB collateral must go through this, not Interaction directly
pub const SLISBNB_PROVIDER: &str = "0xfD31e1C5e5571f8E7FE318f80888C1e6da97819b";

/// Format a uint256 value (18 decimals) to human-readable string with 6 decimal places.
pub fn format_18(raw: u128) -> String {
    let divisor = 1_000_000_000_000_000_000u128;
    let whole = raw / divisor;
    let frac = (raw % divisor) / 1_000_000_000_000u128; // 6 decimal places
    format!("{}.{:06}", whole, frac)
}

/// Parse a human-readable decimal string to u128 with 18 decimals.
pub fn parse_18(s: &str) -> anyhow::Result<u128> {
    let s = s.trim();
    if let Some((whole, frac)) = s.split_once('.') {
        let whole_val: u128 = whole
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        let frac_padded = format!("{:0<18}", frac);
        let frac_str_exact = if frac_padded.len() >= 18 {
            &frac_padded[..18]
        } else {
            &frac_padded
        };
        let frac_val: u128 = frac_str_exact
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid fraction: {}", s))?;
        Ok(whole_val * 1_000_000_000_000_000_000u128 + frac_val)
    } else {
        let whole_val: u128 = s
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        Ok(whole_val * 1_000_000_000_000_000_000u128)
    }
}

/// Encode an address parameter padded to 32 bytes (without 0x prefix).
pub fn encode_address(addr: &str) -> String {
    let stripped = addr.strip_prefix("0x").unwrap_or(addr);
    format!("{:0>64}", stripped)
}


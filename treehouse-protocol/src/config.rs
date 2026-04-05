/// Treehouse Protocol contract addresses and chain configuration.

// ─── Ethereum (Chain ID: 1) ────────────────────────────────────────────────
pub const ETH_CHAIN_ID: u64 = 1;
pub const ETH_RPC_PRIMARY: &str = "https://rpc.mevblocker.io";
pub const ETH_RPC_FALLBACKS: &[&str] = &[
    "https://mainnet.gateway.tenderly.co",
    "https://ethereum-rpc.publicnode.com",
];

pub const TETH_ROUTER: &str = "0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5";
pub const TETH_TOKEN: &str = "0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8";
pub const CURVE_POOL: &str = "0xA10d15538E09479186b4D3278BA5c979110dDdB1";

// Ethereum deposit tokens
pub const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
pub const STETH: &str = "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84";
pub const WSTETH: &str = "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0";

// ─── Avalanche (Chain ID: 43114) ───────────────────────────────────────────
pub const AVAX_CHAIN_ID: u64 = 43114;
pub const AVAX_RPC_PRIMARY: &str = "https://api.avax.network/ext/bc/C/rpc";
pub const AVAX_RPC_FALLBACK: &str = "https://avalanche-c-chain-rpc.publicnode.com";

pub const TAVAX_ROUTER: &str = "0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e";
pub const TAVAX_TOKEN: &str = "0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646";
pub const SAVAX: &str = "0x2b2C81e08f1Af8835a78Bb2A90AE924ACE0eA4bE";

// ─── Function Selectors (verified via cast sig) ────────────────────────────
/// depositETH() → 0xf6326fb3
pub const SEL_DEPOSIT_ETH: &str = "0xf6326fb3";
/// depositAVAX() → 0xa0d065c3
pub const SEL_DEPOSIT_AVAX: &str = "0xa0d065c3";
/// deposit(address,uint256) → 0x47e7ef24
pub const SEL_DEPOSIT_TOKEN: &str = "0x47e7ef24";
/// approve(address,uint256) → 0x095ea7b3
pub const SEL_APPROVE: &str = "0x095ea7b3";
/// balanceOf(address) → 0x70a08231
pub const SEL_BALANCE_OF: &str = "0x70a08231";
/// convertToAssets(uint256) → 0x07a2d13a
pub const SEL_CONVERT_TO_ASSETS: &str = "0x07a2d13a";
/// exchange(int128,int128,uint256,uint256) → 0x3df02124
pub const SEL_EXCHANGE: &str = "0x3df02124";

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Format a uint256 raw value (18 decimals) to human-readable string.
pub fn format_18(raw: u128) -> String {
    let whole = raw / 1_000_000_000_000_000_000u128;
    let frac = (raw % 1_000_000_000_000_000_000u128) / 1_000_000_000_000u128;
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
        let frac_str = &frac_padded[..18.min(frac_padded.len())];
        let frac_val: u128 = frac_str
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

/// Resolve the accepted deposit token address on Ethereum by symbol.
pub fn resolve_eth_token(symbol: &str) -> anyhow::Result<&'static str> {
    match symbol.to_uppercase().as_str() {
        "WETH" => Ok(WETH),
        "STETH" => Ok(STETH),
        "WSTETH" => Ok(WSTETH),
        other => anyhow::bail!(
            "Unsupported token '{}'. Supported: WETH, stETH, wstETH",
            other
        ),
    }
}

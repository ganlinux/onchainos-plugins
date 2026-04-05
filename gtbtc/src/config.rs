/// GTBTC Token contract address (same on Ethereum, BNB, Base)
pub const GTBTC_TOKEN_ADDRESS: &str = "0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3";

/// GTBTC SPL Mint address on Solana
pub const GTBTC_SOLANA_MINT: &str = "gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So";

/// GTBTC decimals = 8 (BTC precision, NOT 18)
pub const GTBTC_DECIMALS: u32 = 8;

/// Gate API v4 base URL
pub const GATE_API_BASE: &str = "https://api.gateio.ws/api/v4";

/// Solana chain ID
pub const SOLANA_CHAIN_ID: u64 = 501;

/// EVM RPC URLs
pub fn get_rpc_url(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "https://rpc.mevblocker.io",
        56 => "https://bsc-rpc.publicnode.com",
        8453 => "https://base-rpc.publicnode.com",
        _ => "https://rpc.mevblocker.io",
    }
}

/// Ethereum fallback RPC URLs (tried in order if primary fails)
pub const ETH_RPC_FALLBACKS: &[&str] = &[
    "https://rpc.mevblocker.io",
    "https://mainnet.gateway.tenderly.co",
    "https://ethereum-rpc.publicnode.com",
];

/// Solana mainnet RPC URL
pub const SOLANA_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

/// Convert a human-readable GTBTC amount to atomic units (decimals=8)
pub fn gtbtc_to_atomic(amount: f64) -> u64 {
    (amount * 1e8_f64).round() as u64
}

/// Convert atomic units to human-readable GTBTC
pub fn atomic_to_gtbtc(atomic: u64) -> f64 {
    atomic as f64 / 1e8_f64
}

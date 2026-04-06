// Chain configuration for Fenix Finance (Blast, chain ID 81457)

// Contract addresses
pub const SWAP_ROUTER: &str = "0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00";
pub const QUOTER_V2: &str = "0x94Ca5B835186A37A99776780BF976fAB81D84ED8";
pub const ALGEBRA_FACTORY: &str = "0x7a44CD060afC1B6F4c80A2B9b37f4473E74E25Df";
pub const NFPM: &str = "0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd";
pub const WETH: &str = "0x4300000000000000000000000000000000000004";
pub const USDB: &str = "0x4300000000000000000000000000000000000003";
pub const BLAST_TOKEN: &str = "0xb1a5700fa2358173fe465e6ea4ff52e36e88e2ad";
pub const FNX_TOKEN: &str = "0x52f847356b38720B55ee18Cb3e094ca11C85A192";

// Subgraph endpoints
pub const SUBGRAPH_V3_URL: &str = "https://api.goldsky.com/api/public/project_clxadvm41bujy01ui2qalezdn/subgraphs/fenix-v3-dex/latest/gn";

// Blast RPC (public)
pub const BLAST_RPC_URL: &str = "https://rpc.blast.io";
pub const BLAST_RPC_FALLBACK: &str = "https://blast-rpc.publicnode.com";

// Token decimals
pub const DEFAULT_DECIMALS: u32 = 18;

/// Resolve token symbol to (address, decimals)
pub fn resolve_token(token: &str) -> Option<(&'static str, u32)> {
    match token.to_uppercase().as_str() {
        "WETH" | "ETH" => Some((WETH, 18)),
        "USDB" => Some((USDB, 18)),
        "BLAST" => Some((BLAST_TOKEN, 18)),
        "FNX" => Some((FNX_TOKEN, 18)),
        _ => None,
    }
}

/// Return token address if input is a symbol, otherwise return as-is (raw address)
pub fn resolve_token_address(token: &str) -> String {
    if let Some((addr, _)) = resolve_token(token) {
        addr.to_string()
    } else {
        token.to_string()
    }
}

/// Return token decimals for known tokens, default 18
pub fn resolve_token_decimals(token: &str) -> u32 {
    if let Some((_, dec)) = resolve_token(token) {
        dec
    } else {
        DEFAULT_DECIMALS
    }
}

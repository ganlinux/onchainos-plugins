/// Gravita Protocol contract addresses and collateral configuration.

pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: &'static str,
    pub borrower_operations: &'static str,
    pub vessel_manager: &'static str,
    pub admin_contract: &'static str,
    pub grai_token: &'static str,
}

pub const ETHEREUM: ChainConfig = ChainConfig {
    chain_id: 1,
    rpc_url: "https://rpc.mevblocker.io",
    borrower_operations: "0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2",
    vessel_manager: "0xdB5DAcB1DFbe16326C3656a88017f0cB4ece0977",
    admin_contract: "0xf7Cc67326F9A1D057c1e4b110eF6c680B13a1f53",
    grai_token: "0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4",
};

pub const LINEA: ChainConfig = ChainConfig {
    chain_id: 59144,
    rpc_url: "https://rpc.linea.build",
    borrower_operations: "0x40E0e274A42D9b1a9D4B64dC6c46D21228d45C20",
    vessel_manager: "0xdC44093198ee130f92DeFed22791aa8d8df7fBfA",
    admin_contract: "0xC8a25eA0Cbd92A6F787AeED8387E04559053a9f8",
    grai_token: "0x894134a25a5faC1c2C26F1d8fBf05111a3CB9487",
};

pub struct Collateral {
    pub symbol: &'static str,
    pub address: &'static str,
    pub max_ltv: f64,
}

pub const ETHEREUM_COLLATERALS: &[Collateral] = &[
    Collateral { symbol: "WETH",   address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", max_ltv: 0.90 },
    Collateral { symbol: "wstETH", address: "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0", max_ltv: 0.85 },
    Collateral { symbol: "rETH",   address: "0xae78736Cd615f374D3085123A210448E74Fc6393", max_ltv: 0.85 },
];

pub const LINEA_COLLATERALS: &[Collateral] = &[
    Collateral { symbol: "wstETH", address: "0xB5beDd42000b71FddE22D3eE8a79Bd49A568fC8F", max_ltv: 0.85 },
];

/// Resolve chain config by chain_id.
pub fn get_chain(chain_id: u64) -> anyhow::Result<&'static ChainConfig> {
    match chain_id {
        1     => Ok(&ETHEREUM),
        59144 => Ok(&LINEA),
        _     => anyhow::bail!("Unsupported chain_id: {}. Supported: 1 (Ethereum), 59144 (Linea)", chain_id),
    }
}

/// Resolve collateral address by symbol for the given chain.
pub fn resolve_collateral(chain_id: u64, symbol: &str) -> anyhow::Result<&'static str> {
    let collaterals = match chain_id {
        1     => ETHEREUM_COLLATERALS,
        59144 => LINEA_COLLATERALS,
        _     => anyhow::bail!("Unsupported chain_id: {}", chain_id),
    };
    let upper = symbol.to_uppercase();
    collaterals.iter()
        .find(|c| c.symbol.to_uppercase() == upper)
        .map(|c| c.address)
        .ok_or_else(|| anyhow::anyhow!(
            "Collateral '{}' not supported on chain {}. Available: {}",
            symbol, chain_id,
            collaterals.iter().map(|c| c.symbol).collect::<Vec<_>>().join(", ")
        ))
}

/// Format a uint256 value (18 decimals) to human-readable string.
pub fn format_18(raw: u128) -> String {
    let whole = raw / 1_000_000_000_000_000_000u128;
    let frac  = (raw % 1_000_000_000_000_000_000u128) / 1_000_000_000_000u128; // 6 decimal places
    format!("{}.{:06}", whole, frac)
}

/// Parse a human-readable decimal string to u128 with 18 decimals.
pub fn parse_18(s: &str) -> anyhow::Result<u128> {
    let s = s.trim();
    if let Some((whole, frac)) = s.split_once('.') {
        let whole_val: u128 = whole.parse().map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        let frac_padded = format!("{:0<18}", frac);
        let frac_str = &frac_padded[..18];
        let frac_val: u128 = frac_str.parse().map_err(|_| anyhow::anyhow!("invalid fraction: {}", s))?;
        Ok(whole_val * 1_000_000_000_000_000_000u128 + frac_val)
    } else {
        let whole_val: u128 = s.parse().map_err(|_| anyhow::anyhow!("invalid number: {}", s))?;
        Ok(whole_val * 1_000_000_000_000_000_000u128)
    }
}

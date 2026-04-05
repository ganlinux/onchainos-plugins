/// LayerZero Endpoint ID (EID) for each supported chain.
pub fn chain_id_to_eid(chain_id: u64) -> Option<u32> {
    match chain_id {
        1 => Some(30101),      // Ethereum
        56 => Some(30102),     // BNB Chain
        43114 => Some(30106),  // Avalanche C
        137 => Some(30109),    // Polygon
        42161 => Some(30110),  // Arbitrum One
        10 => Some(30111),     // OP Mainnet
        5000 => Some(30181),   // Mantle
        8453 => Some(30184),   // Base
        59144 => Some(30183),  // Linea
        534352 => Some(30214), // Scroll
        1088 => Some(30151),   // Metis
        2222 => Some(30177),   // Kava
        _ => None,
    }
}

pub fn chain_name(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "Ethereum",
        56 => "BNB Chain",
        43114 => "Avalanche",
        137 => "Polygon",
        42161 => "Arbitrum",
        10 => "OP Mainnet",
        5000 => "Mantle",
        8453 => "Base",
        59144 => "Linea",
        534352 => "Scroll",
        1088 => "Metis",
        2222 => "Kava",
        _ => "Unknown",
    }
}

/// Parse chain name/alias to chain ID.
pub fn parse_chain(s: &str) -> anyhow::Result<u64> {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "ethereum" | "eth" | "mainnet" | "1" => Ok(1),
        "bnb" | "bsc" | "binance" | "56" => Ok(56),
        "avalanche" | "avax" | "43114" => Ok(43114),
        "polygon" | "matic" | "137" => Ok(137),
        "arbitrum" | "arb" | "42161" => Ok(42161),
        "optimism" | "op" | "10" => Ok(10),
        "mantle" | "mnt" | "5000" => Ok(5000),
        "base" | "8453" => Ok(8453),
        "linea" | "59144" => Ok(59144),
        "scroll" | "534352" => Ok(534352),
        "metis" | "1088" => Ok(1088),
        "kava" | "2222" => Ok(2222),
        other => other
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Unknown chain: {}", s)),
    }
}

/// Stargate V2 pool address for (chain_id, token_symbol).
/// Returns (pool_address, is_native).
pub fn pool_address(chain_id: u64, token: &str) -> Option<(&'static str, bool)> {
    let t = token.to_uppercase();
    match (chain_id, t.as_str()) {
        // Ethereum
        (1, "ETH") => Some(("0x77b2043768d28E9C9aB44E1aBfC95944bcE57931", true)),
        (1, "USDC") => Some(("0xc026395860Db2d07ee33e05fE50ed7bD583189C7", false)),
        (1, "USDT") => Some(("0x933597a323Eb81cAe705C5bC29985172fd5A3973", false)),
        // Arbitrum
        (42161, "ETH") => Some(("0xA45B5130f36CDcA45667738e2a258AB09f4A5f7F", true)),
        (42161, "USDC") => Some(("0xe8CDF27AcD73a434D661C84887215F7598e7d0d3", false)),
        (42161, "USDT") => Some(("0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0", false)),
        // OP Mainnet
        (10, "ETH") => Some(("0xe8CDF27AcD73a434D661C84887215F7598e7d0d3", true)),
        (10, "USDC") => Some(("0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0", false)),
        // Base
        (8453, "ETH") => Some(("0xdc181Bd607330aeeBEF6ea62e03e5e1Fb4B6F7C7", true)),
        (8453, "USDC") => Some(("0x27a16dc786820B16E5c9028b75B99F6f604b5d26", false)),
        // Polygon
        (137, "USDC") => Some(("0x9Aa02D4Fae7F58b8E8f34c66E756cC734DAc7fe4", false)),
        (137, "USDT") => Some(("0xd47b03ee6d86Cf251ee7860FB2ACf9f91B9fD4d7", false)),
        // BNB Chain
        (56, "USDC") => Some(("0x962Bd449E630b0d928f308Ce63f1A21F02576057", false)),
        (56, "USDT") => Some(("0x138EB30f73BC423c6455C53df6D89CB01d9eBc63", false)),
        // Avalanche
        (43114, "USDC") => Some(("0x5634c4a5FEd09819E3c46D86A965Dd9447d86e47", false)),
        (43114, "USDT") => Some(("0x12dC9256Acc9895B076f6638D628382881e62CeE", false)),
        _ => None,
    }
}

/// Token decimals.
pub fn token_decimals(token: &str) -> u32 {
    match token.to_uppercase().as_str() {
        "ETH" | "WETH" | "METH" => 18,
        "USDC" | "USDT" => 6,
        _ => 18,
    }
}

/// Convert human-readable amount string to raw LD units.
pub fn parse_amount(amount_str: &str, decimals: u32) -> anyhow::Result<u128> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part = parts[0].parse::<u128>().unwrap_or(0);
    let frac_str = if parts.len() > 1 { parts[1] } else { "" };
    let frac_len = frac_str.len() as u32;
    if frac_len > decimals {
        anyhow::bail!("Too many decimal places for this token (max {})", decimals);
    }
    let frac_part = if frac_str.is_empty() {
        0u128
    } else {
        frac_str.parse::<u128>()?
    };
    let multiplier = 10u128.pow(decimals);
    let frac_multiplier = 10u128.pow(decimals - frac_len);
    Ok(integer_part * multiplier + frac_part * frac_multiplier)
}

/// Format raw LD amount to human-readable string.
pub fn format_amount(amount_ld: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let integer = amount_ld / divisor;
    let frac = amount_ld % divisor;
    if frac == 0 {
        format!("{}", integer)
    } else {
        format!("{}.{:0>width$}", integer, frac, width = decimals as usize)
            .trim_end_matches('0')
            .to_string()
    }
}

/// Default RPC endpoint for each chain.
pub fn default_rpc(chain_id: u64) -> &'static str {
    match chain_id {
        1 => "https://eth.llamarpc.com",
        56 => "https://bsc-dataseed.binance.org",
        43114 => "https://api.avax.network/ext/bc/C/rpc",
        137 => "https://polygon-rpc.com",
        42161 => "https://arb1.arbitrum.io/rpc",
        10 => "https://mainnet.optimism.io",
        5000 => "https://rpc.mantle.xyz",
        8453 => "https://mainnet.base.org",
        59144 => "https://rpc.linea.build",
        534352 => "https://rpc.scroll.io",
        1088 => "https://andromeda.metis.io/?owner=1088",
        2222 => "https://evm.kava.io",
        _ => "https://eth.llamarpc.com",
    }
}

pub const BSC_CHAIN_ID: u64 = 56;
pub const STAKER_GATEWAY: &str = "0xb32dF5B33dBCCA60437EC17b27842c12bFE83394";
#[allow(dead_code)]
pub const ASSET_REGISTRY: &str = "0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB";
pub const BSC_RPC: &str = "https://bsc-rpc.publicnode.com";

// Common BTC/BNB asset addresses (BSC Mainnet)
pub const WBNB: &str = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";
pub const SLISBNB: &str = "0xb0b84d294e0c75a6abe60171b70edeb2efd14a1b";
pub const BNBX: &str = "0x1bdd3cf7f79cfb8edbb955f20ad99211551ba275";
pub const ASBNB: &str = "0x77734e70b6e88b4d82fe632a168edf6e700912b6";
pub const BTCB: &str = "0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c";
pub const SOLVBTC: &str = "0x4aae823a6a0b376De6A78e74eCC5b079d38cBCf7";
pub const SOLVBTC_BBN: &str = "0x1346b618dc92810ec74163e4c27004c921d446a5";
pub const UNIBTC: &str = "0x6b2a01a5f79deb4c2f3c0eda7b01df456fbd726a";
pub const STBTC: &str = "0xf6718b2701d4a6498ef77d7c152b2137ab28b8a3";
pub const PUMPBTC: &str = "0xf9C4FF105803A77eCB5DAE300871Ad76c2794fa4";
pub const MBTC: &str = "0x9BFA177621119e64CecbEabE184ab9993E2ef727";

/// All known supported assets: (symbol, address, decimals)
pub const KNOWN_ASSETS: &[(&str, &str, u32)] = &[
    ("WBNB", WBNB, 18),
    ("slisBNB", SLISBNB, 18),
    ("BNBx", BNBX, 18),
    ("asBNB", ASBNB, 18),
    ("BTCB", BTCB, 18),
    ("SolvBTC", SOLVBTC, 18),
    ("SolvBTC.BBN", SOLVBTC_BBN, 18),
    ("uniBTC", UNIBTC, 8),
    ("stBTC", STBTC, 18),
    ("pumpBTC", PUMPBTC, 8),
    ("mBTC", MBTC, 18),
];

/// Return symbol and decimals for a known asset address (case-insensitive).
pub fn asset_info(addr: &str) -> Option<(&'static str, u32)> {
    let lower = addr.to_lowercase();
    KNOWN_ASSETS
        .iter()
        .find(|(_, a, _)| a.to_lowercase() == lower)
        .map(|(sym, _, dec)| (*sym, *dec))
}

/// Parse human-readable amount string to raw wei/atomic units.
pub fn parse_amount(amount_str: &str, decimals: u32) -> anyhow::Result<u128> {
    let parts: Vec<&str> = amount_str.split('.').collect();
    let integer_part: u128 = parts[0].parse().unwrap_or(0);
    let frac_str = if parts.len() > 1 { parts[1] } else { "" };
    let frac_len = frac_str.len() as u32;
    if frac_len > decimals {
        anyhow::bail!(
            "Too many decimal places (token supports {} decimals, got {})",
            decimals,
            frac_len
        );
    }
    let frac_part: u128 = if frac_str.is_empty() {
        0
    } else {
        frac_str.parse()?
    };
    let multiplier = 10u128.pow(decimals);
    let frac_multiplier = 10u128.pow(decimals - frac_len);
    Ok(integer_part * multiplier + frac_part * frac_multiplier)
}

/// Format raw atomic amount to human-readable string.
pub fn format_amount(amount: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let integer = amount / divisor;
    let frac = amount % divisor;
    if frac == 0 {
        format!("{}", integer)
    } else {
        format!("{}.{:0>width$}", integer, frac, width = decimals as usize)
            .trim_end_matches('0')
            .to_string()
    }
}

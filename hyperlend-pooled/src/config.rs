// HyperLend Core Pool (Aave V3.2 fork) configuration on HyperEVM (chain 999)

pub const CHAIN_ID: u64 = 999;
pub const RPC_URL: &str = "https://rpc.hyperlend.finance";

// Core Pool contract addresses
pub const POOL: &str = "0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b";
pub const PROTOCOL_DATA_PROVIDER: &str = "0x5481bf8d3946E6A3168640c1D7523eB59F055a29";

// REST API
pub const API_MARKETS: &str = "https://api.hyperlend.finance/data/markets?chain=hyperEvm";

// Function selectors (Aave V3 standard)
pub const SEL_SUPPLY: &str = "0x617ba037";
pub const SEL_BORROW: &str = "0xa415bcad";
pub const SEL_REPAY: &str = "0x573ade81";
pub const SEL_WITHDRAW: &str = "0x69328dec";

// Solayer Protocol Constants (Solana Mainnet)

/// Restaking Program ID
pub const RESTAKING_PROGRAM: &str = "sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn";

/// sSOL Mint address (LRT — Liquid Restaking Token)
pub const SSOL_MINT: &str = "sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh";

/// Stake Pool Mint address (intermediate LST)
pub const STAKE_POOL_MINT: &str = "sSo1wxKKr6zW2hqf5hZrp2CawLibcwi1pMBqk5bg2G4";

/// Stake Pool Program ID
pub const STAKE_POOL_PROGRAM: &str = "po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2";

/// Pool Address
pub const POOL_ADDRESS: &str = "3sk58CzpitB9jsnVzZWwqeCn2zcXVherhALBh88Uw9GQ";

/// Solayer Admin Signer
pub const SOLAYER_ADMIN_SIGNER: &str = "so1MFdbL7gd8mraypNEEQeroQYqTKtS7pZCN4H46BPa";

/// Stake Pool Validator List
pub const STAKE_POOL_VALIDATOR_LIST: &str = "nk5E1Gc2rCuU2MDTRqdcQdiMfV9KnZ6JHykA1cTJQ56";

/// Stake Pool Withdraw Authority
pub const STAKE_POOL_WITHDRAW_AUTHORITY: &str = "H5rmot8ejBUWzMPt6E44h27xj5obbSz3jVuK4AsJpHmv";

/// Stake Pool Validator Stake Account
pub const STAKE_POOL_VALIDATOR_STAKE: &str = "CpWqBteUJodiTcGYWsxq4WTaBPoZJyKkBbkWwAMXSyTK";

/// Stake Pool Manager Fee Account
pub const STAKE_POOL_MANAGER_FEE_ACCOUNT: &str = "ARs3HTD79nsaUdDKqfGhgbNMVJkXVdRs2EpHAm4LNEcq";

/// Partner Restake API base URL
pub const PARTNER_RESTAKE_API: &str = "https://app.solayer.org/api/partner/restake/ssol";

/// Default Solana RPC endpoint
pub const DEFAULT_RPC: &str = "https://mainnet-rpc.solayer.org";

/// Fallback Solana RPC endpoint
pub const FALLBACK_RPC: &str = "https://api.mainnet-beta.solana.com";

/// Solana chain ID for onchainos
pub const SOLANA_CHAIN_ID: &str = "501";

/// Compute unit limit for unrestake transactions
pub const COMPUTE_UNIT_LIMIT: u32 = 500_000;

/// Compute unit price in microLamports for unrestake transactions
pub const COMPUTE_UNIT_PRICE: u64 = 200_000;

/// Lamports per SOL
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

/// Minimum SOL balance required above staked amount to cover fees
pub const MIN_GAS_BUFFER_SOL: f64 = 0.01;

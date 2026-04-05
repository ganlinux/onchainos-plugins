// Chain configuration for swellchain-staking plugin

/// Ethereum Mainnet chain ID
pub const ETHEREUM_CHAIN_ID: u64 = 1;

/// Ethereum RPC endpoints with fallback
pub const ETH_RPC_PRIMARY: &str = "https://rpc.mevblocker.io";
pub const ETH_RPC_FALLBACK1: &str = "https://mainnet.gateway.tenderly.co";
pub const ETH_RPC_FALLBACK2: &str = "https://ethereum-rpc.publicnode.com";

/// Contract addresses — Ethereum Mainnet
pub const SWETH_PROXY: &str = "0xf951E335afb289353dc249e82926178EaC7DEd78";
pub const RSWETH_PROXY: &str = "0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0";
pub const SWEXIT_PROXY: &str = "0x48C11b86807627AF70a34662D4865cF854251663";
pub const SIMPLE_STAKING_ERC20: &str = "0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774";

// Function selectors (verified via cast sig)
// deposit()                               0xd0e30db0
// createWithdrawRequest(uint256)          0x74dc9d1a
// finalizeWithdrawal(uint256)             0x5e15c749
// deposit(address,uint256,address)        0xf45346dc
// withdraw(address,uint256,address)       0x69328dec
// swETHToETHRate()                        0xd68b2cb6
// ethToSwETHRate()                        0x0de3ff57
// rswETHToETHRate()                       0xa7b9544e
// ethToRswETHRate()                       0x780a47e0
// getRate()                               0x679aefce
// getLastTokenIdCreated()                 0x061a499f
// getLastTokenIdProcessed()               0xb61d5978
// getProcessedRateForTokenId(uint256)     0xde886fb0
// balanceOf(address)                      0x70a08231
// approve(address,uint256)                0x095ea7b3

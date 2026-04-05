# Reservoir Protocol — Test Cases

Date: 2026-04-05

## L1: Build & Lint

| # | Test | Command | Expected |
|---|------|---------|----------|
| 1.1 | cargo build --release | `cargo build --release` | Exit 0, binary produced |
| 1.2 | plugin-store lint | `cargo clean && plugin-store lint .` | "passed all checks!" |

## L2: Read-Only Chain Queries (info subcommand)

| # | Test | Command | Expected |
|---|------|---------|----------|
| 2.1 | info — default wallet | `./target/release/reservoir-protocol info --chain 1` | Shows rUSD/srUSD balances, currentPrice, PSM USDC liquidity |
| 2.2 | info — explicit wallet | `./target/release/reservoir-protocol info --wallet 0x0000000000000000000000000000000000000001 --chain 1` | Returns zero balances, still shows currentPrice and PSM liquidity |
| 2.3 | PSM liquidity non-zero | Result from info | PSM USDC balance > 0 (protocol is live) |
| 2.4 | srUSD currentPrice > 1.0 | Result from info | currentPrice >= 1.0 rUSD/srUSD (yield accrued since launch) |

## L3: Dry-Run Calldata Validation

### Mint (USDC -> rUSD)

| # | Test | Command | Expected Selectors |
|---|------|---------|-------------------|
| 3.1 | mint dry-run 1 USDC | `./target/release/reservoir-protocol mint --amount 1.0 --chain 1 --dry-run` | approve calldata starts with `0x095ea7b3`, mintStablecoin calldata starts with `0xa0b4dbb1` |
| 3.2 | mint 6-decimal encoding | 1.0 USDC | Raw amount = 1_000_000 (6 decimals) |
| 3.3 | mint spender | approve step | Spender = Credit Enforcer `0x04716DB62C085D9e08050fcF6F7D775A03d07720` |
| 3.4 | mint contract | mintStablecoin step | Called on Credit Enforcer |

### Save (rUSD -> srUSD)

| # | Test | Command | Expected Selectors |
|---|------|---------|-------------------|
| 3.5 | save dry-run 1 rUSD | `./target/release/reservoir-protocol save --amount 1.0 --chain 1 --dry-run` | approve calldata starts with `0x095ea7b3`, mintSavingcoin calldata starts with `0x660cf34e` |
| 3.6 | save 18-decimal encoding | 1.0 rUSD | Raw amount = 1_000_000_000_000_000_000 (18 decimals) |
| 3.7 | save spender | approve step | Spender = Credit Enforcer `0x04716DB62C085D9e08050fcF6F7D775A03d07720` |

### Redeem rUSD -> USDC

| # | Test | Command | Expected Selectors |
|---|------|---------|-------------------|
| 3.8 | redeem-rusd dry-run 1 rUSD | `./target/release/reservoir-protocol redeem-rusd --amount 1.0 --chain 1 --dry-run` | approve calldata starts with `0x095ea7b3`, PSM redeem calldata starts with `0xdb006a75` |
| 3.9 | redeem-rusd 18-decimal encoding | 1.0 rUSD | Raw amount = 1_000_000_000_000_000_000 |
| 3.10 | redeem-rusd approve spender | approve step | Spender = PSM `0x4809010926aec940b550D34a46A52739f996D75D` |

### Redeem srUSD -> rUSD

| # | Test | Command | Expected Selectors |
|---|------|---------|-------------------|
| 3.11 | redeem-srusd dry-run 1 srUSD | `./target/release/reservoir-protocol redeem-srusd --amount 1.0 --chain 1 --dry-run` | SavingModule redeem calldata starts with `0xdb006a75` |
| 3.12 | redeem-srusd no approve | Output | No approve step (single step — SavingModule burns srUSD directly) |

## L4: Live On-Chain (chain 1)

| # | Test | Precondition | Expected |
|---|------|-------------|----------|
| 4.1 | mint 1 USDC | Wallet has >= 1.0 USDC | approve + mintStablecoin broadcast, txHash returned |
| 4.2 | USDC balance check | USDC balance readable | If USDC < 1.0: L4 BLOCKED, documented |

## Decimal Trap Checks

| # | Trap | Verification |
|---|------|-------------|
| D1 | mintStablecoin uses 6-decimal USDC | 1.0 USDC encoded as 1_000_000 in calldata |
| D2 | mintSavingcoin uses 18-decimal rUSD | 1.0 rUSD encoded as 1_000_000_000_000_000_000 in calldata |
| D3 | PSM.redeem uses 18-decimal rUSD | Same as D2 |
| D4 | SavingModule.redeem uses 18-decimal srUSD | Same as D2 |

# Reservoir Protocol — Test Results

Date: 2026-04-05
Plugin version: 0.1.0
Chain: Ethereum Mainnet (chain 1)
Wallet: 0xe4621cadb69e7eda02248ba03ba538137d329b94

---

## L1: Build & Lint

| Test | Command | Result | Notes |
|------|---------|--------|-------|
| cargo build --release | `cargo build --release` | PASS | Binary produced in target/release/ |
| plugin-store lint | `cargo clean && plugin-store lint .` | PASS | "Plugin 'reservoir-protocol' passed all checks!" |

---

## Bugs Found & Fixed (Fix Round 1)

### Bug 1: resolve_wallet fails when wallet has no token assets on chain 1

**File:** `src/onchainos.rs`

**Root cause:** `resolve_wallet()` reads `data.details[0].tokenAssets[0].address` from `onchainos wallet balance --chain 1`. When the wallet holds no tokens on chain 1, `tokenAssets` is an empty array, so this lookup returns nothing. The fallback read `data.address` also doesn't exist in the balance response schema. Result: wallet address resolves to empty string, causing `info`/`mint`/`save`/`redeem` to fail with "No wallet found".

**Fix:** Added secondary fallback: call `onchainos wallet addresses`, parse `data.evm[]`, find entry matching `chainIndex == "1"`, return its `address`. If no chain-specific match found, use first EVM entry (all EVM chains share the same address).

**Verification:** `reservoir-protocol info` now resolves wallet `0xe4621cadb69e7eda02248ba03ba538137d329b94` correctly.

### Bug 2: RPC rate limiting causes silent zero results

**File:** `src/config.rs`, `src/rpc.rs`

**Root cause:** `eth.llamarpc.com` returns "Too many connections" as a plain-text response (not JSON). The client errors and `unwrap_or(0)` silences it, returning 0 for all on-chain reads (PSM liquidity, srUSD price, etc.).

**Fix:** Added `RPC_FALLBACK = "https://ethereum.publicnode.com"` in config.rs. Refactored `eth_call()` to call a new `eth_call_once()` helper, and retry on the fallback URL if primary fails.

**Verification:** `info` now reliably returns currentPrice = 1.13575565 rUSD/srUSD and PSM USDC = 521,191.417227 USDC.

---

## L2: Read-Only Chain Queries (info)

| Test | Expected | Actual | Result |
|------|----------|--------|--------|
| Wallet resolved | Non-empty address | 0xe4621cadb69e7eda02248ba03ba538137d329b94 | PASS |
| rUSD balance | 0 (new wallet) | 0.000000 rUSD | PASS |
| srUSD balance | 0 (new wallet) | 0.000000 srUSD | PASS |
| srUSD currentPrice | > 1.0 rUSD/srUSD | 1.13575565 rUSD/srUSD | PASS |
| PSM USDC liquidity | > 0 | 521191.417227 USDC | PASS |

**Conclusion:** L2 PASS. Protocol is live, currentPrice confirms yield has accrued (~13.6% since inception). PSM has ample liquidity (521K USDC) for redemptions.

---

## L3: Dry-Run Calldata Validation

### Mint (USDC -> rUSD, 1.0 USDC)

| Test | Expected | Actual | Result |
|------|----------|--------|--------|
| approve selector | `0x095ea7b3` | `0x095ea7b3...` | PASS |
| approve spender | Credit Enforcer `0x04716DB6...` | `0x04716DB62C085D9e08050fcF6F7D775A03d07720` | PASS |
| mintStablecoin selector | `0xa0b4dbb1` | `0xa0b4dbb100000000000000000000000000000000000000000000000000000000000f4240` | PASS |
| 6-decimal encoding | 1_000_000 | `000f4240` = 1000000 | PASS |
| Contract | Credit Enforcer | `0x04716DB62C085D9e08050fcF6F7D775A03d07720` | PASS |

### Save (rUSD -> srUSD, 1.0 rUSD)

| Test | Expected | Actual | Result |
|------|----------|--------|--------|
| approve selector | `0x095ea7b3` | `0x095ea7b3...` | PASS |
| approve spender | Credit Enforcer | `0x04716DB62C085D9e08050fcF6F7D775A03d07720` | PASS |
| mintSavingcoin selector | `0x660cf34e` | `0x660cf34e0000000000000000000000000000000000000000000000000de0b6b3a7640000` | PASS |
| 18-decimal encoding | 1_000_000_000_000_000_000 | `0de0b6b3a7640000` = 1000000000000000000 | PASS |
| Exchange rate preview | ~0.88 srUSD per rUSD | 0.880471 srUSD (matches 1/1.13575565) | PASS |

### Redeem rUSD -> USDC (1.0 rUSD)

| Test | Expected | Actual | Result |
|------|----------|--------|--------|
| approve selector | `0x095ea7b3` | `0x095ea7b3...` | PASS |
| approve spender | PSM `0x48090109...` | `0x4809010926aec940b550D34a46A52739f996D75D` | PASS |
| PSM redeem selector | `0xdb006a75` | `0xdb006a750000000000000000000000000000000000000000000000000de0b6b3a7640000` | PASS |
| 18-decimal encoding | 1_000_000_000_000_000_000 | `0de0b6b3a7640000` = 1000000000000000000 | PASS |
| PSM liquidity check | 521191 USDC shown | 521191.417227 USDC | PASS |

**Conclusion:** L3 PASS. All selectors correct. Decimal encoding correct (6 dec for USDC, 18 dec for rUSD/srUSD).

---

## L4: Live On-Chain Test

| Item | Value |
|------|-------|
| Lock acquired | YES — phase3 lock acquired after queue (stargate-v2 was ahead) |
| Wallet USDC balance | 0.000000 USDC |
| L4 Status | **BLOCKED** — insufficient USDC to execute mint |

**Reason:** Test wallet `0xe4621cadb69e7eda02248ba03ba538137d329b94` holds no USDC on Ethereum mainnet. Minting requires >= 1.0 USDC. L4 live transaction test cannot proceed.

**Lock released:** YES

---

## Step 1.5: L0 Routing Validation

| Route | Trigger | Subcommand | Status |
|-------|---------|------------|--------|
| info | "rUSD balance", "srUSD APY" | `info` | PASS |
| mint | "mint rUSD", "deposit USDC" | `mint --amount X` | PASS |
| save | "save rUSD", "earn yield" | `save --amount X` | PASS |
| redeem-rusd | "redeem rUSD", "rUSD to USDC" | `redeem-rusd --amount X` | PASS |
| redeem-srusd | "redeem srUSD", "srUSD to rUSD" | `redeem-srusd --amount X` | PASS |

---

## Decimal Trap Verification

| Trap | Encoding | Status |
|------|----------|--------|
| mintStablecoin (USDC 6 dec) | 1.0 USDC = 1,000,000 raw | PASS |
| mintSavingcoin (rUSD 18 dec) | 1.0 rUSD = 1,000,000,000,000,000,000 raw | PASS |
| PSM.redeem (rUSD 18 dec) | 1.0 rUSD = 1,000,000,000,000,000,000 raw | PASS |
| SavingModule.redeem (srUSD 18 dec) | 1.0 srUSD = 1,000,000,000,000,000,000 raw | PASS |

---

## Summary

| Level | Status | Notes |
|-------|--------|-------|
| L1 Build | PASS | `cargo build --release` succeeds |
| L1 Lint | PASS | `plugin-store lint` all checks passed |
| L0 Routing | PASS | All 5 routes verified |
| L2 Info | PASS | Live data: price=1.1358, PSM=521K USDC |
| L3 Dry-run | PASS | All selectors correct, decimals verified |
| L4 Live | BLOCKED | No USDC in test wallet |

**Fixes applied:** 2 bugs fixed (wallet resolution fallback, RPC rate-limit fallback)
**Fix rounds used:** 1 of 3
**Overall verdict: PASS (L1-L3), L4 BLOCKED (balance)**

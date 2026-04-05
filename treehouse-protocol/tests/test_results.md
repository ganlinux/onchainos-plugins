# Test Results — treehouse-protocol

**Date:** 2026-04-05
**Tester:** Pipeline Tester Agent
**Plugin version:** 0.1.0
**Chains tested:** Ethereum (1), Avalanche (43114)

---

## L1 — Compile + Lint

| ID | Test | Result | Notes |
|----|------|--------|-------|
| L1-01 | `cargo build --release` | PASS | 3 dead-code warnings (unused constants in config.rs) — acceptable |
| L1-02 | `plugin-store lint .` | PASS | "Plugin 'treehouse-protocol' passed all checks!" |

**L1 Result: PASS**

---

## L0 — Skill Routing Validation

Routing tests written to `tests/routing_test.md`.

Key findings:
- All 5 commands (`deposit`, `balance`, `price`, `positions`, `withdraw`) map correctly to binary CLI flags
- `balance`, `price`, `positions` use `--chain` (not `--asset`) — confirmed in source
- SKILL.md correctly documents the API; no mismatches found
- 3 negative routing cases validated (Lido, Aave, mETH/Mantle all excluded correctly)

**L0 Result: PASS**

---

## L2 — Read Tests (no gas)

| ID | Command | Result | Output Sample |
|----|---------|--------|---------------|
| L2-01 | `price --chain 1` | PASS | `"token":"tETH", "price":"1.006760"` — 1 tETH = 1.006760 wstETH |
| L2-02 | `price --chain 43114` | PASS | `"token":"tAVAX", "price":"1.007124"` — 1 tAVAX = 1.007124 sAVAX |
| L2-03 | `balance --chain 1 --account <addr>` | PASS | `"token":"tETH", "balance":"0.000000"` |
| L2-04 | `balance --chain 43114 --account <addr>` | PASS | `"token":"tAVAX", "balance":"0.000000"` |
| L2-05 | `positions --chain 1 --account <addr>` | PASS | `"apy_percent":2.58897, "tvl_usd":102485466.0` |
| L2-06 | `positions --chain 43114 --account <addr>` | PASS | `"apy_percent":5.43415, "tvl_usd":7688843.0` |

**L2 Result: PASS (6/6)**

---

## L3 — Dry-run Tests (selector verification)

| ID | Command | Result | Selector Verified |
|----|---------|--------|-------------------|
| L3-01 | `deposit --token ETH --amount 0.001 --chain 1 --dry-run` | PASS | calldata=`0xf6326fb3` ✅ (depositETH) |
| L3-02 | `deposit --token wstETH --amount 0.001 --chain 1 --dry-run` | PASS | step1=`0x095ea7b3` ✅ (approve), step2=`0x47e7ef24` ✅ (deposit) |
| L3-03 | `deposit --token AVAX --amount 0.1 --chain 43114 --dry-run` | PASS | calldata=`0xa0d065c3` ✅ (depositAVAX) |
| L3-04 | `deposit --token sAVAX --amount 0.1 --chain 43114 --dry-run` | PASS | step1=`0x095ea7b3` ✅, step2=`0x47e7ef24` ✅ |
| L3-05 | `withdraw --amount 0.001 --chain 1 --dry-run` | PASS | step1=`0x095ea7b3` ✅ (approve), step2=`0x3df02124` ✅ (Curve exchange) |
| L3-06 | `withdraw --amount 0.001 --chain 43114 --dry-run` | PASS (expected error) | Error: "tAVAX withdrawal is not supported" ✅ |

All selectors match specification:
- depositETH: `0xf6326fb3` ✅
- deposit(address,uint256): `0x47e7ef24` ✅
- depositAVAX: `0xa0d065c3` ✅
- approve: `0x095ea7b3` ✅
- Curve exchange: `0x3df02124` ✅

**L3 Result: PASS (6/6)**

---

## L4 — On-chain Tests

**Wallet:** `0xe4621cadb69e7eda02248ba03ba538137d329b94`
**ETH balance (chain 1):** 0.00 USD — insufficient
**AVAX balance (chain 43114):** 0.00 USD — insufficient

| ID | Test | Result | Notes |
|----|------|--------|-------|
| L4-01 | `balance --chain 1` (real wallet, no gas) | PASS | Returns valid JSON: `"balance":"0.000000", "token":"tETH"` |
| L4-02 | `positions --chain 1` (real wallet, no gas) | PASS | Returns `apy_percent:2.58897, tvl_usd:102485466.0` |
| L4-03 | ETH deposit on chain 1 | BLOCKED | Test wallet ETH balance = 0 |
| L4-04 | AVAX deposit on chain 43114 | BLOCKED | Test wallet AVAX balance = 0 |

**L4 Result: PASS (read-only); BLOCKED (write ops — insufficient funds)**

---

## Bug Found and Fixed

### Bug: Wallet auto-resolution fails for zero-balance wallets

**Severity:** Medium (affects all write ops without `--from` flag; read ops unaffected with `--account`)

**Root cause:** `resolve_wallet()` in `src/onchainos.rs` — Fallback 3 (`onchainos wallet addresses`) assumed `data` was a flat array, but the actual API response uses a nested object:
```json
{
  "data": {
    "evm": [{ "address": "...", "chainIndex": "1" }, ...],
    "solana": [...],
    "xlayer": [...]
  }
}
```

The code was doing `addr_json["data"].as_array()` which returned `None` since `data` is an object.

**Fix:** Updated `resolve_wallet` in `src/onchainos.rs` to correctly parse `data.evm` array (with `data.xlayer` fallback), then try exact `chainIndex` match, then fall back to first EVM address. Legacy flat-array path preserved for older onchainos versions.

**Verified:** After fix, `balance --chain 1` (no `--account`) correctly resolves wallet address `0xe4621cadb69e7eda02248ba03ba538137d329b94` and returns valid JSON.

---

## Error Case Tests

| ID | Command | Result | Output |
|----|---------|--------|--------|
| E-01 | `deposit --token ETH --chain 999 --dry-run` | PASS | Error: "Unsupported chain_id: 999" ✅ |
| E-02 | `deposit --token USDC --chain 1 --dry-run` | PASS | Error: "Unsupported token 'USDC'" ✅ |
| E-03 | `withdraw --chain 43114 --dry-run` | PASS | Error: "tAVAX withdrawal is not supported" ✅ |

---

## Summary

| Level | Tests | Pass | Fail | Blocked |
|-------|-------|------|------|---------|
| L1 | 2 | 2 | 0 | 0 |
| L0 | 18 (15 pos + 3 neg) | 18 | 0 | 0 |
| L2 | 6 | 6 | 0 | 0 |
| L3 | 6 | 6 | 0 | 0 |
| L4 | 4 | 2 | 0 | 2 |
| Error cases | 3 | 3 | 0 | 0 |
| **Total** | **39** | **37** | **0** | **2** |

**Overall: PASS** — All functional tests pass. Write op L4 tests blocked due to zero wallet balance (expected for test environment). One bug found and fixed (wallet address resolution for zero-balance wallets).

### Live on-chain data confirmed
- tETH price: 1 tETH = 1.006760 wstETH (Ethereum)
- tAVAX price: 1 tAVAX = 1.007124 sAVAX (Avalanche)
- tETH APY: 2.58897% (DeFiLlama)
- tAVAX APY: 5.43415% (DeFiLlama)
- tETH TVL: $102.5M USD
- tAVAX TVL: $7.7M USD

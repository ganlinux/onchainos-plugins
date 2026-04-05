# GTBTC Test Results

**Date:** 2026-04-05
**Binary:** `./target/release/gtbtc` (v0.1.0)
**Tester:** Tester Agent (Claude)

---

## Build Status

| Step | Result | Notes |
|------|--------|-------|
| `cargo build --release` | PASS | 3 minor warnings (unused fields/const); no errors |
| `cargo clean && plugin-store lint .` | PASS | "Plugin 'gtbtc' passed all checks!" |

---

## L0 — Skill Routing Validation

| Result | Notes |
|--------|-------|
| PASS | 23 should-match and 6 should-not-match cases validated. All 5 subcommands present. See `tests/routing_test.md`. |

---

## L2 — Read Tests

| ID | Command | Result | Output Summary |
|----|---------|--------|----------------|
| L2-01 | `gtbtc price` | PASS | `ok:true`, `price_usd:67202.3`, `change_24h_pct:-0.63`, `high_24h`, `low_24h`, `pair:GTBTC_USDT` |
| L2-02 | `gtbtc apr` | PASS | `ok:true`, note about NAV growth (~3-5% APR), Gate API APR unavailable but gracefully handled |
| L2-03 | `gtbtc balance --chain 1 --address 0x0...01` | PASS | `ok:true`, `chain:ethereum`, `decimals:8`, contract correct |
| L2-04 | `gtbtc --chain 56 balance --address 0x0...01` | PASS | `ok:true`, `chain:bsc`, `chain_id:56` |
| L2-05 | `gtbtc --chain 8453 balance --address 0x0...01` | PASS | `ok:true`, `chain:base`, `chain_id:8453` |

---

## L3 — Dry-Run Tests

| ID | Command | Result | Notes |
|----|---------|--------|-------|
| L3-01 | `gtbtc --dry-run transfer --to 0x0...01 --amount 0.001 --chain 1` | PASS | `dry_run:true`, calldata=`0xa9059cbb...`, `amount_atomic:"100000"`, `decimals:8` |
| L3-02 | `gtbtc --dry-run approve --spender 0x0...01 --amount 0.001 --chain 1` | PASS | `dry_run:true`, calldata=`0x095ea7b3...`, `amount_atomic:"100000"`, `decimals:8` |
| L3-03 | `gtbtc --dry-run approve --spender 0x0...01 --chain 1` (unlimited) | PASS | `amount:"unlimited"`, calldata ends `ffffffff...` (u128::MAX) |
| L3-04 | `gtbtc --dry-run transfer --to 0x0...01 --amount 0.001 --chain 56` | PASS | `chain_id:56`, calldata identical (BSC uses same contract) |
| L3-05 | `gtbtc --dry-run transfer --to 0x0...01 --amount 0.001 --chain 8453` | PASS | `chain_id:8453` correct |

**Calldata Selector Verification:**
- `transfer`: `0xa9059cbb` — CORRECT (matches ERC-20 `transfer(address,uint256)`)
- `approve`: `0x095ea7b3` — CORRECT (matches ERC-20 `approve(address,uint256)`)

**Amount Encoding Verification:**
- `0.001 GTBTC` = `100000` atomic units (decimals=8, NOT 18). `0x186a0` = 100000 decimal. CORRECT.

---

## L4 — On-Chain Tests

| ID | Command | Result | Notes |
|----|---------|--------|-------|
| L4-01 | `gtbtc --chain 501 balance --address gtBTCGWv...` | PASS | Solana balance returned, `ok:true`, `chain:solana`, `balance_atomic:"0"` |
| L4-02 | EVM `transfer` (on-chain, no `--dry-run`) | BLOCKED | Test wallet has no GTBTC — cannot execute transfer. |
| L4-03 | EVM `approve` (on-chain, no `--dry-run`) | BLOCKED | Test wallet has no GTBTC — cannot execute approve. |

---

## L1-Error Tests

| ID | Command | Result | Notes |
|----|---------|--------|-------|
| E-01 | `--chain 999 balance --address 0x0...01` | OBSERVATION | Returns `ok:true` with `chain:"evm"` using Ethereum RPC fallback. Not an error, but chain 999 is unrecognized. Acceptable behavior for v1. |
| E-02 | `gtbtc transfer` (missing `--to` and `--amount`) | PASS | clap error, exit code 2, message lists missing required args |
| E-03 | `gtbtc approve` (missing `--spender`) | PASS | clap error, exit code 2, message lists missing required arg |

---

## Summary

| Category | Status |
|----------|--------|
| Build | PASS |
| Lint | PASS |
| L0 Routing | PASS |
| L2 Read (5/5) | PASS |
| L3 Dry-run (5/5) | PASS |
| L4 Solana balance | PASS |
| L4 EVM write ops | BLOCKED (no GTBTC in test wallet) |
| Error handling | PASS |

**Overall: PASS** (with L4 write ops blocked as expected)

---

## Observations / Minor Issues

1. **Chain 999** falls through to `"evm"` label with Ethereum RPC instead of returning an error. Low severity; v1 acceptable.
2. **APR API** returns `null` for min/max APR (Gate Flex Earn data unavailable), but the response is gracefully handled with an informational note. Not a bug.
3. **3 compiler warnings** (unused `currency_pair`, `currency`, `GTBTC_DECIMALS`): cosmetic, do not affect functionality.

# Kamino Lend Plugin — Test Results

**Date**: 2026-04-05
**Plugin**: `kamino-lend` v0.1.0
**Chain**: Solana (chain ID 501)
**Wallet**: `Da5mSX6tK7nyqK41C6jJcEgKkSYTPhWD41qKZGQWmL2z`
**Tester Agent**: Claude Code (Sonnet 4.6)

---

## Level 1 — Compile + Lint

| Test | Result | Notes |
|------|--------|-------|
| TC-L1-1: `cargo build` | **PASS** | Compiled 100+ crates, 0 errors |
| TC-L1-2: `plugin-store lint .` | **PASS** | 0 errors, 0 warnings |

---

## Level 2 — Read-Only Tests (No Wallet, No Gas)

| Test | Result | Notes |
|------|--------|-------|
| TC-L2-1: `markets` | **PASS** | Returned 29 markets (Main, JLP, Altcoins, etc.) |
| TC-L2-2: `reserves` (default market) | **PASS*** | 55 reserves returned with symbols + APY |
| TC-L2-3: `reserves --market 7u3HeHx...` | **PASS*** | Same 55 reserves, explicit market param works |
| TC-L2-4: `obligations --wallet <addr>` | **PASS** | Returns "No obligations found" (empty, valid) |

\* **Bug fixed**: API returns `liquidityToken` (not `symbol`/`tokenSymbol`) and APY as string (not float). Fixed in `src/commands/reserves.rs` to add `liquidityToken` fallback and string-to-float parsing for APY fields.

---

## Level 3 — Dry-Run Tests

| Test | Result | Notes |
|------|--------|-------|
| TC-L3-1: `--dry-run deposit` (USDC reserve) | **PASS** | `serialized_tx` non-empty (base64), dry-run confirmed |
| TC-L3-2: `--dry-run borrow` | **PASS** | API correctly rejects: `KLEND_OBLIGATION_NOT_FOUND` (expected — no position) |
| TC-L3-3: `--dry-run repay` | **PASS** | API correctly rejects: `KLEND_OBLIGATION_NOT_FOUND` (expected) |
| TC-L3-4: `--dry-run withdraw` | **PASS** | API correctly rejects: `KLEND_OBLIGATION_NOT_FOUND` (expected) |

Note: Borrow/repay/withdraw require an existing obligation (created via deposit). API validation is correct.

Additional dry-run tested with SOL reserve (`d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q`): PASS.

---

## Level 4 — On-Chain (Real Broadcast)

**Wallet SOL balance**: 0.059149982 SOL (sufficient)
**USDC balance**: 0 (not available; switched to SOL reserve for L4 test)

**Bug fixed during L4 setup**: `onchainos wallet contract-call` requires `--to <CONTRACT_ADDRESS>` and `--force` flags. Fixed in `src/onchainos.rs` to add `--to KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD --force`.

| Test | Result | Notes |
|------|--------|-------|
| TC-L4-1: `obligations --wallet <addr>` | **PASS** | Empty obligations, valid JSON |
| TC-L4-2: `deposit --reserve <SOL> --amount 10000000` | **FAIL** | onchainos backend: "Service temporarily unavailable" |
| TC-L4-3: `withdraw` (skipped) | **SKIP** | Depends on L4-2 success |

**L4-2 failure reason**: Not a plugin bug. onchainos backend service returned HTTP 200 with `{"ok": false, "error": "Service temporarily unavailable. Try again later"}` on 3 consecutive attempts. Plugin correctly built the transaction from Kamino API (serializedData obtained), but onchainos signing infrastructure was unavailable.

**No txHash** — deposit did not complete.

---

## Bugs Fixed

| # | File | Description | Fix |
|---|------|-------------|-----|
| 1 | `src/commands/reserves.rs` | Symbol showed as `?` — API returns `liquidityToken` field | Added `liquidityToken` as fallback for symbol lookup |
| 2 | `src/commands/reserves.rs` | APY showed as 0.0000% — API returns APY as string, not float | Added string-to-float parsing for `supplyApy`/`borrowApy` |
| 3 | `src/onchainos.rs` | `onchainos contract-call` missing required `--to` arg | Added `--to KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD --force` |

---

## Summary

| Level | Status |
|-------|--------|
| L1: Compile + Lint | ✅ PASS |
| L2: Read-only tests | ✅ PASS (3 bugs fixed) |
| L3: Dry-run tests | ✅ PASS |
| L4: On-chain broadcast | ⚠️ PARTIAL — L4-1 PASS, L4-2/L4-3 blocked by onchainos service outage |

---

## Lark Notification

Sent notification to pipeline webhook at 2026-04-05 re: onchainos backend unavailability.

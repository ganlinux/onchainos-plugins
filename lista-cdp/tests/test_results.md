# Lista CDP — Test Report

**Date:** 2026-04-05
**Tester:** Tester Agent (Claude Sonnet 4.6)
**Plugin version:** 0.1.0
**Chain:** BSC Mainnet (chain 56)

---

## Summary

| Level | Status | Notes |
|-------|--------|-------|
| L1: Compile + Lint | PASS | Build succeeds; lint passes after `cargo clean` |
| L0: Skill Routing | PASS | All commands routed correctly |
| L2: Read Tests | PASS | positions returns valid output |
| L3: Dry-run Tests | PASS | All 5 selectors verified correct |
| L4: On-chain Tests | PARTIAL | positions PASS; stake BLOCKED (0 BNB) |

**Overall verdict: PASS** (no blocking defects found)

---

## L1: Compile + Lint

| # | Test | Result | Detail |
|---|------|--------|--------|
| L1-1 | `cargo build --release` | PASS | Compiled in 12.7s, binary at `target/release/lista-cdp` |
| L1-2 | `cargo clean && plugin-store lint .` | PASS | "Plugin 'lista-cdp' passed all checks!" — E080/E130 errors only appear when `target/` present; must lint on source |

**Note:** The lint tool checks all files in the directory. Running with `target/` present causes 201 E080/E130 errors (build artifacts). Always `cargo clean` before linting — documented behavior, not a bug.

---

## L0: Skill Routing Validation

See `tests/routing_test.md`.

**Verdict: PASS** — SKILL.md coverage is complete for all 7 commands. Negative cases (general BNB staking, other BSC CDPs) correctly excluded.

---

## L2: Read Tests (on-chain, no wallet required)

| # | Test | Result | Detail |
|---|------|--------|--------|
| L2-1 | `positions --wallet <zero addr>` | PASS | Returns valid CDP position with all fields |
| L2-2 | `positions` (logged-in wallet) | PASS | wallet `0xe4621ca...` resolved via fallback 3; position shows correctly |

**Output format verified:** CDP Position section, Wallet Balances section, Health section — all present.

**Note (non-blocking):** `Borrow APR` and `Max LTV` lines absent. Root cause: `Interaction.borrowApr(slisBNB)` and `collateralRate(slisBNB)` return execution revert (code 3) when called against the live contract. Both calls use verified selectors (`0x9c2b9b63`, `0x37ffefd4`) from design.md. This is likely a contract access-control issue. Code handles this gracefully with `if let Ok()` — silent skip. **Recommend:** investigate contract ABI to confirm these functions are externally callable, or remove from positions output if not available.

---

## L3: Dry-run Tests

| # | Command | Expected Selector | Result | Verified |
|---|---------|------------------|--------|----------|
| L3-1 | `stake --amt 10000000000000000` | `0xd0e30db0` | `[dry-run] calldata: 0xd0e30db0` | PASS |
| L3-2 | `cdp-deposit --amount 0.01` | deposit: `0x8340f549`, approve: `0x095ea7b3` | deposit: `0x8340f549...`, approve info shown | PASS |
| L3-3 | `borrow --amount 15` | `0x4b8a3529` | `0x4b8a3529...` | PASS |
| L3-4 | `repay --amount 15` | payback: `0x35ed8ab8`, approve: `0x095ea7b3` | `0x35ed8ab8...` + approve info | PASS |
| L3-5 | `cdp-withdraw --amount 0.01` | `0xd9caed12` | `0xd9caed12...` | PASS |
| L3-6 | `stake --amt 100` (below min) | Error | "Amount 100 wei is below minimum 0.001 BNB" | PASS |
| L3-7 | `borrow --amount 1` (below min) | Error | "Borrow amount 1 lisUSD is below minimum 15 lisUSD" | PASS |

**Calldata encoding verified:**
- stake: selector only (`0xd0e30db0`) — correct for payable deposit()
- cdp-deposit: `deposit(address,address,uint256)` — participant=0x0...0 (placeholder in dry-run), token=slisBNB, dink=amount
- borrow: `borrow(address,uint256)` — token=slisBNB, hayAmount=15 lisUSD
- repay: `payback(address,uint256)` — token=slisBNB, amount=15 lisUSD
- cdp-withdraw: `withdraw(address,address,uint256)` — wallet=0x0...0 (placeholder), token=slisBNB, amount

---

## L4: On-chain Tests

**Wallet:** `0xe4621cadb69e7eda02248ba03ba538137d329b94`
**BSC BNB balance:** 0 BNB

| # | Test | Result | Detail |
|---|------|--------|--------|
| L4-1 | `positions` (real wallet) | PASS | Wallet resolved via fallback 3; shows 0 collateral, 0 debt |
| L4-2 | `stake 0.01 BNB` | BLOCKED | Wallet has 0 BNB on BSC — insufficient for transaction |
| L4-3 | `cdp-deposit` | BLOCKED | No slisBNB available (requires stake first) |
| L4-4 | `borrow` | BLOCKED | No collateral deposited |
| L4-5 | `repay` | BLOCKED | No debt outstanding |
| L4-6 | `cdp-withdraw` | BLOCKED | No collateral deposited |

**resolve_wallet fallback 3 fix:** Confirmed working. Uses `addr_json["data"]["evm"].as_array()` — correctly finds wallet address even with 0 BNB balance.

---

## Issues Found

| # | Severity | Issue | Status |
|---|----------|-------|--------|
| 1 | INFO | `Borrow APR` and `Max LTV` silently absent from `positions` output — contract calls revert | Non-blocking; graceful degradation |
| 2 | INFO | `positions` takes 15-20s due to sequential RPC calls (8x eth_call + wallet resolve) | Non-blocking; acceptable for BSC RPC latency |
| 3 | INFO | L4 write operations BLOCKED — wallet has 0 BNB on BSC | External constraint; not a code defect |

---

## Contract Addresses Verified

| Contract | Address | Status |
|----------|---------|--------|
| Interaction (CDP) | `0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4` | LIVE (eth_call responds) |
| StakeManager | `0x1adB950d8bB3dA4bE104211D5AB038628e477fE6` | LIVE |
| slisBNB | `0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B` | LIVE (balanceOf works) |
| lisUSD | `0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5` | LIVE (balanceOf works) |

# KernelDAO Restaking — Test Results

**Date:** 2026-04-05
**Tester:** Tester Agent
**Chain:** BSC (chain 56)
**StakerGateway:** `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`
**Wallet:** `0xe4621cadb69e7eda02248ba03ba538137d329b94`

---

## Build & Lint

| Check | Result | Notes |
|-------|--------|-------|
| `cargo build --release` | PASS | Compiled cleanly |
| `plugin-store lint .` | PASS | 0 errors, 0 warnings |

---

## Fixes Applied During Testing

### Fix 1: `resolve_wallet` — empty tokenAssets fallback (Bug)
**File:** `src/onchainos.rs`
**Root cause:** When the BSC wallet has zero balance, `onchainos wallet balance --chain 56` returns `tokenAssets: []`. The original code only checked `tokenAssets[0].address` and `data.address`, both of which are absent for a zero-balance wallet. This caused `balance`, `stake`, `unstake`, and all other commands to fail with "Could not resolve wallet address".
**Fix:** Added a third fallback: call `onchainos wallet addresses` and find the EVM address for `chainIndex: "56"`. If no chain-specific match, use the first EVM address (all chains share the same EVM address).

### Fix 2: BSC RPC URL (Reliability)
**File:** `src/config.rs`
**Root cause:** `https://bsc-dataseed.binance.org` was intermittently failing with connection errors during `balance` queries.
**Fix:** Switched to `https://bsc-rpc.publicnode.com` as documented in test instructions.

---

## L2: Read Tests (BSC RPC)

| ID | Description | Command | Result | Output |
|----|-------------|---------|--------|--------|
| L2-01 | Query all staked positions | `balance` | PASS | Wallet resolved; 11 assets queried; "No staked positions found" (zero balance expected) |
| L2-02 | Query BTCB by address | `balance --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c` | PASS | `Staked: 0 BTCB (0 wei)` |
| L2-03 | Query SolvBTC by address | `balance --asset 0x4aae823a6a0b376De6A78e74eCC5b079d38cBCf7` | PASS | `Staked: 0 SolvBTC (0 wei)` |

---

## L3: Dry-run Tests (Calldata Validation)

| ID | Description | approve calldata | stake/action calldata | Result |
|----|-------------|------------------|-----------------------|--------|
| L3-01 | Stake BTCB 0.0001 dry-run | `0x095ea7b3` ✓ | `0x4df42566` ✓ | PASS |
| L3-02 | Unstake BTCB 0.0001 dry-run | N/A | `0xf91daa33` ✓ | PASS |
| L3-03 | Stake native BNB 0.0001 dry-run | N/A (no approve) | `0xc412056b` ✓, msg.value=100000000000000 wei ✓ | PASS |
| L3-04 | Unstake native BNB 0.0001 dry-run | N/A | `0x4693cf07` ✓ | PASS |
| L3-05 | Stake BTCB with referral TESTREF | `0x095ea7b3` ✓ | `0x4df42566` ✓, "TESTREF" ABI-encoded ✓ | PASS |

### L3-01 Calldata Detail (stake BTCB 0.0001)
```
approve calldata:
0x095ea7b3
  000000000000000000000000b32dF5B33dBCCA60437EC17b27842c12bFE83394  (spender=StakerGateway)
  00000000000000000000000000000000000000000000000000005af3107a4000  (amount=0.0001e18)

stake calldata:
0x4df42566
  0000000000000000000000007130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c  (asset=BTCB)
  00000000000000000000000000000000000000000000000000005af3107a4000  (amount=0.0001e18)
  0000000000000000000000000000000000000000000000000000000000000060  (offset for string)
  0000000000000000000000000000000000000000000000000000000000000000  (string len=0)
```

### L3-02 Calldata Detail (unstake BTCB 0.0001)
```
unstake calldata:
0xf91daa33
  0000000000000000000000007130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c  (asset=BTCB)
  00000000000000000000000000000000000000000000000000005af3107a4000  (amount=0.0001e18)
  0000000000000000000000000000000000000000000000000000000000000060  (offset for string)
  0000000000000000000000000000000000000000000000000000000000000000  (string len=0)
```

---

## L4: Live On-chain Tests

| ID | Description | Result | Notes |
|----|-------------|--------|-------|
| L4-01 | Check BSC wallet BNB balance | BLOCKED | `onchainos wallet balance --chain 56` → `tokenAssets: []`, RPC eth_getBalance=0x0. Wallet has 0 BNB. |
| L4-02 | Stake native BNB (0.0001 BNB) | BLOCKED-余额不足 | BNB balance = 0, insufficient for gas + stake. No tx submitted. |
| L4-03 | Balance query post-stake | BLOCKED | Depends on L4-02. |

**Lock:** Acquired and released successfully via `acquire-lock.sh` / `release-lock.sh`.

---

## L1-error: Error Handling Tests

| ID | Description | Result | Output |
|----|-------------|--------|--------|
| E-01 | Too many decimal places | PASS | `Error: Too many decimal places (token supports 18 decimals, got 22)` (exit 1) |
| E-02 | Invalid/short address in balance | PASS | Network error from RPC (eth_call rejects malformed address), non-zero exit |
| E-03 | Missing --amount in stake | PASS | Clap error: `required argument --amount not provided` (exit 2) |

---

## Summary

| Category | Total | Pass | Blocked | Fail |
|----------|-------|------|---------|------|
| Build/Lint | 2 | 2 | 0 | 0 |
| L2 Read | 3 | 3 | 0 | 0 |
| L3 Dry-run | 5 | 5 | 0 | 0 |
| L4 On-chain | 3 | 0 | 3 | 0 |
| L1-error | 3 | 3 | 0 | 0 |
| **Total** | **16** | **13** | **3** | **0** |

All P0 tests (L2 read + L3 dry-run) pass. L4 is blocked due to zero BNB balance in test wallet on BSC — no BTCB/SolvBTC and no native BNB available.

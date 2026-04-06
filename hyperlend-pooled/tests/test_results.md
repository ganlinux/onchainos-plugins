# hyperlend-pooled — Test Results

**Date:** 2026-04-05
**Chain:** HyperEVM (chain ID 999)
**RPC:** https://rpc.hyperlend.finance
**Plugin version:** 0.1.0
**Tester:** Tester Agent (Claude Sonnet 4.6)

---

## L1: Compile + Lint

```
cargo clean && plugin-store lint .
```

**Result: PASS**

- `cargo build --release` — zero errors, zero warnings
- `plugin-store lint .` — output: `✓ Plugin 'hyperlend-pooled' passed all checks!`

---

## L0: Skill Routing

**Result: PASS** — see `tests/routing_test.md` for full analysis.

- SKILL.md exists at `skills/hyperlend-pooled/SKILL.md` ✅
- All required trigger phrases covered: HyperLend markets, supply, borrow, repay, withdraw, health factor ✅
- Negative triggers correctly excluded: Aave mainnet, HyperLend Isolated Pools, HyperLend P2P Pools ✅

---

## L2: Read Tests

### get-markets

```bash
./target/release/hyperlend-pooled get-markets
```

**Result: PASS**

- Returns `ok: true`, `marketCount: 17`, `markets` array
- Each entry has: `symbol`, `supplyApy`, `borrowApy`, `utilizationRate`, `underlyingAsset`, `totalSupply`, `totalVariableDebt`, `availableLiquidity`, `borrowingEnabled`, `isActive`, `isFrozen`, `ltv`, `liquidationThreshold`, `decimals`
- `--active-only` flag works: returns 16 active markets (1 frozen filtered out)
- Live data fetched from `https://api.hyperlend.finance/data/markets?chain=hyperEvm`

### positions (zero wallet)

```bash
./target/release/hyperlend-pooled positions --from 0x0000000000000000000000000000000000000001
```

**Result: PASS**

- Returns `ok: true`
- `supplied: []`, `borrowed: []` (zero address has no positions)
- `healthFactor: "0.0000"`, `healthFactorStatus: "no-debt"` ✅
- `accountSummary` includes `totalCollateralUsd: 0.0`, `totalDebtUsd: 0.0`

**Note:** HyperEVM RPC at `https://rpc.hyperlend.finance` enforces a rate limit (error -32005). Consecutive calls within ~90 seconds hit the limit. A 2-minute cooldown resolved it. The `positions` command makes ~34 eth_call requests (2 per market × 17 markets); rate limiting is expected under heavy test conditions.

---

## L3: Dry-Run (Selector Verification)

### supply

```bash
./target/release/hyperlend-pooled --dry-run supply \
  --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
```

**Result: PASS**

- Step 1 (approve): selector `0x095ea7b3` ✅, target = asset address ✅
- Step 2 (Pool.supply): selector `0x617ba037` ✅, target = Pool `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` ✅
- ABI encoding: asset, amount, onBehalfOf=zero, referralCode=0 — correct 4-param layout

### borrow

```bash
./target/release/hyperlend-pooled --dry-run borrow \
  --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
```

**Result: PASS**

- Selector `0xa415bcad` ✅
- ABI: asset, amount, interestRateMode=2 (variable), referralCode=0, onBehalfOf — correct 5-param layout
- No approval step (correct for borrow)

### repay

```bash
./target/release/hyperlend-pooled --dry-run repay \
  --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
```

**Result: PASS**

- Step 1 (approve): selector `0x095ea7b3` ✅
- Step 2 (Pool.repay): selector `0x573ade81` ✅
- ABI: asset, amount, interestRateMode=2, onBehalfOf — correct 4-param layout
- Note in output warns against using u128::MAX for repay-all ✅

### withdraw

```bash
./target/release/hyperlend-pooled --dry-run withdraw \
  --asset 0x5555555555555555555555555555555555555555 --amount 1000000000000000
```

**Result: PASS**

- Selector `0x69328dec` ✅
- ABI: asset, amount, to — correct 3-param layout
- No approval step (correct for withdraw)
- `withdrawAll: false` for non-zero amount ✅

### Selector Summary

| Operation | Expected | Actual | Status |
|-----------|----------|--------|--------|
| approve   | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| supply    | `0x617ba037` | `0x617ba037` | ✅ |
| borrow    | `0xa415bcad` | `0xa415bcad` | ✅ |
| repay     | `0x573ade81` | `0x573ade81` | ✅ |
| withdraw  | `0x69328dec` | `0x69328dec` | ✅ |

---

## L4: On-Chain Tests

```bash
onchainos wallet balance --chain 999
```

**Result: BLOCKED**

- `onchainos wallet balance --chain 999` returns `{"ok": false, "error": "unknown chain: 999"}`
- Checked via `onchainos wallet balance` (no chain filter): wallet holds tokens on Base (8453), Solana (501), Arbitrum (42161) — **no HyperEVM (999) balance**
- No HYPE available for gas → on-chain supply/withdraw cycle skipped

---

## Overall Summary

| Level | Test | Result |
|-------|------|--------|
| L1 | Compile + Lint | ✅ PASS |
| L0 | Skill Routing | ✅ PASS |
| L2 | get-markets | ✅ PASS |
| L2 | positions (zero wallet) | ✅ PASS |
| L3 | supply dry-run | ✅ PASS |
| L3 | borrow dry-run | ✅ PASS |
| L3 | repay dry-run | ✅ PASS |
| L3 | withdraw dry-run | ✅ PASS |
| L4 | On-chain transactions | ⛔ BLOCKED (no HyperEVM wallet balance) |

**Overall: PASS (L4 BLOCKED)**

All selector hashes match Aave V3 standard. ABI encodings are correct. Live RPC integration works (subject to rate limiting). The plugin is ready for production pending an on-chain smoke test when HyperEVM funds are available.

---

## Issues Found

1. **RPC Rate Limiting** — `positions` makes 34+ sequential eth_call requests, which triggers the HyperLend RPC rate limit (error -32005) under concurrent test conditions. This is a protocol-side limitation, not a plugin bug. Consider batching calls via multicall in a future iteration.

2. **`onchainos wallet balance --chain 999` unsupported** — The onchainos CLI does not recognize chain ID 999 by number. Users must rely on the default `wallet balance` and look for chain 999 entries manually. Not a blocker; SKILL.md documents checking wallet status via `onchainos wallet status`.

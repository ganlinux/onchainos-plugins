# Fenix Finance Plugin — Test Cases

**Plugin**: fenix-finance
**Chain**: Blast (chain ID 81457)
**Date**: 2026-04-05
**Protocol**: Fenix Finance V3 (Algebra Integral V1 AMM)

---

## L0 — Skill Routing

| ID | User Intent | Expected Skill | Notes |
|----|-------------|----------------|-------|
| R01 | "swap WETH to USDB on Fenix" | fenix-finance | Core trigger phrase |
| R02 | "add liquidity on Fenix DEX" | fenix-finance | LP trigger |
| R03 | "show my Fenix positions" | fenix-finance | Positions trigger |
| R04 | "remove liquidity from Fenix" | fenix-finance | Remove LP trigger |
| R05 | "swap on Uniswap" | uniswap (NOT fenix-finance) | Negative: different DEX |
| R06 | "Thruster Finance swap on Blast" | thruster (NOT fenix-finance) | Negative: different Blast DEX |
| R07 | "concentrated liquidity Blast" | fenix-finance | Blast DEX trigger |
| R08 | "Algebra AMM Blast swap" | fenix-finance | Algebra trigger |
| R09 | "Fenix LP positions" | fenix-finance | LP positions trigger |
| R10 | "FNX token swap" | fenix-finance | FNX token reference |

---

## L1 — Compile + Lint

| ID | Check | Expected |
|----|-------|----------|
| C01 | `cargo build --release` | Exits 0, binary produced at `target/release/fenix-finance` |
| C02 | `plugin-store lint .` | No errors; warnings acceptable |
| C03 | `cargo clean && plugin-store lint .` | Clean lint pass |

---

## L2 — Read Commands (On-chain, no wallet needed)

### TC-PRICE-01: WETH → USDB price quote

```bash
./target/release/fenix-finance price --token-in WETH --token-out USDB --amount 0.01
```

**Expected**:
- Exit code 0
- JSON with `"ok": true`
- `amount_out_human` is a positive decimal number (USDB has 18 decimals like WETH)
- `pool` field contains a non-zero address
- Price should be roughly 0.01 ETH worth of USDB (~$25–$40 range)

### TC-PRICE-02: USDB → WETH reverse quote

```bash
./target/release/fenix-finance price --token-in USDB --token-out WETH --amount 100
```

**Expected**:
- Exit code 0
- JSON with `"ok": true`
- `amount_out_human` is a small decimal (100 USDB → ~0.03–0.05 WETH)

### TC-PRICE-03: Invalid pair (no pool)

```bash
./target/release/fenix-finance price --token-in FNX --token-out BLAST --amount 1
```

**Expected**:
- Exit code 0
- JSON with `"ok": false` and `"error": "Pool does not exist for this pair"`
  OR a valid quote if pool exists

### TC-POS-01: Positions for zero wallet (no positions expected)

```bash
./target/release/fenix-finance positions --owner 0x0000000000000000000000000000000000000001
```

**Expected**:
- Exit code 0
- JSON: `{"ok": true, "positions": [], "count": 0}`
- Source may be "subgraph" or "onchain"

### TC-POS-02: Positions on-chain fallback

```bash
./target/release/fenix-finance positions --owner 0x0000000000000000000000000000000000000001 --onchain
```

**Expected**:
- Exit code 0
- JSON: `{"ok": true, "positions": [], "count": 0, "source": "onchain"}`

---

## L3 — Dry-run (Selector Verification)

### TC-DRY-01: Swap dry-run

```bash
./target/release/fenix-finance swap --token-in WETH --token-out USDB --amount 0.001 --dry-run
```

**Expected**:
- Exit code 0
- `"dry_run": true`
- `"selector": "0xbc651188"` (exactInputSingle)
- `"swap_router": "0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00"`

### TC-DRY-02: Add liquidity dry-run

```bash
./target/release/fenix-finance add-liquidity --token0 WETH --token1 USDB --amount0 0.001 --amount1 1 --dry-run
```

**Expected**:
- Exit code 0
- `"dry_run": true`
- `"selector": "0x9cc1a283"` (mint)
- `"nfpm": "0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd"`

### TC-DRY-03: Remove liquidity dry-run

```bash
./target/release/fenix-finance remove-liquidity --token-id 1 --dry-run
```

**Expected**:
- Exit code 0
- `"dry_run": true`
- `"selectors"` contains `"decreaseLiquidity": "0x0c49ccbe"` and `"collect": "0xfc6f7865"`

---

## L4 — On-chain (Requires funded wallet)

### TC-LIVE-01: Balance check

```bash
onchainos wallet balance --chain 81457
```

**Expected**: Shows ETH and any WETH/USDB/FNX balances on Blast

### TC-LIVE-02: Real swap (only if wallet has WETH)

```bash
./target/release/fenix-finance swap --token-in WETH --token-out USDB --amount 0.001
```

**Expected**:
- Prints approve tx hash (if needed)
- Prints swap result JSON with `"ok": true` and `"tx_hash"`

### TC-LIVE-03: Balance after swap

```bash
./target/release/fenix-finance balance
```

**Expected**: USDB balance increased

---

## Contract Address Verification

| Contract | Expected Address |
|----------|-----------------|
| SwapRouter | `0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00` |
| QuoterV2 | `0x94Ca5B835186A37A99776780BF976fAB81D84ED8` |
| AlgebraFactory | `0x7a44CD060afC1B6F4c80A2B9b37f4473E74E25Df` |
| NFPM | `0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd` |
| WETH | `0x4300000000000000000000000000000000000004` |
| USDB | `0x4300000000000000000000000000000000000003` |

---

## Selector Verification

| Operation | Function | Expected Selector |
|-----------|----------|------------------|
| swap | `exactInputSingle` | `0xbc651188` |
| add-liquidity | `mint` | `0x9cc1a283` |
| remove-liquidity | `decreaseLiquidity` | `0x0c49ccbe` |
| collect | `collect` | `0xfc6f7865` |
| ERC-20 approve | `approve` | `0x095ea7b3` |

---

## Notes

- The `price` command uses `--amount` (not `--amount-in`); SKILL.md shows different arg names in some examples — this discrepancy should be noted.
- The `swap` command uses `--amount` (not `--amount-in` as in the task description).
- All tokens (WETH, USDB, BLAST, FNX) use 18 decimals.
- Algebra V1 has no fee tier — one pool per token pair.
- The plugin uses QuoterV2 selector `0x5e5e6e0f` for `quoteExactInputSingle`.

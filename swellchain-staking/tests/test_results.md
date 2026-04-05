# Test Results — swellchain-staking

Date: 2026-04-05
Tester: Tester Agent (Pipeline)
Plugin: swellchain-staking v0.1.0
Chain: Ethereum Mainnet (chain_id=1)

---

## Summary

| Level | Status | Notes |
|-------|--------|-------|
| L1: Compile + Lint | PASS | 1 dead_code warning (non-blocking) |
| L0: Skill Routing | PASS | SKILL.md routing is distinctive and correct |
| L2: Read Tests | PASS | balance, positions return valid data with live rates |
| L3: Dry-run Tests | PASS | All 6 commands verified; all selectors correct |
| L4: On-chain Tests | BLOCKED | Wallet has 0 ETH and 0 tokens on chain 1 |

**Overall: PASS (L4 BLOCKED due to empty wallet)**

---

## Level 1: Compile + Lint

```
cargo build --release   → success (1 dead_code warning for unused `decode_address` fn)
plugin-store lint .     → "Plugin 'swellchain-staking' passed all checks!"
```

**Warning (non-blocking):** `src/rpc.rs:78` — `pub fn decode_address` is defined but never called.

---

## Level 0: Skill Routing Validation

SKILL.md reviewed. Key routing keywords confirmed:
- Positive: swETH, rswETH, swEXIT, SimpleStakingERC20, Swell Earn, Swell Network, swellchain staking
- Negative exclusions: Lido stETH, Rocketpool rETH, EigenLayer direct
- Boundary: Distinguished from PR #141 (Swell Staking) and PR #179 (Swell Restaking) by Earn pool operations

Full routing test matrix written to `tests/routing_test.md`.

---

## Level 2: Read Tests (No Gas)

### TC-L2-01: balance

```bash
./target/release/swellchain-staking balance --address 0xf951E335afb289353dc249e82926178EaC7DEd78
```

**Result: PASS**

```json
{
  "ok": true,
  "address": "0xf951E335afb289353dc249e82926178EaC7DEd78",
  "swETH": {
    "balance": "0.980887",
    "swETHToETHRate": "1.119040",
    "ethToSwETHRate": "0.893622",
    "eth_equivalent": "1.097653"
  },
  "rswETH": {
    "balance": "0.069729",
    "rswETHToETHRate": "1.069053",
    "ethToRswETHRate": "0.935406",
    "eth_equivalent": "0.074544"
  }
}
```

Key fields verified: `swETHToETHRate`, `ethToSwETHRate`, `rswETHToETHRate`, `ethToRswETHRate` all > 0.

### TC-L2-02: positions

```bash
./target/release/swellchain-staking positions --address 0xf951E335afb289353dc249e82926178EaC7DEd78
```

**Result: PASS**

```json
{
  "ok": true,
  "liquid_staking": {
    "swETH": { "rate_sweth_per_eth": "1.119040", "apr_note": "~3% APR (repricing token)" },
    "rswETH": { "rate_rsweth_per_eth": "1.069053", "apr_note": "~2.63% APR + EigenLayer restaking rewards" }
  },
  "earn_pool": { "contract": "0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774" },
  "withdrawals": {
    "last_token_id_created": "30694",
    "last_token_id_processed": "30634",
    "pending_requests": "60"
  }
}
```

Key fields verified: rates present, withdrawals section shows live swEXIT state.

---

## Level 3: Dry-run Tests

### TC-L3-01: stake --dry-run

**Result: PASS**
- `calldata: "0xd0e30db0"` — correct, 4-byte only (no params for `deposit()`)
- `contract: "0xf951E335afb289353dc249e82926178EaC7DEd78"` — correct swETH proxy

### TC-L3-02: earn-deposit swETH --dry-run

**Result: PASS**
- `step1_approve_calldata` starts with `0x095ea7b3` ✓
- `step2_deposit_calldata` starts with `0xf45346dc` ✓
- `token_addr: "0xf951E335afb289353dc249e82926178EaC7DEd78"` ✓
- `staking_contract: "0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774"` ✓

**Note:** Dry-run deposit calldata uses literal `<wallet>` placeholder for receiver — acceptable for dry-run preview.

### TC-L3-03: earn-deposit rswETH --dry-run

**Result: PASS**
- `token_addr: "0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0"` ✓

### TC-L3-04: request-withdrawal --dry-run

**Result: PASS**
- `step1_approve_calldata` starts with `0x095ea7b3` ✓
- `step2_create_withdraw_request_calldata` starts with `0x74dc9d1a` ✓
- `swexit_contract: "0x48C11b86807627AF70a34662D4865cF854251663"` ✓

### TC-L3-05: finalize-withdrawal --token-id 1 --dry-run

**Result: PASS**
- `calldata: "0x5e15c749..."` ✓
- `contract: "0x48C11b86807627AF70a34662D4865cF854251663"` ✓

### TC-L3-06: earn-withdraw --dry-run

**Result: PASS**
- `calldata` starts with `0x69328dec` ✓
- `staking_contract: "0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774"` ✓

### TC-ERR-01: earn-deposit invalid token

**Result: PASS**
- Returns exit code 1 with message: `Error: Unsupported token 'ETH'. Use 'swETH' or 'rswETH'.`

---

## Selector Verification Summary

| Operation | Selector | Design.md | Verified |
|-----------|----------|-----------|---------|
| stake (deposit()) | `0xd0e30db0` | ✓ | ✓ |
| earn-deposit (deposit(address,uint256,address)) | `0xf45346dc` | ✓ | ✓ |
| earn-withdraw (withdraw(address,uint256,address)) | `0x69328dec` | ✓ | ✓ |
| request-withdrawal (createWithdrawRequest(uint256)) | `0x74dc9d1a` | ✓ | ✓ |
| finalize-withdrawal (finalizeWithdrawal(uint256)) | `0x5e15c749` | ✓ | ✓ |
| approve (ERC-20) | `0x095ea7b3` | ✓ | ✓ |

---

## Level 4: On-chain Tests

**Wallet balance:** 0 ETH, 0 tokens (empty wallet)

| Test | Status | Reason |
|------|--------|--------|
| TC-L4-01: stake | BLOCKED | Wallet has < 0.001 ETH |
| TC-L4-02: earn-deposit swETH | BLOCKED | Wallet has no swETH |
| TC-L4-03: earn-withdraw swETH | BLOCKED | Wallet has no swETH in Earn pool |

---

## Issues Found

| # | Severity | Description | Status |
|---|----------|-------------|--------|
| 1 | WARNING | Dead code: `decode_address` in `src/rpc.rs:78` never called | Non-blocking (lint passes) |
| 2 | MINOR | CLI uses `--amt` but SKILL.md examples use `--amt` — consistent; pipeline prompt uses `--amount` incorrectly | No fix needed (binary is correct) |
| 3 | MINOR | earn-deposit/earn-withdraw dry-run calldata uses literal `<wallet>` as receiver placeholder | Acceptable for dry-run; live mode uses real wallet address |

---

## Fix Loop

No errors requiring fix. All tests pass at L1-L3 level. Dead-code warning is non-blocking.

---

## Artifacts

- `tests/routing_test.md` — L0 routing validation matrix
- `tests/test_cases.md` — full test case specification
- `tests/test_results.md` — this report

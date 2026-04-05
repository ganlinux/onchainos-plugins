# Gravita Protocol — Test Cases

Generated: 2026-04-05
Chains tested: Ethereum (1), Linea (59144)
Contracts:
  - BorrowerOperations (ETH): 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2
  - VesselManager (ETH): 0xdB5DAcB1DFbe16326C3656a88017f0cB4ece0977

---

## L1 — Build & Lint

| ID | Test | Expected |
|----|------|----------|
| L1-01 | `cargo build --release` | Exits 0, binary produced |
| L1-02 | `plugin-store lint .` | 0 errors |

---

## L2 — Read Tests (Ethereum RPC, no wallet needed for calldata; wallet needed for position)

| ID | Command | Expected |
|----|---------|----------|
| L2-01 | `position --collateral wstETH --chain 1` | JSON output or stdout with vessel status, collateral, debt fields |
| L2-02 | `position --collateral rETH --chain 1` | Vessel status for rETH |
| L2-03 | `position --collateral WETH --chain 1` | Vessel status for WETH |
| L2-04 | `position --collateral wstETH --chain 59144` | Vessel status on Linea |
| L2-05 | `position --collateral UNSUPPORTED --chain 1` | Error: collateral not supported |
| L2-06 | `position --collateral wstETH --chain 999` | Error: unsupported chain |

---

## L3 — Dry-run Tests (calldata validation)

### open

| ID | Command | Expected Calldata Prefix |
|----|---------|--------------------------|
| L3-01 | `open --collateral wstETH --coll-amount 0.001 --debt-amount 100 --chain 1 --dry-run` | approve: `0x095ea7b3`, openVessel: `0xd92ff442` |
| L3-02 | `open --collateral rETH --coll-amount 0.5 --debt-amount 2000 --chain 1 --dry-run` | approve: `0x095ea7b3`, openVessel: `0xd92ff442` |
| L3-03 | `open --collateral wstETH --coll-amount 1.0 --debt-amount 2000 --chain 59144 --dry-run` | approve: `0x095ea7b3`, openVessel: `0xd92ff442` |

### close

| ID | Command | Expected Calldata Prefix |
|----|---------|--------------------------|
| L3-04 | `close --collateral wstETH --chain 1 --dry-run` | approve (GRAI): `0x095ea7b3`, closeVessel: `0xe687854f` |
| L3-05 | `close --collateral rETH --chain 1 --dry-run` | approve (GRAI): `0x095ea7b3`, closeVessel: `0xe687854f` |

### adjust

| ID | Command | Expected Calldata Prefix |
|----|---------|--------------------------|
| L3-06 | `adjust --collateral wstETH --action add-coll --amount 0.5 --chain 1 --dry-run` | approve: `0x095ea7b3`, adjustVessel: `0x...` |
| L3-07 | `adjust --collateral wstETH --action repay --amount 1000 --chain 1 --dry-run` | approve GRAI: `0x095ea7b3`, adjustVessel: `0x...` |
| L3-08 | `adjust --collateral wstETH --action borrow --amount 500 --chain 1 --dry-run` | adjustVessel (no approve): `0x...` |
| L3-09 | `adjust --collateral wstETH --action withdraw-coll --amount 0.1 --chain 1 --dry-run` | adjustVessel (no approve): `0x...` |

---

## L4 — On-chain Tests (requires wallet with assets)

| ID | Test | Required Balance | Expected |
|----|------|-----------------|----------|
| L4-01 | `open --collateral wstETH --coll-amount 0.001 --debt-amount 2000 --chain 1` | wstETH >= 0.001, ETH for gas | Vessel opened, txHash returned |
| L4-02 | `position --collateral wstETH --chain 1` (after L4-01) | - | Status=Active, debt shown |
| L4-03 | `adjust --collateral wstETH --action borrow --amount 500 --chain 1` | Active Vessel | debt increases by 500 GRAI |
| L4-04 | `close --collateral wstETH --chain 1` | GRAI >= full debt | Vessel closed, collateral returned |

---

## Selector Reference

| Function | Selector |
|----------|----------|
| ERC-20 `approve(address,uint256)` | `0x095ea7b3` |
| `openVessel(address,uint256,uint256,address,address)` | `0xd92ff442` |
| `closeVessel(address)` | `0xe687854f` |
| `adjustVessel(address,uint256,uint256,uint256,bool,address,address)` | check on-chain |

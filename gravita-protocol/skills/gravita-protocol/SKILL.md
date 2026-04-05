---
name: gravita-protocol
description: >-
  Use when the user asks about Gravita Protocol, borrowing GRAI stablecoin, CDP Vessels,
  'open a Vessel', 'borrow GRAI', 'deposit wstETH Gravita', 'deposit rETH Gravita',
  'deposit WETH Gravita', 'close Vessel', 'repay GRAI', 'add collateral Gravita',
  'withdraw collateral Gravita', 'Gravita position', 'Gravita debt',
  'Gravita collateral ratio', 'Gravita ICR', 'Gravita liquidation',
  'interest-free borrow', 'LST collateral stablecoin',
  or mentions Gravita, GRAI, Gravita Vessel, Gravita CDP on Ethereum or Linea.
  Covers: open Vessel (deposit collateral + borrow GRAI), adjust Vessel
  (add/withdraw collateral, borrow/repay GRAI), close Vessel, and position queries.
  Do NOT use for Liquity, MakerDAO, Aave, or other lending protocols.
  Do NOT use for non-GRAI stablecoins.
license: MIT
metadata:
  author: skylavis-sky
  version: "0.1.0"
---

# Gravita Protocol Plugin

Gravita Protocol is a decentralized CDP (Collateralized Debt Position) borrowing protocol. Users deposit LST collateral (wstETH, rETH, WETH on Ethereum; wstETH on Linea) to borrow GRAI — a USD-pegged stablecoin — at 0% annual interest. Only a one-time borrowing fee is charged (0% to 10%, pro-rata refunded if closed within 6 months).

**Key facts:**
- Each address can have one Vessel per collateral type
- Minimum borrow: ~2,000 GRAI (checked on-chain via AdminContract)
- 200 GRAI gas compensation is locked at open, returned at close
- MCR (minimum collateral ratio): ~111% for wstETH/rETH/WETH

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command. Always follow in order.

1. **Check onchainos**: `which onchainos` — if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet; if found, run `onchainos --version` and verify version is **>= 2.0.0**
2. **Check binary**: `which gravita-protocol` — if not found, install via `plugin-store install gravita-protocol`
3. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true`; if not, run `onchainos wallet login`
4. **For write operations**: verify sufficient collateral balance and GRAI balance as needed

## Supported Chains and Collaterals

| Chain | Chain ID | Collaterals |
|-------|----------|-------------|
| Ethereum | 1 | WETH (90% max LTV), wstETH (85%), rETH (85%) |
| Linea | 59144 | wstETH (85%) |

## Commands

### `gravita-protocol position` — Query Vessel Status

**Triggers:** "my Gravita position", "Gravita debt", "Vessel status", "Gravita collateral", "how much GRAI borrowed", "Gravita ICR"

```bash
gravita-protocol position --chain <CHAIN_ID> --collateral <SYMBOL>
```

**Parameters:**
- `--chain <ID>` — Chain ID: `1` (Ethereum, default) or `59144` (Linea)
- `--collateral <SYMBOL>` — Collateral symbol: `wstETH`, `rETH`, `WETH`

**Examples:**
```bash
# Query wstETH Vessel on Ethereum
gravita-protocol position --chain 1 --collateral wstETH

# Query wstETH Vessel on Linea
gravita-protocol position --chain 59144 --collateral wstETH
```

Output: Vessel status, collateral locked, GRAI debt, MCR, one-time borrowing fee.

---

### `gravita-protocol open` — Open a New Vessel

**Triggers:** "open Gravita Vessel", "borrow GRAI", "deposit wstETH Gravita", "deposit rETH borrow GRAI", "start Gravita position"

```bash
gravita-protocol open --chain <CHAIN_ID> --collateral <SYMBOL> --coll-amount <AMOUNT> --debt-amount <GRAI_AMOUNT>
gravita-protocol --dry-run open --chain 1 --collateral wstETH --coll-amount 1.0 --debt-amount 2000.0
```

**Parameters:**
- `--chain <ID>` — Chain ID (default: `1`)
- `--collateral <SYMBOL>` — Collateral symbol
- `--coll-amount <AMOUNT>` — Collateral to deposit (e.g. `1.0` for 1 wstETH)
- `--debt-amount <AMOUNT>` — GRAI to borrow (minimum ~2000)

**Constraints:**
- Minimum debt: ~2,000 GRAI (enforced by AdminContract.getMinNetDebt on-chain)
- Vessel must not already exist for this collateral (use `adjust` to modify)
- Uses address(0) hints for SortedVessels (safe but higher gas)

**Steps performed:**
1. ERC-20 approve collateral -> BorrowerOperations (ask user to confirm)
2. Wait 5 seconds (nonce safety)
3. openVessel (ask user to confirm)

**Examples:**
```bash
# Open with 1 wstETH, borrow 2000 GRAI on Ethereum
gravita-protocol open --chain 1 --collateral wstETH --coll-amount 1.0 --debt-amount 2000.0

# Dry-run simulation
gravita-protocol --dry-run open --chain 1 --collateral rETH --coll-amount 0.5 --debt-amount 2000.0

# Open on Linea
gravita-protocol open --chain 59144 --collateral wstETH --coll-amount 1.0 --debt-amount 2000.0
```

---

### `gravita-protocol adjust` — Adjust an Existing Vessel

**Triggers:** "add collateral Gravita", "deposit more wstETH", "withdraw collateral Gravita", "borrow more GRAI", "repay GRAI", "partial repay Gravita"

```bash
gravita-protocol adjust --chain <CHAIN_ID> --collateral <SYMBOL> --action <ACTION> --amount <AMOUNT>
gravita-protocol --dry-run adjust --chain 1 --collateral wstETH --action add-coll --amount 0.5
```

**Parameters:**
- `--chain <ID>` — Chain ID (default: `1`)
- `--collateral <SYMBOL>` — Collateral symbol
- `--action <ACTION>` — One of: `add-coll`, `withdraw-coll`, `borrow`, `repay`
- `--amount <AMOUNT>` — Amount (collateral units for add-coll/withdraw-coll; GRAI for borrow/repay)

**Action details:**

| Action | Requires approve | Description |
|--------|-----------------|-------------|
| `add-coll` | Yes (collateral) | Deposit more collateral to increase safety |
| `withdraw-coll` | No | Remove collateral (increases liquidation risk) |
| `borrow` | No | Mint more GRAI (increases debt) |
| `repay` | Yes (GRAI) | Repay partial GRAI debt (no close) |

**Note:** Each write action will ask user to confirm before broadcasting.

**Examples:**
```bash
# Add 0.5 wstETH collateral on Ethereum (ask user to confirm)
gravita-protocol adjust --chain 1 --collateral wstETH --action add-coll --amount 0.5

# Repay 1000 GRAI (ask user to confirm)
gravita-protocol adjust --chain 1 --collateral wstETH --action repay --amount 1000.0

# Borrow 500 more GRAI (ask user to confirm)
gravita-protocol adjust --chain 1 --collateral rETH --action borrow --amount 500.0

# Withdraw 0.1 wstETH collateral (ask user to confirm)
gravita-protocol adjust --chain 59144 --collateral wstETH --action withdraw-coll --amount 0.1

# Dry-run
gravita-protocol --dry-run adjust --chain 1 --collateral wstETH --action add-coll --amount 0.5
```

---

### `gravita-protocol close` — Close a Vessel

**Triggers:** "close Gravita Vessel", "repay all GRAI", "exit Gravita", "get collateral back Gravita", "close CDP Gravita"

```bash
gravita-protocol close --chain <CHAIN_ID> --collateral <SYMBOL>
gravita-protocol --dry-run close --chain 1 --collateral wstETH
```

**Parameters:**
- `--chain <ID>` — Chain ID (default: `1`)
- `--collateral <SYMBOL>` — Collateral symbol

**Steps performed:**
1. Query current debt (getVesselDebt)
2. ERC-20 approve GRAI -> BorrowerOperations for full debt amount (ask user to confirm)
3. Wait 5 seconds
4. closeVessel (ask user to confirm)

**Important:** You must hold at least the full GRAI debt amount (including 200 GRAI gas compensation) to close the Vessel. Check your GRAI balance first.

**Examples:**
```bash
# Close wstETH Vessel on Ethereum (ask user to confirm)
gravita-protocol close --chain 1 --collateral wstETH

# Close rETH Vessel (ask user to confirm)
gravita-protocol close --chain 1 --collateral rETH

# Dry-run to see calldata
gravita-protocol --dry-run close --chain 1 --collateral wstETH
```

---

## Contract Addresses Reference

### Ethereum (chain 1)
| Contract | Address |
|----------|---------|
| BorrowerOperations | `0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2` |
| VesselManager | `0xdB5DAcB1DFbe16326C3656a88017f0cB4ece0977` |
| AdminContract | `0xf7Cc67326F9A1D057c1e4b110eF6c680B13a1f53` |
| GRAI | `0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4` |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` |
| wstETH | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` |
| rETH | `0xae78736Cd615f374D3085123A210448E74Fc6393` |

### Linea (chain 59144)
| Contract | Address |
|----------|---------|
| BorrowerOperations | `0x40E0e274A42D9b1a9D4B64dC6c46D21228d45C20` |
| VesselManager | `0xdC44093198ee130f92DeFed22791aa8d8df7fBfA` |
| AdminContract | `0xC8a25eA0Cbd92A6F787AeED8387E04559053a9f8` |
| GRAI | `0x894134a25a5faC1c2C26F1d8fBf05111a3CB9487` |
| wstETH | `0xB5beDd42000b71FddE22D3eE8a79Bd49A568fC8F` |

## Error Handling

| Error | Cause | Fix |
|-------|-------|-----|
| "No active Vessel" | No open Vessel for this collateral | Use `open` to create one |
| "Vessel already active" | Vessel exists | Use `adjust` to modify |
| "below minimum" | Debt < ~2000 GRAI | Increase debt-amount |
| "insufficient GRAI" | Not enough GRAI for closeVessel | Acquire more GRAI before closing |
| "execution reverted" | Various on-chain errors | Check collateral ratio, balances, and allowance |

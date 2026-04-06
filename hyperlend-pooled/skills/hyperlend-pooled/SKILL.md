---
name: hyperlend-pooled
description: >-
  Use when the user asks about HyperLend, HyperLend Core Pools, lending on HyperEVM,
  'supply to HyperLend', 'borrow from HyperLend', 'repay HyperLend', 'withdraw HyperLend',
  'HyperLend position', 'HyperLend health factor', 'HyperLend markets', 'HyperLend rates',
  'HyperLend APY', 'wHYPE collateral', 'borrow USDC HyperLend', 'HyperLend Aave',
  or mentions HyperLend, HyperLend Core, hyperlend-core, HyperEVM lending, Aave fork on Hyperliquid.
  Covers: viewing Core Pool markets, checking supply/borrow positions, supplying assets,
  borrowing assets, repaying debt, and withdrawing supplied assets on Hyperliquid EVM (chain 999).
  Do NOT use for HyperLend Isolated Pools or HyperLend P2P Pools (separate plugins).
  Do NOT use for general Aave operations on other chains.
license: MIT
metadata:
  author: ganlinux
  version: "0.1.0"
---

# HyperLend Core Pools Plugin

HyperLend Core Pools is an Aave V3.2 fork on Hyperliquid EVM (chain ID 999). Users can supply
assets as collateral to earn variable yield (hTokens), borrow against collateral, repay debt, and
withdraw supplied assets.

**Key facts:**
- Chain: HyperEVM (chain ID 999)
- Pool contract: `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b`
- Only variable-rate borrowing (stable rate removed in Aave V3.2)
- Supply tokens received: hTokens (yield-bearing ERC-20)
- Minimum health factor to borrow: > 1.0 (recommend maintaining > 1.5)

**Write ops — after user confirmation, submits via `onchainos wallet contract-call`**

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command:

1. **Check onchainos**: `which onchainos` — if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet
2. **Check binary**: `which hyperlend-pooled` — if not found, install via `plugin-store install hyperlend-pooled`
3. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true`; if not, run `onchainos wallet login`
4. **For write operations**: verify HyperEVM (chain 999) wallet has sufficient HYPE for gas

## Commands

### `hyperlend-pooled get-markets` — View All Markets

**Triggers:** "HyperLend markets", "HyperLend rates", "HyperLend APY", "lending rates HyperEVM",
"HyperLend supply rate", "HyperLend borrow rate", "what can I supply on HyperLend"

```bash
hyperlend-pooled get-markets
hyperlend-pooled get-markets --active-only
```

**Parameters:**
- `--active-only` — only show active, non-frozen markets

**Flow:** Calls `https://api.hyperlend.finance/data/markets?chain=hyperEvm`. No wallet needed.

---

### `hyperlend-pooled positions` — Check Your Position

**Triggers:** "my HyperLend position", "HyperLend health factor", "what have I supplied on HyperLend",
"HyperLend debt", "my HyperLend balance", "check HyperLend collateral"

```bash
hyperlend-pooled positions
hyperlend-pooled positions --from 0xYourWallet
```

**Parameters:**
- `--from <address>` — wallet to query (defaults to logged-in wallet)

**Flow:** Calls Pool.getUserAccountData + ProtocolDataProvider.getUserReserveData for each asset.

---

### `hyperlend-pooled supply` — Supply Asset

**Triggers:** "supply to HyperLend", "deposit USDC HyperLend", "add collateral HyperLend",
"earn yield HyperLend", "supply wHYPE HyperLend"

```bash
hyperlend-pooled --dry-run supply --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
hyperlend-pooled supply --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
```

**Parameters:**
- `--asset <address>` — ERC-20 token address to supply
- `--amount <raw_units>` — amount in raw token units (e.g. `1000000` = 1 USDC at 6 decimals)
- `--from <address>` — sender wallet (optional, defaults to logged-in wallet)

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
1. ERC-20 approve Pool for `amount`
2. Wait 3 seconds (nonce safety)
3. Pool.supply(asset, amount, onBehalfOf, referralCode=0)

User receives hTokens (e.g. hUSDC) representing their yield-bearing position.

---

### `hyperlend-pooled borrow` — Borrow Asset

**Triggers:** "borrow from HyperLend", "borrow USDC HyperLend", "take loan HyperLend",
"borrow against wHYPE", "get USDC from HyperLend"

```bash
hyperlend-pooled --dry-run borrow --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 200000000
hyperlend-pooled borrow --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 200000000
```

**Parameters:**
- `--asset <address>` — ERC-20 token address to borrow
- `--amount <raw_units>` — amount in raw token units
- `--from <address>` — borrower wallet (optional)

**Constraints:**
- Must have sufficient collateral supplied first
- Health factor after borrow must remain > 1.0 (recommend > 1.5)
- Only variable-rate borrowing (interestRateMode=2)

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
No ERC-20 approval needed. Calls Pool.borrow(asset, amount, 2, 0, onBehalfOf).
VariableDebtTokens minted automatically.

**Risk warning:** Always inform user of liquidation risk if collateral price drops.
Recommend maintaining health factor > 1.5.

---

### `hyperlend-pooled repay` — Repay Debt

**Triggers:** "repay HyperLend", "pay back HyperLend loan", "repay USDC HyperLend",
"clear HyperLend debt", "repay all HyperLend"

```bash
hyperlend-pooled --dry-run repay --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 200000000
hyperlend-pooled repay --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 200000000
```

**Parameters:**
- `--asset <address>` — borrowed ERC-20 token address
- `--amount <raw_units>` — amount to repay in raw token units. For repay-all, use your actual
  token wallet balance (do NOT use u128::MAX — interest accrues per-second and the pool
  may revert if pulled amount exceeds wallet balance)
- `--from <address>` — repayer wallet (optional)

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
1. ERC-20 approve Pool for `amount`
2. Wait 3 seconds
3. Pool.repay(asset, amount, 2, onBehalfOf)

---

### `hyperlend-pooled withdraw` — Withdraw Supplied Asset

**Triggers:** "withdraw from HyperLend", "remove collateral HyperLend", "get back USDC HyperLend",
"withdraw all HyperLend", "exit HyperLend position"

```bash
hyperlend-pooled --dry-run withdraw --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
hyperlend-pooled withdraw --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
# Withdraw all (amount=0 uses uint256.max):
hyperlend-pooled withdraw --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 0
```

**Parameters:**
- `--asset <address>` — ERC-20 token address to withdraw
- `--amount <raw_units>` — amount to withdraw. Pass `0` to withdraw all (uses uint256.max internally)
- `--from <address>` — recipient wallet (optional)

**Constraints:**
- Withdraw-all (amount=0) reverts if user has any outstanding debt. Clear all debt first.
- Health factor must remain > 1.0 after withdrawal.

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
No ERC-20 approval needed. Calls Pool.withdraw(asset, amount, to). hTokens burned automatically.

## Token Addresses (HyperEVM Mainnet)

| Token | Address | Decimals |
|-------|---------|----------|
| USDC  | `0xb88339CB7199b77E23DB6E890353E22632Ba630f` | 6 |
| wHYPE | `0x5555555555555555555555555555555555555555` | 18 |
| USD0  | `0xB8CE59FC3717ada4C02eaDF9682A9e934F625ebb` | 6 |

Use `get-markets` to retrieve all current market addresses and their decimals.

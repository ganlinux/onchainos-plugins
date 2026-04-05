---
name: lista-cdp
description: >-
  Use when the user asks about Lista DAO, Lista CDP, slisBNB, lisUSD, staking BNB on Lista,
  'stake BNB Lista', 'get slisBNB', 'deposit slisBNB', 'borrow lisUSD', 'repay lisUSD',
  'withdraw slisBNB collateral', 'Lista CDP position', 'Lista stablecoin', 'lisUSD borrow',
  'slisBNB collateral', 'Lista liquid staking', 'Lista DAO BSC',
  or mentions Lista, Lista DAO, Lista CDP, slisBNB liquid staking, lisUSD stablecoin.
  Covers: BNB staking to slisBNB, CDP collateral deposit, lisUSD borrowing, repay,
  collateral withdrawal, and position queries on BSC.
  Do NOT use for general BNB staking unrelated to Lista. Do NOT use for other BSC CDP protocols.
license: MIT
metadata:
  author: ganlinux
  version: "0.1.0"
---

# Lista CDP Plugin

Lista DAO is a BSC-native CDP (Collateralized Debt Position) protocol. Users stake BNB to receive slisBNB
(Liquid Staking Token with BNB staking yield), then deposit slisBNB as collateral to borrow lisUSD stablecoin.

**Key parameters:**
- Max LTV: 80% (minimum collateral ratio: 125%)
- Borrow APR: ~4.35% annualized
- Min BNB stake: 0.001 BNB
- Min borrow: ~15 lisUSD
- slisBNB exchange rate: ~1 BNB = 0.9659 slisBNB (increases over time with staking rewards)

**Write ops — after user confirmation, submits via `onchainos wallet contract-call`**

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command. Always follow in order.

1. **Check onchainos**: `which onchainos` — if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet
2. **Check binary**: `which lista-cdp` — if not found, install via `plugin-store install lista-cdp`
3. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true`; if not, run `onchainos wallet login`
4. **For write operations**: verify BSC (chain 56) wallet has sufficient BNB for gas

## Commands

### `lista-cdp stake` — Stake BNB for slisBNB

**Triggers:** "stake BNB", "stake BNB Lista", "get slisBNB", "BNB to slisBNB", "Lista staking"

```bash
lista-cdp stake --amt <BNB_WEI>
lista-cdp --dry-run stake --amt 100000000000000000
```

**Parameters:**
- `--amt <wei>` — BNB amount in wei (e.g. `1000000000000000000` = 1 BNB)

**Constraints:**
- Minimum: 0.001 BNB (1e15 wei)
- Wallet must have BNB balance + gas

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
Calls `StakeManager.deposit()` payable with `--amt` as ETH value.

**Output:** slisBNB minted to wallet, BSCScan link.

---

### `lista-cdp unstake` — Request slisBNB Withdrawal

**Triggers:** "unstake slisBNB", "redeem slisBNB", "slisBNB to BNB", "Lista unstake"

```bash
lista-cdp unstake --amount <SLISBNB_AMOUNT>
lista-cdp --dry-run unstake --amount 0.5
```

**Parameters:**
- `--amount <float>` — slisBNB amount in human-readable units (e.g. `0.5`)

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
Calls `StakeManager.requestWithdraw(uint256)`. After unbonding period, user must call `claimWithdraw`.

**Output:** requestWithdraw transaction hash, BSCScan link.

---

### `lista-cdp cdp-deposit` — Deposit slisBNB Collateral

**Triggers:** "deposit slisBNB", "add collateral Lista", "deposit collateral Lista CDP"

```bash
lista-cdp cdp-deposit --amount <SLISBNB_AMOUNT>
lista-cdp --dry-run cdp-deposit --amount 0.5
```

**Parameters:**
- `--amount <float>` — slisBNB amount to deposit (e.g. `0.5`)

**Flow:**
1. Run `--dry-run` to preview both transactions
2. **Ask user to confirm** Step 1 (approve slisBNB) before broadcasting
3. Execute: `onchainos wallet contract-call` → slisBNB.approve(Interaction, amount)
4. Wait 3 seconds to avoid nonce conflict
5. **Ask user to confirm** Step 2 (CDP deposit) before broadcasting
6. Execute: `onchainos wallet contract-call` → Interaction.deposit(wallet, slisBNB, amount)

**Output:** Approve and deposit transaction hashes, BSCScan links.

---

### `lista-cdp borrow` — Borrow lisUSD

**Triggers:** "borrow lisUSD", "mint lisUSD", "get lisUSD Lista", "borrow stablecoin Lista"

```bash
lista-cdp borrow --amount <LISUSD_AMOUNT>
lista-cdp --dry-run borrow --amount 100
```

**Parameters:**
- `--amount <float>` — lisUSD amount to borrow (e.g. `100`)

**Constraints:**
- Minimum: 15 lisUSD
- Must have sufficient collateral deposited
- Post-borrow collateral ratio must be >= 125%

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
Calls `Interaction.borrow(slisBNB_addr, hayAmount)`.

**Output:** Borrow transaction hash, BSCScan link, remaining available borrow capacity.

---

### `lista-cdp repay` — Repay lisUSD Debt

**Triggers:** "repay lisUSD", "payback lisUSD", "close Lista CDP", "repay debt Lista"

```bash
lista-cdp repay --amount <LISUSD_AMOUNT>
lista-cdp --dry-run repay --amount 100
```

**Parameters:**
- `--amount <float>` — lisUSD amount to repay (e.g. `100`)

**Tip:** For full repayment, first run `lista-cdp positions` to get exact debt amount (avoid using max uint256).

**Flow:**
1. Run `--dry-run` to preview both transactions
2. **Ask user to confirm** Step 1 (approve lisUSD) before broadcasting
3. Execute: `onchainos wallet contract-call` → lisUSD.approve(Interaction, amount)
4. Wait 3 seconds to avoid nonce conflict
5. **Ask user to confirm** Step 2 (payback) before broadcasting
6. Execute: `onchainos wallet contract-call` → Interaction.payback(slisBNB_addr, amount)

**Output:** Approve and payback transaction hashes, remaining debt.

---

### `lista-cdp cdp-withdraw` — Withdraw slisBNB Collateral

**Triggers:** "withdraw collateral Lista", "get back slisBNB", "close CDP Lista"

```bash
lista-cdp cdp-withdraw --amount <SLISBNB_AMOUNT>
lista-cdp --dry-run cdp-withdraw --amount 0.5
```

**Parameters:**
- `--amount <float>` — slisBNB amount to withdraw (e.g. `0.5`)

**Constraints:**
- Amount must not exceed locked collateral
- If debt outstanding: post-withdrawal collateral ratio must remain >= 125%
- For full withdrawal: repay all debt first

**Flow:** Run `--dry-run` to preview, then **ask user to confirm** before proceeding.
Calls `Interaction.withdraw(wallet, slisBNB_addr, amount)`.

**Output:** Withdraw transaction hash, BSCScan link.

---

### `lista-cdp positions` — View CDP Position

**Triggers:** "Lista CDP position", "slisBNB collateral", "lisUSD debt", "Lista health factor", "view Lista position"

```bash
lista-cdp positions
lista-cdp positions --wallet 0xYourAddress
```

**Parameters:**
- `--wallet <address>` — (optional) query a specific address instead of logged-in wallet

**Output:**
- Locked slisBNB collateral amount
- Outstanding lisUSD debt
- Available borrow capacity (remaining)
- Liquidation trigger price (USD per slisBNB)
- Current borrow APR
- Wallet slisBNB and lisUSD balances

---

## Common Workflows

### Workflow 1: Stake BNB and Open CDP

```bash
# Step 1: Stake BNB to get slisBNB
lista-cdp stake --amt 1000000000000000000    # 1 BNB

# Step 2: Deposit slisBNB as collateral
lista-cdp cdp-deposit --amount 0.9           # ~0.9659 slisBNB from 1 BNB

# Step 3: Check available borrow capacity
lista-cdp positions

# Step 4: Borrow lisUSD (stay below 80% LTV for safety)
lista-cdp borrow --amount 200
```

### Workflow 2: Repay and Withdraw

```bash
# Step 1: Check exact debt
lista-cdp positions

# Step 2: Repay all debt (use exact amount from positions output)
lista-cdp repay --amount 200.123456

# Step 3: Withdraw all collateral
lista-cdp cdp-withdraw --amount 0.9
```

## Contract Addresses (BSC Mainnet)

| Contract | Address |
|----------|---------|
| StakeManager | `0x1adB950d8bB3dA4bE104211D5AB038628e477fE6` |
| Interaction (CDP) | `0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4` |
| slisBNB token | `0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B` |
| lisUSD token | `0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5` |

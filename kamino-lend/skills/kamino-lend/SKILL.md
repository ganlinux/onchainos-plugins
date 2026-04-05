---
name: kamino-lend
description: >
  Kamino Lend plugin for Solana. Enables supplying (depositing), borrowing, withdrawing,
  and repaying assets on Kamino Finance lending markets on Solana (chain ID 501).
  Trigger phrases: "deposit to kamino", "borrow from kamino", "repay kamino loan",
  "withdraw from kamino", "check kamino position", "view kamino market",
  "存入 kamino", "从 kamino 借款", "还款 kamino", "从 kamino 取款", "查看 kamino 仓位"
---

# Kamino Lend Skill

## Overview

This skill provides an AI agent with full access to Kamino Finance lending markets on Solana.
It supports both read-only market data queries and on-chain write operations (deposit, borrow,
withdraw, repay) via the `kamino-lend` binary, which integrates with `onchainos` for Solana
transaction signing and broadcasting.

**Architecture**: The agent fetches unsigned Solana transactions from the Kamino REST API, then
after user confirmation, submits via `onchainos wallet contract-call` with `--unsigned-tx`.
Solana blockhashes expire in ~60 seconds, so the agent must submit transactions immediately
after building them.

## Pre-flight Checks

Before executing any command, verify:

```bash
# 1. Check onchainos is installed (version >= 2.0.0)
onchainos --version

# 2. Check kamino-lend binary is available
kamino-lend --version

# 3. For on-chain operations: check wallet is logged in
onchainos wallet balance --chain 501 --output json
```

If `onchainos` is not installed, direct the user to install the onchainOS Agentic Wallet.
If `kamino-lend` is not installed, it must be built from source or obtained from the plugin store.
For on-chain operations, the user must be logged in to their Solana wallet via onchainos.

## Commands

### 1. markets — List Kamino Markets (off-chain)

**Trigger**: "show kamino markets", "list kamino lending markets", "查看 kamino 市场"

```bash
kamino-lend markets
```

**Output**: List of all Kamino lending markets with pubkeys and names.
**No wallet required.**

---

### 2. reserves — Reserve Metrics (off-chain)

**Trigger**: "kamino reserve rates", "kamino APY", "kamino supply rates", "查看 kamino 利率"

```bash
# Main market (default)
kamino-lend reserves

# Specific market
kamino-lend reserves --market <MARKET_PUBKEY>
```

**Output**: Reserve list with symbol, supply APY, borrow APY.
**No wallet required.**

Common reserves in the main market (`7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF`):
- USDC: `D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59`
- SOL: `d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q`
- JLP: `DdTmCCjv7zHRD1hJv3E8bpnSEQBzdKkzB1j9ApXX5QoP`

---

### 3. obligations — User Positions (off-chain)

**Trigger**: "check kamino position", "my kamino loans", "kamino health factor",
"查看 kamino 仓位", "我的 kamino 借款"

```bash
# Current logged-in wallet
kamino-lend obligations

# Specific wallet
kamino-lend obligations --wallet <WALLET_PUBKEY>

# Specific market
kamino-lend obligations --market <MARKET_PUBKEY>
```

**Output**: All obligations with supplied/borrowed amounts and health factor.
Health factor warnings are displayed:
- Below 1.5: WARNING
- Below 1.1: DANGER (risk of liquidation)

---

### 4. deposit — Supply Tokens (on-chain, REQUIRES CONFIRMATION)

**Trigger**: "deposit to kamino", "supply to kamino", "存入 kamino", "向 kamino 充值"

**IMPORTANT**: Before executing this command, the agent MUST ask user to confirm the
deposit details (token, amount, market) and wait for explicit user confirmation.

```bash
# Deposit 100 USDC to the main market
kamino-lend deposit \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 100

# With explicit wallet and market
kamino-lend deposit \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 100 \
  --wallet <WALLET_PUBKEY> \
  --market 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF

# Dry-run (build tx without submitting)
kamino-lend --dry-run deposit --reserve <RESERVE> --amount <AMOUNT>
```

**Output**: Transaction hash on success.

**Confirmation prompt example**:
> You are about to deposit 100 USDC into Kamino Lend (main market).
> Reserve: D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59
> Please confirm (yes/no):

---

### 5. withdraw — Withdraw Tokens (on-chain, REQUIRES CONFIRMATION)

**Trigger**: "withdraw from kamino", "取出 kamino 资产", "从 kamino 提款"

**IMPORTANT**: Before executing this command, the agent MUST ask user to confirm the
withdrawal details and wait for explicit user confirmation.

```bash
kamino-lend withdraw \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 50

# Dry-run
kamino-lend --dry-run withdraw --reserve <RESERVE> --amount <AMOUNT>
```

**Output**: Transaction hash on success.

**Confirmation prompt example**:
> You are about to withdraw 50 USDC from Kamino Lend (main market).
> Please confirm (yes/no):

---

### 6. borrow — Borrow Tokens (on-chain, REQUIRES CONFIRMATION)

**Trigger**: "borrow from kamino", "从 kamino 借款", "kamino 借贷"

**IMPORTANT**: Before executing this command, the agent MUST ask user to confirm the
borrow amount and token, and warn about liquidation risk. Wait for explicit user confirmation.
The health factor is automatically checked before borrowing — if below 1.1, the borrow is rejected.

```bash
kamino-lend borrow \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 50

# Dry-run
kamino-lend --dry-run borrow --reserve <RESERVE> --amount <AMOUNT>
```

**Output**: Transaction hash on success, or error if health factor is too low.

**Confirmation prompt example**:
> You are about to borrow 50 USDC from Kamino Lend.
> WARNING: Borrowing increases liquidation risk. Current health factor will be checked.
> Please confirm (yes/no):

---

### 7. repay — Repay Loan (on-chain, REQUIRES CONFIRMATION)

**Trigger**: "repay kamino loan", "还款 kamino", "偿还 kamino 借款"

**IMPORTANT**: Before executing this command, the agent MUST ask user to confirm the
repayment details and wait for explicit user confirmation.

```bash
kamino-lend repay \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 50

# Dry-run
kamino-lend --dry-run repay --reserve <RESERVE> --amount <AMOUNT>
```

**Output**: Transaction hash on success.

**Confirmation prompt example**:
> You are about to repay 50 USDC to Kamino Lend.
> Please confirm (yes/no):

---

## Typical Workflows

### Check rates and deposit
```bash
# 1. View available markets and rates
kamino-lend markets
kamino-lend reserves

# 2. After user confirms deposit details, deposit USDC
kamino-lend deposit --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 100
```

### Borrow against collateral
```bash
# 1. Check current positions and health factor
kamino-lend obligations

# 2. After user confirms borrow details, borrow
kamino-lend borrow --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 50
```

### Repay loan
```bash
# 1. Check outstanding loans
kamino-lend obligations

# 2. After user confirms repayment, repay
kamino-lend repay --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 50
```

## Error Handling

| Error | Likely Cause | Resolution |
|-------|-------------|------------|
| `onchainos: command not found` | onchainos not installed | Install onchainOS Agentic Wallet |
| `Could not find address in onchainos output` | Wallet not logged in | Run `onchainos wallet login` |
| `Health factor ... is below minimum (1.1)` | Undercollateralized | Repay loans or add collateral before borrowing |
| `API error 4xx` | Invalid reserve/market pubkey | Verify reserve address from `kamino-lend reserves` |
| `Failed to parse onchainos response` | onchainos version mismatch | Update onchainos to >= 2.0.0 |
| `onchainos contract-call failed` | Transaction rejected or expired | Retry; Solana blockhash expires in ~60s |

## Skill Routing

Use this skill when the user mentions:
- **Kamino**, **Kamino Finance**, **Kamino Lend**, **klend**
- Actions: **deposit**, **supply**, **borrow**, **withdraw**, **repay** on Solana
- Checking lending **positions**, **health factor**, **APY** on Kamino
- Chinese: **存入 kamino**, **从 kamino 借款**, **还款 kamino**, **查看 kamino 仓位**

Do NOT use this skill for:
- Kamino Liquidity (concentrated liquidity) — that is a different product
- Non-Solana lending protocols
- Generic Solana transfers (use onchainos wallet directly)

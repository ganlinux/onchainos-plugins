---
name: gtbtc
description: >-
  Use when the user asks about GTBTC, Gate Wrapped BTC, Gate.io BTC token,
  'GTBTC balance', 'GTBTC price', 'GTBTC APR', 'GTBTC transfer', 'GTBTC approve',
  'gate wrapped btc balance', 'gate btc staking yield', 'how much is GTBTC worth',
  'GTBTC on Ethereum', 'GTBTC on BSC', 'GTBTC on Base', 'GTBTC on Solana',
  'transfer GTBTC', 'approve GTBTC', 'GTBTC DEX',
  'GTBTC 余额', 'GTBTC 价格', 'GTBTC 转账', 'GTBTC 授权', 'Gate 包装比特币',
  or mentions GTBTC, Gate Wrapped BTC, Gate BTC yield token on any supported chain.
  Covers: balance, price, apr, transfer, approve.
  Do NOT use for general BTC swaps -- use a DEX skill instead.
  Do NOT use for SolvBTC or other wrapped BTC tokens -- use their dedicated skills.
  Do NOT use for Gate.io spot trading -- this only covers on-chain GTBTC operations.
license: MIT
metadata:
  author: ganlinux
  version: "0.1.0"
---

# GTBTC Plugin

Gate Wrapped BTC (GTBTC) is a yield-bearing BTC token issued by Gate Web3. Users stake BTC on Gate to receive GTBTC 1:1; yield accrues automatically via NAV growth (similar to rETH). GTBTC is a standard ERC-20 on Ethereum/BSC/Base and SPL token on Solana.

**Key facts:**
- Contract (Ethereum/BSC/Base): `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3`
- Solana mint: `gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So`
- Decimals: **8** (BTC precision, not 18)
- Mint/redeem: Gate platform only (https://www.gate.com/staking/BTC)

## Architecture

- Read ops (balance, price, apr) -> direct `eth_call` via public RPC or Gate API; no confirmation needed
- Write ops (transfer, approve) -> after user confirmation, submits via `onchainos wallet contract-call`

## Pre-flight Checks

Run immediately when this skill is triggered:

1. **Check onchainos**: `which onchainos` -- if not found, direct user to install; if found, run `onchainos --version` (>= 2.0.0)
2. **Check binary**: `which gtbtc` -- if not found, install via `plugin-store install gtbtc`
3. **Check wallet login** (write ops only): `onchainos wallet status` -- must show `loggedIn: true`

## Execution Flow for Write Operations

1. Run with `--dry-run` first to preview calldata
2. **Ask user to confirm** the transaction details before executing on-chain
3. Execute only after explicit user approval
4. Report transaction hash and outcome

## Commands

### `gtbtc balance` -- Query GTBTC Balance

**Triggers:** "GTBTC balance", "how much GTBTC do I have", "GTBTC 余额", "check my GTBTC"

```bash
# Current wallet on Ethereum (default)
gtbtc balance

# Specific chain
gtbtc --chain 56 balance           # BSC
gtbtc --chain 8453 balance         # Base
gtbtc --chain 501 balance          # Solana

# Specific address
gtbtc balance --address 0xYourAddress
gtbtc --chain 501 balance --address YourSolanaAddress
```

Output: address, chain, GTBTC balance (human-readable and atomic units).

---

### `gtbtc price` -- Get GTBTC Price

**Triggers:** "GTBTC price", "how much is GTBTC", "GTBTC USD value", "GTBTC 价格"

```bash
gtbtc price
```

Output: current USD price, 24h change, 24h high/low, volume. Source: Gate.io spot market.

---

### `gtbtc apr` -- Get GTBTC Staking APR

**Triggers:** "GTBTC APR", "GTBTC yield", "GTBTC staking rate", "GTBTC 年化", "GTBTC 收益率"

```bash
gtbtc apr
```

Output: current APR range from Gate Flex Earn. Note: GTBTC native BTC staking yield accrues automatically via NAV growth even without Flex Earn.

---

### `gtbtc transfer` -- Transfer GTBTC (EVM)

**Triggers:** "transfer GTBTC", "send GTBTC to", "GTBTC 转账", "move GTBTC"

```bash
# Preview (dry run)
gtbtc --dry-run transfer --to 0xRecipient --amount 0.001

# Execute (ask user to confirm first)
gtbtc --chain 1 transfer --to 0xRecipient --amount 0.001
gtbtc --chain 56 transfer --to 0xRecipient --amount 0.001   # BSC
gtbtc --chain 8453 transfer --to 0xRecipient --amount 0.001 # Base
```

**Parameters:**
- `--to <address>` -- Recipient EVM address (0x...)
- `--amount <GTBTC>` -- Amount in GTBTC (e.g. `0.001` = 100,000 atomic units, decimals=8)
- `--from <address>` -- (optional) Override sender address
- `--chain <id>` -- Chain ID: 1 (Ethereum), 56 (BSC), 8453 (Base)

**Constraints:**
- EVM only in v1; Solana SPL transfer not supported
- Decimals=8: 0.001 GTBTC = 100,000 atomic units (NOT 10^15)

**Before executing:** Run `--dry-run` first and **ask user to confirm** the recipient and amount.

---

### `gtbtc approve` -- Approve GTBTC Spender (EVM)

**Triggers:** "approve GTBTC for DEX", "GTBTC approval", "allow DEX to use GTBTC", "GTBTC 授权"

```bash
# Preview (dry run)
gtbtc --dry-run approve --spender 0xDEXRouter --amount 0.1

# Unlimited approval
gtbtc --dry-run approve --spender 0xDEXRouter

# Execute (ask user to confirm first)
gtbtc --chain 1 approve --spender 0xE592427A0AEce92De3Edee1F18E0157C05861564 --amount 0.1
```

**Parameters:**
- `--spender <address>` -- DEX router or protocol address
- `--amount <GTBTC>` -- Amount to approve (omit for unlimited `uint256::MAX`)
- `--from <address>` -- (optional) Override owner address
- `--chain <id>` -- Chain ID: 1, 56, or 8453

**Before executing:** Run `--dry-run` first and **ask user to confirm** the spender address and approval amount.

---

## Notes

- **Mint/Redeem**: GTBTC can only be minted/redeemed via Gate platform (https://www.gate.com/staking/BTC), not via on-chain contracts.
- **Decimals=8**: All amounts use BTC precision (8 decimals). 1 GTBTC = 100,000,000 atomic units.
- **Same address**: The same contract address `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` is used on Ethereum, BSC, and Base -- but always specify the correct `--chain`.

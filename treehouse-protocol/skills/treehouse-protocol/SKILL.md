---
name: treehouse-protocol
description: >-
  Use when the user asks about Treehouse Protocol, tETH, tAVAX,
  'deposit ETH Treehouse', 'stake ETH tETH', 'deposit wstETH Treehouse',
  'deposit stETH Treehouse', 'deposit WETH Treehouse',
  'stake AVAX Treehouse', 'deposit AVAX tAVAX', 'deposit sAVAX Treehouse',
  'tETH balance', 'tAVAX balance', 'tETH price', 'tAVAX price',
  'tETH APY', 'tAVAX APY', 'Treehouse yield', 'Treehouse position',
  'redeem tETH', 'withdraw tETH', 'tETH to wstETH Curve',
  'Treehouse fixed income', 'MEY token', 'tETH wstETH exchange rate',
  or mentions Treehouse Protocol, tETH, tAVAX on Ethereum or Avalanche.
  Covers: deposit (ETH/WETH/stETH/wstETH on Ethereum; AVAX/sAVAX on Avalanche),
  balance query, price query, positions overview, and tETH withdrawal via Curve.
  Do NOT use for Lido, Rocketpool, or other liquid staking protocols.
  Do NOT use for mETH Protocol or cmETH (that is Mantle staking, not Treehouse).
  Do NOT use for Aave, Compound, or other lending protocols.
license: MIT
metadata:
  author: skylavis-sky
  version: "0.1.0"
---

# Treehouse Protocol Plugin

Treehouse Protocol is a fixed-income yield aggregation protocol. Users deposit ETH or LST tokens (WETH, stETH, wstETH) on Ethereum to receive **tETH** — a yield-bearing token that accrues staking APY plus MEY (Maximal Extractable Yield). On Avalanche, users deposit AVAX or sAVAX to receive **tAVAX**.

Both tETH and tAVAX are ERC-4626 compatible vault tokens; their value grows over time relative to the underlying asset.

**Key facts:**
- tETH Router accepts: ETH (native), WETH, stETH, wstETH
- tAVAX Router accepts: AVAX (native), sAVAX (Benqi)
- tETH fast redemption: via Curve StableSwap pool (tETH/wstETH), limited to ~200 wstETH
- tAVAX redemption: 7-day waiting period (NOT supported in this plugin)

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command:

1. **Check onchainos**: `which onchainos` — if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet; if found, run `onchainos --version` and verify version is **>= 2.0.0**
2. **Check binary**: `which treehouse-protocol` — if not found, install via `plugin-store install treehouse-protocol`
3. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true`; if not, run `onchainos wallet login`
4. **For write operations**: verify sufficient token/ETH/AVAX balance before proceeding

## Architecture

- Write ops (deposit, withdraw) → after user confirmation, submits via `onchainos wallet contract-call`
- Read ops (balance, price, positions) → direct `eth_call` via public RPC; no confirmation needed
- APY data → DeFiLlama Yields API (https://yields.llama.fi/pools)

## Execution Flow for Write Operations

1. Run with `--dry-run` first to preview calldata
2. **Ask user to confirm** before executing on-chain
3. Execute only after explicit user approval
4. Report transaction hash and outcome

## Supported Chains

| Chain | Chain ID | Token | Accepted Deposits |
|-------|----------|-------|-------------------|
| Ethereum | 1 | tETH | ETH (native), WETH, stETH, wstETH |
| Avalanche | 43114 | tAVAX | AVAX (native), sAVAX |

## Commands

### `treehouse-protocol deposit` — Deposit to get yield token

**Triggers:** "deposit ETH to Treehouse", "stake ETH get tETH", "deposit wstETH Treehouse",
"deposit AVAX get tAVAX", "stake AVAX Treehouse", "deposit sAVAX"

```bash
# Deposit native ETH → tETH (Ethereum)
treehouse-protocol deposit --chain 1 --token ETH --amount 1.0

# Deposit ERC-20 token → tETH (requires approve + deposit, handled automatically)
treehouse-protocol deposit --chain 1 --token wstETH --amount 0.5
treehouse-protocol deposit --chain 1 --token stETH --amount 1.0
treehouse-protocol deposit --chain 1 --token WETH --amount 1.0

# Deposit native AVAX → tAVAX (Avalanche)
treehouse-protocol deposit --chain 43114 --token AVAX --amount 10.0

# Deposit sAVAX → tAVAX
treehouse-protocol deposit --chain 43114 --token sAVAX --amount 5.0
```

**Parameters:**
- `--chain <ID>` — Chain ID: `1` (Ethereum, default) or `43114` (Avalanche)
- `--token <SYMBOL>` — Token: `ETH`, `WETH`, `stETH`, `wstETH` (Ethereum); `AVAX`, `sAVAX` (Avalanche)
- `--amount <AMOUNT>` — Amount in human-readable units (e.g. "1.0")
- `--from <ADDRESS>` — Optional sender address; resolved from logged-in wallet if omitted

**Important:** Always run with `--dry-run` first, then **ask user to confirm** before the real transaction.

```bash
# Preview first (dry run)
treehouse-protocol deposit --chain 1 --token ETH --amount 1.0 --dry-run

# Execute after user confirms
treehouse-protocol deposit --chain 1 --token ETH --amount 1.0
```

**ERC-20 deposit flow (WETH/stETH/wstETH/sAVAX):**
The plugin automatically sends two transactions:
1. `approve(Router, amount)` — authorizes the Router to spend your token
2. `deposit(tokenAddress, amount)` — deposits the token and mints tETH/tAVAX

**Contracts:**
- tETH Router (Ethereum): `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5`
- tAVAX Router (Avalanche): `0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e`

---

### `treehouse-protocol balance` — Query tETH or tAVAX balance

**Triggers:** "my tETH balance", "how much tETH do I have", "tAVAX balance",
"Treehouse balance", "check tETH", "check tAVAX"

```bash
# Query tETH balance (Ethereum)
treehouse-protocol balance --chain 1

# Query tAVAX balance (Avalanche)
treehouse-protocol balance --chain 43114

# Query for a specific address
treehouse-protocol balance --chain 1 --account 0xYourAddress
```

**Parameters:**
- `--chain <ID>` — `1` (Ethereum, default) or `43114` (Avalanche)
- `--account <ADDRESS>` — Optional address; resolved from logged-in wallet if omitted

**Output:** tETH/tAVAX balance and equivalent underlying value (wstETH for tETH; sAVAX for tAVAX).

---

### `treehouse-protocol price` — Get tETH or tAVAX price

**Triggers:** "tETH price", "tAVAX price", "tETH exchange rate", "tETH to wstETH rate",
"tAVAX to sAVAX rate", "Treehouse exchange rate"

```bash
# Get tETH price vs wstETH (Ethereum)
treehouse-protocol price --chain 1

# Get tAVAX price vs sAVAX (Avalanche)
treehouse-protocol price --chain 43114
```

**Parameters:**
- `--chain <ID>` — `1` (Ethereum, default) or `43114` (Avalanche)

**Output:** How many underlying tokens (wstETH/sAVAX) 1 tETH/tAVAX is worth, calculated via ERC-4626 `convertToAssets`.

---

### `treehouse-protocol positions` — Full position overview

**Triggers:** "my Treehouse position", "tETH holdings", "tAVAX holdings",
"Treehouse APY", "tETH APY", "tAVAX APY", "Treehouse yield",
"show my Treehouse", "Treehouse portfolio"

```bash
# Full tETH position on Ethereum
treehouse-protocol positions --chain 1

# Full tAVAX position on Avalanche
treehouse-protocol positions --chain 43114

# For a specific address
treehouse-protocol positions --chain 1 --account 0xYourAddress
```

**Parameters:**
- `--chain <ID>` — `1` (Ethereum, default) or `43114` (Avalanche)
- `--account <ADDRESS>` — Optional address; resolved from logged-in wallet if omitted

**Output:** Balance, underlying value, price per share, APY (from DeFiLlama), and TVL.

---

### `treehouse-protocol withdraw` — Redeem tETH → wstETH (Ethereum only)

**Triggers:** "redeem tETH", "withdraw tETH", "swap tETH to wstETH",
"exit Treehouse", "sell tETH", "get back wstETH from Treehouse"

```bash
# Dry-run first to preview
treehouse-protocol withdraw --chain 1 --amount 10.0 --dry-run

# Execute after user confirms (ask user to confirm first!)
treehouse-protocol withdraw --chain 1 --amount 10.0

# Custom slippage tolerance (default 1% = 100 bps)
treehouse-protocol withdraw --chain 1 --amount 10.0 --slippage-bps 50
```

**Parameters:**
- `--chain <ID>` — Must be `1` (Ethereum); tAVAX withdrawal is not supported
- `--amount <AMOUNT>` — Amount of tETH to redeem (e.g. "10.0")
- `--slippage-bps <BPS>` — Slippage tolerance in basis points (default: `100` = 1%)
- `--from <ADDRESS>` — Optional sender address

**Important:** Always run with `--dry-run` first, then **ask user to confirm** before the real transaction.

**Limitations:**
- Only suitable for **<= 200 wstETH** equivalent (Curve Redemption Band)
- For larger amounts, the standard 7-day redemption flow is required (not supported in this plugin)
- tAVAX redemption is NOT supported (requires 7-day waiting period)

**Flow:**
1. Queries Curve pool `get_dy(0, 1, amount)` to estimate wstETH output
2. Approves tETH to Curve pool
3. Calls `exchange(0, 1, amount, min_dy)` with slippage protection

**Contracts:**
- tETH Token: `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8`
- Curve tETH/wstETH Pool: `0xA10d15538E09479186b4D3278BA5c979110dDdB1`

---

## Contract Addresses Reference

### Ethereum (Chain ID: 1)

| Contract | Address |
|----------|---------|
| tETH Token | `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8` |
| tETH Router | `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5` |
| Curve tETH/wstETH Pool | `0xA10d15538E09479186b4D3278BA5c979110dDdB1` |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` |
| stETH (Lido) | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` |
| wstETH (Lido) | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` |

### Avalanche C-Chain (Chain ID: 43114)

| Contract | Address |
|----------|---------|
| tAVAX Token | `0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646` |
| tAVAX Router | `0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e` |
| sAVAX (Benqi) | `0x2b2C81e08f1Af8835a78Bb2A90AE924ACE0eA4bE` |

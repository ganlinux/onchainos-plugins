---
name: reservoir-protocol
description: >-
  Use when the user asks about Reservoir Protocol, rUSD stablecoin, srUSD yield,
  'mint rUSD', 'deposit USDC Reservoir', 'get rUSD', 'rUSD from USDC',
  'srUSD yield', 'save rUSD', 'earn yield rUSD', 'deposit rUSD srUSD',
  'redeem rUSD', 'rUSD to USDC', 'redeem srUSD', 'srUSD to rUSD',
  'Reservoir Protocol balance', 'rUSD balance', 'srUSD balance', 'srUSD APY',
  'srUSD exchange rate', 'Reservoir Protocol info', 'PSM liquidity',
  or mentions Reservoir Protocol, rUSD, srUSD, wsrUSD on Ethereum.
  Covers: portfolio info, mint rUSD from USDC, deposit rUSD for srUSD yield,
  redeem rUSD to USDC via PSM, redeem srUSD to rUSD via Saving Module.
  Do NOT use for general Ethereum token swaps unrelated to Reservoir Protocol.
  Do NOT use for NFT Reservoir (reservoir.tools) — this plugin is for reservoir.xyz stablecoin protocol only.
  Only supports Ethereum mainnet (chain 1) for minting/saving/redeeming.
license: MIT
metadata:
  author: skylavis-sky
  version: "0.1.0"
---

# Reservoir Protocol Plugin

Reservoir Protocol is a decentralized stablecoin protocol on Ethereum mainnet. Users deposit USDC to mint rUSD (1:1 stablecoin) via the Peg Stability Module (PSM), then optionally deposit rUSD into the Saving Module to receive srUSD — a yield-bearing stablecoin (~7-8% APY). All minting, saving, and redeeming operations are Ethereum mainnet only (chain 1).

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command:

1. **Check onchainos**: `which onchainos` — if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet; if found, run `onchainos --version` and verify version is **>= 2.0.0**
2. **Check binary**: `which reservoir-protocol` — if not found, install via `plugin-store install reservoir-protocol`
3. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true` with an Ethereum address; if not, run `onchainos wallet login`
4. **For mint/save/redeem**: verify sufficient token balance before submitting transactions

## Commands

### `reservoir-protocol info` — Portfolio Info

**Triggers:** "rUSD balance", "srUSD balance", "srUSD APY", "srUSD exchange rate", "Reservoir Protocol info", "PSM liquidity", "Reservoir portfolio"

```bash
reservoir-protocol info
reservoir-protocol info --wallet <ETH_ADDRESS>
```

**Parameters:**
- `--wallet <ADDRESS>` — (optional) Ethereum address to query; defaults to onchainos wallet

Output: rUSD balance, srUSD balance, srUSD current price (rUSD/srUSD rate), redeemable rUSD value from srUSD, PSM USDC liquidity.

---

### `reservoir-protocol mint` — Mint rUSD from USDC

**Triggers:** "mint rUSD", "deposit USDC Reservoir", "get rUSD", "rUSD from USDC", "buy rUSD with USDC"

```bash
reservoir-protocol mint --amount <USDC_AMOUNT>
reservoir-protocol --dry-run mint --amount 100
```

**Parameters:**
- `--amount <USDC>` — Amount of USDC to deposit (UI units, e.g. `100` or `100.5`)

**Constraints:**
- Ethereum mainnet only (chain 1)
- USDC is 6 decimals; mintStablecoin parameter uses 6-decimal raw units internally
- Credit Enforcer checks PSM debt ceiling and protocol solvency ratio; may revert if limits reached
- Two steps: approve USDC + mintStablecoin; ask user to confirm each step before proceeding
- Use `--dry-run` to simulate without broadcasting

**Examples:**
```bash
# Mint 1000 rUSD from 1000 USDC
reservoir-protocol mint --amount 1000

# Simulate 100 USDC mint
reservoir-protocol --dry-run mint --amount 100
```

---

### `reservoir-protocol save` — Deposit rUSD for srUSD Yield

**Triggers:** "save rUSD", "earn yield rUSD", "deposit rUSD srUSD", "rUSD to srUSD", "get srUSD", "Reservoir yield"

```bash
reservoir-protocol save --amount <RUSD_AMOUNT>
reservoir-protocol --dry-run save --amount 100
```

**Parameters:**
- `--amount <rUSD>` — Amount of rUSD to deposit (UI units, e.g. `100` or `100.0`)

**Constraints:**
- Ethereum mainnet only (chain 1)
- rUSD is 18 decimals; mintSavingcoin parameter uses 18-decimal raw units internally
- srUSD accrues yield continuously; redeem anytime with `redeem-srusd`
- Two steps: approve rUSD + mintSavingcoin; ask user to confirm each step before proceeding

**Examples:**
```bash
# Deposit 500 rUSD for srUSD yield
reservoir-protocol save --amount 500

# Simulate deposit
reservoir-protocol --dry-run save --amount 500
```

---

### `reservoir-protocol redeem-rusd` — Redeem rUSD to USDC

**Triggers:** "redeem rUSD", "rUSD to USDC", "withdraw USDC Reservoir", "PSM redeem"

```bash
reservoir-protocol redeem-rusd --amount <RUSD_AMOUNT>
reservoir-protocol --dry-run redeem-rusd --amount 100
```

**Parameters:**
- `--amount <rUSD>` — Amount of rUSD to redeem (UI units, e.g. `100`)

**Constraints:**
- Ethereum mainnet only (chain 1)
- Redeem is 1:1 rUSD -> USDC via PSM; limited by PSM USDC liquidity
- Check PSM liquidity with `reservoir-protocol info` before large redemptions
- Two steps: approve rUSD + PSM.redeem; ask user to confirm each step before proceeding

**Examples:**
```bash
# Redeem 200 rUSD for 200 USDC
reservoir-protocol redeem-rusd --amount 200

# Simulate redeem
reservoir-protocol --dry-run redeem-rusd --amount 200
```

---

### `reservoir-protocol redeem-srusd` — Redeem srUSD to rUSD

**Triggers:** "redeem srUSD", "srUSD to rUSD", "withdraw srUSD", "exit srUSD position"

```bash
reservoir-protocol redeem-srusd --amount <SRUSD_AMOUNT>
reservoir-protocol --dry-run redeem-srusd --amount 100
```

**Parameters:**
- `--amount <srUSD>` — Amount of srUSD to redeem (UI units, e.g. `100`)

**Constraints:**
- Ethereum mainnet only (chain 1)
- No ERC-20 approve needed; Saving Module burns srUSD directly from caller's wallet
- Returned rUSD = srUSD amount x currentPrice / 1e8 (minus redeemFee)
- Single step; ask user to confirm before proceeding

**Examples:**
```bash
# Redeem 50 srUSD for rUSD (includes accumulated yield)
reservoir-protocol redeem-srusd --amount 50

# Simulate redeem
reservoir-protocol --dry-run redeem-srusd --amount 50
```

---

## Key Addresses (Ethereum Mainnet)

| Name | Address |
|------|---------|
| rUSD Token | `0x09D4214C03D01F49544C0448DBE3A27f768F2b34` |
| srUSD Token | `0x738d1115B90efa71AE468F1287fc864775e23a31` |
| Credit Enforcer | `0x04716DB62C085D9e08050fcF6F7D775A03d07720` |
| PSM (USDC) | `0x4809010926aec940b550D34a46A52739f996D75D` |
| Saving Module | `0x5475611Dffb8ef4d697Ae39df9395513b6E947d7` |
| USDC | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` |

## Decimals Reference

| Token | Decimals | Example |
|-------|----------|---------|
| USDC | 6 | 100 USDC = 100,000,000 raw |
| rUSD | 18 | 100 rUSD = 100 * 10^18 raw |
| srUSD | 18 | 100 srUSD = 100 * 10^18 raw |

**CRITICAL:** `mintStablecoin(uint256)` takes USDC 6-decimal units. All other operations (mintSavingcoin, PSM.redeem, SavingModule.redeem) take 18-decimal units.

## Error Handling

| Error | Cause | Solution |
|-------|-------|---------|
| `No wallet found` | onchainos not logged in | Run `onchainos wallet login` |
| `Insufficient USDC balance` | Not enough USDC | Check balance with `onchainos wallet balance --chain 1` |
| `Insufficient rUSD balance` | Not enough rUSD | Mint rUSD first with `reservoir-protocol mint` |
| `Insufficient PSM USDC liquidity` | PSM has insufficient USDC reserves | Redeem smaller amount or wait for liquidity |
| `Credit Enforcer revert` | PSM debt ceiling or solvency check failed | Check protocol status at reservoir.xyz |

## Skill Routing

- For **general ERC-20 token swaps**: delegate to `okx-dex-swap`
- For **checking Ethereum wallet balance**: `onchainos wallet balance --chain 1`
- For **NFT Reservoir (reservoir.tools)**: this plugin is NOT for NFTs; use a different tool
- For **rUSD/srUSD on Base or Berachain**: only OFT bridge versions exist (no PSM); minting/saving not supported on those chains

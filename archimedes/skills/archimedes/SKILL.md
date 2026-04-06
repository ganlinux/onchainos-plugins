---
name: archimedes
version: "0.1.0"
binary: archimedes
description: "Deposit into Archimedes Finance V2 protected yield vaults (ERC4626) on Ethereum mainnet. Supports WETH and crvFRAX strategies via Convex and Aura."
---

# archimedes

## description

Archimedes Finance V2 is a protected yield protocol on Ethereum mainnet. It wraps Convex and Aura LP strategies inside ERC4626 vaults with an offchain auto-protection monitor. Users deposit WETH or crvFRAX, receive vault shares that appreciate over time, and can withdraw or redeem at any point.

- Read ops: direct eth_call against Ethereum mainnet (no SDK, no REST API)
- Write ops: after user confirmation, submits via `onchainos wallet contract-call`
- Non-standard ERC4626: withdraw/redeem take a 4th `minimumReceive` slippage param

## commands

### vaults

List all known Archimedes V2 vault addresses with their underlying asset and current TVL.

```
archimedes vaults [--rpc <URL>]
```

**Parameters:**
- `--rpc`: Optional custom Ethereum RPC URL (default: mevblocker)

**Examples:**
```
archimedes vaults
archimedes vaults --rpc https://rpc.mevblocker.io
```

**Output:** JSON list of vaults with name, vault address, underlying symbol, underlying address, and formatted TVL.

---

### positions

Show the wallet's share balance and underlying asset value in each Archimedes vault.

```
archimedes positions [--wallet <ADDR>] [--rpc <URL>]
```

**Parameters:**
- `--wallet`: Optional wallet address to query (defaults to logged-in wallet)
- `--rpc`: Optional custom Ethereum RPC URL

**Examples:**
```
archimedes positions
archimedes positions --wallet 0xAbCd...1234
```

**Output:** JSON list of positions with vault name, shares held, underlying value, and vault TVL.

---

### deposit

Deposit underlying assets into an Archimedes V2 vault. Executes an ERC-20 approve followed by a vault deposit.

**ask user to confirm** each on-chain transaction (approve and deposit) before submitting.

```
archimedes deposit --vault <ADDR> --amount <AMOUNT> [--from <ADDR>] [--rpc <URL>] [--dry-run]
```

**Parameters:**
- `--vault`: Vault contract address (use `archimedes vaults` to list)
- `--amount`: Amount of underlying asset to deposit (human-readable, e.g. "0.01")
- `--from`: Wallet address (receiver); defaults to logged-in wallet
- `--rpc`: Optional custom Ethereum RPC URL
- `--dry-run`: Build and print calldata without submitting

**On-chain operations (each requires user confirmation):**
1. **ERC-20 approve**: token.approve(vault, amount) — ask user to confirm before submitting
2. **Vault deposit**: vault.deposit(assets, receiver) — ask user to confirm before submitting

**Examples:**
```
archimedes deposit --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --amount 0.01
archimedes deposit --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --amount 0.001 --dry-run
```

**Output:** JSON with approve tx hash, deposit tx hash, expected shares received.

---

### withdraw

Withdraw underlying assets from a vault by specifying the asset amount. Uses the non-standard 4-param `withdraw(assets, receiver, owner, minimumReceive)`.

**ask user to confirm** the on-chain transaction before submitting.

```
archimedes withdraw --vault <ADDR> --amount <AMOUNT> [--from <ADDR>] [--slippage-bps <N>] [--rpc <URL>] [--dry-run]
```

**Parameters:**
- `--vault`: Vault contract address
- `--amount`: Amount of underlying asset to withdraw (human-readable)
- `--from`: Wallet address (receiver and owner); defaults to logged-in wallet
- `--slippage-bps`: Slippage tolerance in basis points (default: 50 = 0.5%). Use 0 to skip minimum.
- `--rpc`: Optional custom Ethereum RPC URL
- `--dry-run`: Simulate without broadcasting

**On-chain operations (requires user confirmation):**
- **Vault withdraw**: vault.withdraw(assets, receiver, owner, minimumReceive) -- ask user to confirm before submitting

**Examples:**
```
archimedes withdraw --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --amount 0.01
archimedes withdraw --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --amount 0.01 --slippage-bps 100
archimedes withdraw --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --amount 0.01 --dry-run
```

**Output:** JSON with tx hash, assets requested, minimum receive.

---

### redeem

Redeem vault shares for underlying assets. Redeems all shares by default, or a specified number. Uses the non-standard 4-param `redeem(shares, receiver, owner, minimumReceive)`.

**ask user to confirm** the on-chain transaction before submitting.

```
archimedes redeem --vault <ADDR> [--shares <AMOUNT>] [--from <ADDR>] [--slippage-bps <N>] [--rpc <URL>] [--dry-run]
```

**Parameters:**
- `--vault`: Vault contract address
- `--shares`: Number of shares to redeem (omit to redeem all)
- `--from`: Wallet address (receiver and owner); defaults to logged-in wallet
- `--slippage-bps`: Slippage tolerance in basis points (default: 50 = 0.5%). Use 0 to skip minimum.
- `--rpc`: Optional custom Ethereum RPC URL
- `--dry-run`: Simulate without broadcasting

**On-chain operations (requires user confirmation):**
- **Vault redeem**: vault.redeem(shares, receiver, owner, minimumReceive) -- ask user to confirm before submitting

**Examples:**
```
archimedes redeem --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269
archimedes redeem --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --shares 0.5
archimedes redeem --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 --dry-run
```

**Output:** JSON with tx hash, shares redeemed, expected underlying assets received.

---

## notes

- Vault factory is inactive -- vault addresses are hardcoded (3 known vaults)
- Most recently active vault: WETH ETH+ Strategy (0xfA364CB..., last tx Aug 2025)
- Funds sit idle in vault until offchain monitor triggers adjustIn(); yield accrues after rebalance
- minimumReceive is computed as: amount * (1 - slippage_bps / 10000)
- Use --dry-run to preview calldata before broadcasting any transaction

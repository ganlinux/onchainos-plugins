---
name: solayer
description: >-
  Use when the user asks about Solayer, restaking SOL, getting sSOL, liquid restaking on Solana,
  'stake SOL on Solayer', 'get sSOL', 'unrestake sSOL', 'check my sSOL balance',
  'check Solayer position', 'how much is my sSOL worth', 'Solayer restaking',
  'withdraw from Solayer', 'Solayer AVS', 'Solayer points',
  '质押SOL', '获取sSOL', '解质押sSOL', '查看Solayer仓位', '质押收益',
  or mentions Solayer protocol, sSOL liquid restaking token, or SOL restaking.
  Covers restaking SOL to receive sSOL, unrestaking sSOL back to SOL, querying balances,
  and viewing restaking positions with current exchange rate.
  Do NOT use for general Solana staking without Solayer. Do NOT use for Marinade or Lido.
license: MIT
metadata:
  author: skylavis-sky
  category: defi-protocol
  chain: Solana
  version: 0.1.0
  homepage: https://app.solayer.org
---

# Solayer Restaking Plugin

4 commands for native SOL restaking on Solayer: restake SOL to receive sSOL liquid restaking tokens, unrestake sSOL back to SOL, query balances, and view restaking positions with real-time exchange rate.

---

## Overview

Solayer is a hardware-accelerated native restaking protocol on Solana mainnet. Users stake SOL into the Solayer Stake Pool and receive sSOL (Liquid Restaking Token). sSOL holders accumulate Solayer points and can delegate to AVSs (Actively Validated Services) for additional yield. The exit process requires unrestaking sSOL through a 5-step on-chain transaction, after which SOL enters a deactivating stake account and becomes withdrawable after the current Solana epoch ends (typically 2–3 days).

---

## Pre-flight Checks

Before running any Solayer command, verify the following:

1. **onchainos CLI installed** (v2.0.0+):
   ```
   onchainos --version
   ```
   If not installed, visit https://docs.okx.com/onchainos

2. **solayer binary installed**:
   ```
   solayer --help
   ```
   If missing, build from source: `cargo build --release` in the plugin directory.

3. **Wallet logged in on Solana**:
   ```
   onchainos wallet address --chain 501
   ```
   If not logged in, run `onchainos wallet login` first.

4. **Sufficient SOL balance** (for restake operations, you need staked amount + ~0.01 SOL for fees):
   ```
   onchainos wallet balance --chain 501
   ```

---

## Commands

### 1. `restake` — Stake SOL to receive sSOL

**Usage:**
```
solayer restake --amount <SOL_AMOUNT> [--referrer <PARTNER_PUBKEY>] [--dry-run]
```

**Description:**
Calls the Solayer Partner Restake API to get an unsigned transaction, converts it from base64 to base58, and broadcasts it via `onchainos wallet contract-call --force`.

**Parameters:**
- `--amount`: Amount of SOL to restake (UI units, e.g., `1.0`). Minimum ~0.01 SOL.
- `--referrer`: (Optional) Partner wallet address for tracking referrals.
- `--dry-run`: Print the onchainos command without executing it.

**Example:**
```
solayer restake --amount 1.0
solayer restake --amount 0.5 --referrer So1ayerxxx...
```

**⚠️ IMPORTANT — Ask user to confirm before executing:**
Before running this command, present the user with:
> "You are about to restake [AMOUNT] SOL to Solayer. You will receive approximately [SSOL_AMOUNT] sSOL. The transaction will be signed via onchainos. Do you confirm? (yes/no)"

Only proceed after the user confirms.

**Expected output:**
```
Wallet: <PUBKEY>
SOL balance: 5.2100 SOL ✓
API message: restaking 1.0 SOL for 0.9854 sSOL
Broadcasting transaction via onchainos...

=== Restaking Successful ===
Transaction hash: <TX_HASH>
Explorer: https://solscan.io/tx/<TX_HASH>
```

**Notes:**
- The API uses SOL UI units (e.g., `1.0`), NOT lamports.
- The unsigned tx from the API is base64; onchainos requires base58 (conversion is automatic).
- `--force` is always added to ensure the transaction is broadcast (not just simulated).
- Solana blockhash expires in ~60 seconds — the call to onchainos happens immediately after receiving the API response.

---

### 2. `unrestake` — Unrestake sSOL to recover SOL

**Usage:**
```
solayer unrestake --amount <SSOL_AMOUNT> [--dry-run]
```

**Description:**
Initiates the 5-step unrestake process: burns sSOL, releases intermediate LST, creates a new stake account, withdraws from the Stake Pool, and deactivates the stake account. After deactivation, SOL is recoverable after the current Solana epoch ends (~2–3 days).

**Parameters:**
- `--amount`: Amount of sSOL to unrestake (UI units, e.g., `1.0`). Converted to lamports internally (1 sSOL = 1,000,000,000 lamports).
- `--dry-run`: Print instructions without executing.

**Example:**
```
solayer unrestake --amount 1.0
solayer unrestake --amount 0.5 --dry-run
```

**⚠️ IMPORTANT — Ask user to confirm before executing:**
Before running this command, present the user with:
> "You are about to unrestake [AMOUNT] sSOL from Solayer. The recovered SOL will enter a deactivating stake account and will not be immediately available — you must wait until the current Solana epoch ends (approximately 2–3 days) before you can withdraw SOL to your wallet. Do you confirm? (yes/no)"

Only proceed after the user confirms.

**Expected output (dry-run):**
```
Preparing to unrestake 1.000000 sSOL from Solayer...
Wallet: <PUBKEY>
sSOL balance: 2.450000 sSOL ✓

=== Important: Unrestake Process & Waiting Period ===
Unrestaking 1.000000 sSOL requires a 5-step on-chain transaction:
  1. Restaking Program unrestake() — burn sSOL, release intermediate LST
  2. Approve LST transfer from your LST token account
  3. Create a new stake account (requires ~0.0023 SOL rent)
  4. Withdraw stake from Solayer Stake Pool into the new stake account
  5. Deactivate the stake account
```

**Notes:**
- The 5-instruction unrestake transaction requires Anchor CPI and a freshly generated stake account keypair.
- For full execution, use the official Solayer TypeScript CLI: `npx solayer-cli unrestake --amount <AMOUNT>`
- After unrestaking, users must wait for epoch end (~2–3 days) then call `StakeProgram.withdraw` to move SOL to wallet.

---

### 3. `balance` — Query SOL and sSOL balances

**Usage:**
```
solayer balance [--rpc <RPC_URL>]
```

**Description:**
Queries both the native SOL balance (via `onchainos wallet balance --chain 501`) and the sSOL token balance (via Solana JSON-RPC `getTokenAccountsByOwner`).

**Example:**
```
solayer balance
solayer balance --rpc https://api.mainnet-beta.solana.com
```

**Expected output:**
```
=== Solayer Balance Summary ===
Wallet:       <PUBKEY>
SOL balance:  3.2100 SOL
sSOL balance: 2.453700 sSOL
```

**Notes:**
- No wallet write operations; no confirmation required.
- `onchainos wallet balance --chain 501` does NOT use `--output json` (not supported on Solana).

---

### 4. `positions` — View restaking positions and SOL value

**Usage:**
```
solayer positions [--rpc <RPC_URL>]
```

**Description:**
Queries sSOL balance and calculates its current SOL value using the live sSOL/SOL exchange rate from the Solayer Stake Pool account. Exchange rate = `total_lamports / pool_token_supply`.

**Example:**
```
solayer positions
```

**Expected output:**
```
=== Solayer Restaking Positions ===
Wallet:              <PUBKEY>

--- Liquid Assets ---
  SOL (available):   3.2100 SOL

--- Restaked Position ---
  sSOL balance:      2.453700 sSOL
  Exchange rate:     1 sSOL ≈ 1.014800 SOL
  Position value:    2.489927 SOL
  Cumulative yield:  +1.48%

--- Total Portfolio ---  5.700027 SOL (liquid + restaked)
```

**Notes:**
- No wallet write operations; no confirmation required.
- Exchange rate is fetched live from the Stake Pool program account (`po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2`).

---

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| `onchainos not found` | onchainos CLI not installed | Install onchainos from https://docs.okx.com/onchainos |
| `onchainos wallet address failed` | Not logged in | Run `onchainos wallet login` |
| `Insufficient SOL balance` | Not enough SOL for restake + fees | Add more SOL to wallet or reduce restake amount |
| `Insufficient sSOL balance` | Not enough sSOL for unrestake | Check balance with `solayer balance` |
| `Solayer Restake API returned HTTP 4xx` | Invalid parameters or API issue | Check amount > 0, wallet address is valid |
| `Failed to decode base64 transaction` | Malformed API response | Retry; API may be temporarily down |
| `onchainos wallet contract-call failed` | Transaction simulation failure or RPC error | Check SOL balance, network congestion; retry |
| `txHash: pending` | Missing `--force` flag | Plugin always adds `--force`; if seen, file a bug |
| `Could not parse wallet address` | Unexpected onchainos output format | Update onchainos to latest version |
| Exchange rate shows `1.015` (approx) | RPC returned binary-encoded account data | Use a different RPC with `--rpc` flag |

---

## Skill Routing

| User Intent | Route to |
|-------------|----------|
| Stake SOL on Solayer | `solayer restake` |
| Get sSOL | `solayer restake` |
| Unrestake sSOL / withdraw from Solayer | `solayer unrestake` |
| Check sSOL balance / wallet balance | `solayer balance` |
| View position value / how much is my sSOL worth | `solayer positions` |
| General Solana staking (not Solayer) | Use Marinade or Lido skill |
| SOL/sSOL swap on DEX | Use Jupiter or Orca skill |
| Solayer AVS delegation | Not yet supported; direct user to https://app.solayer.org |
| Withdraw deactivated stake account | Not yet supported; use `solana-cli stake withdraw` |

---

## Key Concepts

- **sSOL**: Solayer Liquid Restaking Token (LRT). Minted when SOL is restaked via Solayer. Accumulates Solayer points. Exchange rate increases over time as staking rewards accrue.
- **Exchange Rate**: `total_lamports / pool_token_supply`. Example: 1 sSOL ≈ 1.0148 SOL.
- **Unrestake waiting period**: After unrestaking, SOL enters a deactivating stake account. It becomes withdrawable after the current Solana epoch ends (~2–3 days).
- **Solana chain ID**: `501` for all onchainos commands targeting Solana mainnet.
- **`--force` flag**: Required for all Solana write operations via onchainos. Without it, transactions return `txHash: "pending"` and are never broadcast.

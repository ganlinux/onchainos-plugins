---
name: kerneldao-restaking
description: >-
  Use when the user asks about KernelDAO, Kernel restaking, staking BTC on BSC,
  'stake BTCB', 'stake SolvBTC', 'stake BNB KernelDAO', 'unstake BTCB',
  'unstake BNB KernelDAO', 'KernelDAO balance', 'Kernel Points', 'restake BTC BSC',
  'restake BNB BSC', 'KernelDAO TVL', 'KernelDAO supported assets',
  or mentions KernelDAO, Kernel restaking, BTCB restaking, SolvBTC restaking,
  BNB restaking on BSC, staking BTC derivatives, or earning Kernel Points.
  Covers: balance query, ERC-20 stake (approve + stake), native BNB stake,
  ERC-20 unstake, native BNB unstake, and supported asset listing.
  Do NOT use for general BSC token swaps unrelated to KernelDAO.
  Do NOT use for Kelp (Ethereum rsETH) or Gain vaults.
license: MIT
metadata:
  author: skylavis-sky
  version: "0.1.0"
---

# KernelDAO Restaking Plugin

KernelDAO is a multi-chain restaking protocol on BNB Chain. Users stake BTC derivatives
(BTCB, SolvBTC, uniBTC, stBTC, pumpBTC, mBTC, SolvBTC.BBN) and BNB derivatives
(WBNB, slisBNB, BNBx, asBNB) to earn Kernel Points, which may convert to KERNEL token rewards.
Native BNB can also be staked directly without wrapping.

## Pre-flight Checks

Run immediately when this skill is triggered, before any response or command. Always follow in order.

1. **Check onchainos**: `which onchainos` -- if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet; if found, run `onchainos --version` and verify version is >= 2.0.0
2. **Check binary**: `which kerneldao-restaking` -- if not found, install via `plugin-store install kerneldao-restaking`
3. **Check wallet login**: `onchainos wallet status` -- must show `loggedIn: true` and have a BSC address; if not, run `onchainos wallet login`
4. **For stake/unstake**: verify sufficient BNB balance for gas on BSC (chain 56)

## Commands

### `kerneldao-restaking balance` -- Query Staked Balance

**Triggers:** "KernelDAO balance", "my BTCB staked", "Kernel restaking position", "how much BNB staked KernelDAO",
"KernelDAO holdings", "list my KernelDAO positions", "show all Kernel stakes"

```bash
kerneldao-restaking balance
kerneldao-restaking balance --asset <TOKEN_ADDRESS>
```

**Parameters:**
- `--asset <ADDRESS>` (optional) -- ERC-20 token address to query. If omitted, queries all known assets and shows non-zero balances.

**Output:** Token symbol, staked amount (human-readable), raw wei amount.

**Examples:**
```bash
# Show all positions with non-zero balance
kerneldao-restaking balance

# Query only BTCB staked balance
kerneldao-restaking balance --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c
```

---

### `kerneldao-restaking stake` -- Stake ERC-20 Token

**Triggers:** "stake BTCB KernelDAO", "restake SolvBTC", "deposit BTCB KernelDAO",
"stake BTC KernelDAO", "stake uniBTC", "stake pumpBTC", "stake WBNB KernelDAO",
"stake slisBNB", "earn Kernel Points"

```bash
kerneldao-restaking stake --asset <TOKEN_ADDRESS> --amount <AMOUNT>
kerneldao-restaking --dry-run stake --asset <TOKEN_ADDRESS> --amount <AMOUNT>
```

**Parameters:**
- `--asset <ADDRESS>` -- ERC-20 token address to stake (e.g. BTCB: `0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c`)
- `--amount <AMOUNT>` -- Amount in human-readable units (e.g. `0.001` for 0.001 BTCB)
- `--referral <CODE>` (optional) -- Referral code string (default: empty string)

**Flow:**
1. Resolve wallet address on BSC (chain 56)
2. Ask user to confirm the approve transaction (ERC-20 approve for StakerGateway)
3. Submit approve tx and wait for confirmation
4. Ask user to confirm the stake transaction
5. Submit stake tx

**IMPORTANT:** Ask user to confirm before submitting the approve transaction and again before the stake transaction.

**Examples:**
```bash
# Stake 0.001 BTCB
kerneldao-restaking stake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --amount 0.001

# Dry-run: simulate staking 0.5 SolvBTC
kerneldao-restaking --dry-run stake --asset 0x4aae823a6a0b376De6A78e74eCC5b079d38cBCf7 --amount 0.5
```

**Notes:**
- ERC-20 assets require an approve step before staking. The plugin handles this automatically.
- Decimals for BTCB and most BTC assets are 18.
- After staking, Kernel Points accumulate in your account automatically.

---

### `kerneldao-restaking stake-native` -- Stake Native BNB

**Triggers:** "stake BNB KernelDAO", "stake native BNB", "restake BNB Kernel",
"deposit BNB KernelDAO", "earn Kernel Points with BNB"

```bash
kerneldao-restaking stake-native --amount <BNB_AMOUNT>
kerneldao-restaking --dry-run stake-native --amount <BNB_AMOUNT>
```

**Parameters:**
- `--amount <BNB>` -- Amount of native BNB to stake (human-readable units, e.g. `0.01`)
- `--referral <CODE>` (optional) -- Referral code string (default: empty string)

**IMPORTANT:** Ask user to confirm before submitting the stake transaction. This sends native BNB as msg.value.

**Examples:**
```bash
# Stake 0.01 BNB
kerneldao-restaking stake-native --amount 0.01

# Dry-run
kerneldao-restaking --dry-run stake-native --amount 0.1
```

**Notes:**
- Native BNB staking does NOT require an approve step.
- BNB is sent as msg.value in the transaction.

---

### `kerneldao-restaking unstake` -- Unstake ERC-20 Token

**Triggers:** "unstake BTCB KernelDAO", "withdraw BTCB Kernel", "redeem SolvBTC KernelDAO",
"exit KernelDAO position", "remove BTCB from Kernel"

```bash
kerneldao-restaking unstake --asset <TOKEN_ADDRESS> --amount <AMOUNT>
kerneldao-restaking --dry-run unstake --asset <TOKEN_ADDRESS> --amount <AMOUNT>
```

**Parameters:**
- `--asset <ADDRESS>` -- ERC-20 token address to unstake
- `--amount <AMOUNT>` -- Amount in human-readable units (e.g. `0.001`)
- `--referral <CODE>` (optional) -- Referral code string (default: empty string)

**IMPORTANT:** Ask user to confirm before submitting the unstake transaction. Warn user that unstaking initiates a 7-14 day unbonding period.

**Examples:**
```bash
# Unstake 0.001 BTCB
kerneldao-restaking unstake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --amount 0.001
```

**Notes:**
- After unstaking, there is a **7-14 day unbonding period** before tokens can be claimed.
- The plugin initiates the unbonding request only. Claiming after the period requires a separate operation.

---

### `kerneldao-restaking unstake-native` -- Unstake Native BNB

**Triggers:** "unstake BNB KernelDAO", "withdraw native BNB Kernel", "exit BNB position KernelDAO"

```bash
kerneldao-restaking unstake-native --amount <BNB_AMOUNT>
kerneldao-restaking --dry-run unstake-native --amount <BNB_AMOUNT>
```

**Parameters:**
- `--amount <BNB>` -- Amount of native BNB to unstake (human-readable units)
- `--referral <CODE>` (optional) -- Referral code string (default: empty string)

**IMPORTANT:** Ask user to confirm before submitting the unstake transaction. Warn user that unstaking initiates a 7-14 day unbonding period.

**Examples:**
```bash
# Unstake 0.01 BNB
kerneldao-restaking unstake-native --amount 0.01
```

**Notes:**
- After unstaking, there is a **7-14 day unbonding period** before BNB can be claimed.

---

## Supported Assets (BSC Mainnet)

| Type | Token | Address |
|------|-------|---------|
| BNB  | WBNB    | `0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c` |
| BNB  | slisBNB | `0xb0b84d294e0c75a6abe60171b70edeb2efd14a1b` |
| BNB  | BNBx    | `0x1bdd3cf7f79cfb8edbb955f20ad99211551ba275` |
| BNB  | asBNB   | `0x77734e70b6e88b4d82fe632a168edf6e700912b6` |
| BTC  | BTCB    | `0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c` |
| BTC  | SolvBTC | `0x4aae823a6a0b376De6A78e74eCC5b079d38cBCf7` |
| BTC  | SolvBTC.BBN | `0x1346b618dc92810ec74163e4c27004c921d446a5` |
| BTC  | uniBTC  | `0x6b2a01a5f79deb4c2f3c0eda7b01df456fbd726a` |
| BTC  | stBTC   | `0xf6718b2701d4a6498ef77d7c152b2137ab28b8a3` |
| BTC  | pumpBTC | `0xf9C4FF105803A77eCB5DAE300871Ad76c2794fa4` |
| BTC  | mBTC    | `0x9BFA177621119e64CecbEabE184ab9993E2ef727` |

## Key Contracts (BSC Mainnet)

| Name | Address |
|------|---------|
| StakerGateway | `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394` |
| AssetRegistry | `0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB` |

## Error Handling

| Error | Cause | Solution |
|-------|-------|---------|
| `Could not resolve wallet address` | onchainos not logged in or no BSC address | Run `onchainos wallet login` |
| `eth_call failed` | BSC RPC unreachable or wrong calldata | Check network connectivity; verify asset address |
| `amount too small` | Amount converts to 0 wei | Use a larger amount with correct decimals |
| `approve tx failed` | Insufficient token balance or gas | Check balance with `onchainos wallet balance --chain 56` |
| `stake tx failed` | Allowance not set or insufficient balance | Re-run; ensure approve succeeded first |

## Skill Routing

- For **general BSC token swaps**: use `okx-dex-swap` skill
- For **Kelp rsETH staking on Ethereum**: use a dedicated Kelp plugin (not this skill)
- For **checking BSC wallet balance**: `onchainos wallet balance --chain 56`
- For **BNB liquid staking (slisBNB, BNBx)**: use respective liquid staking protocols to obtain the LST first, then stake here

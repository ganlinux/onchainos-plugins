---
name: jito
description: >-
  Use when the user asks about Jito, JitoSOL liquid staking, staking SOL on Solana,
  'stake SOL', 'get JitoSOL', 'unstake JitoSOL', 'JitoSOL to SOL', 'sell JitoSOL',
  'Jito APY', 'JitoSOL exchange rate', 'Jito MEV rewards', 'Jito pool info',
  'Jito restaking', 'deposit JitoSOL vault', 'withdraw from Jito vault',
  'Jito NCN vault', 'mint VRT', 'redeem VRT',
  '质押 SOL', '质押收益', '买 JitoSOL', '兑换 JitoSOL', 'JitoSOL 余额',
  '再质押', '存入 Jito Vault', '从 Jito Vault 取回',
  or mentions Jito, JitoSOL, Jito liquid staking, Jito restaking, MEV staking rewards on Solana.
  Covers: pool info, SOL staking, instant unstake via DEX, positions, restaking vault listing, deposit, and withdrawal.
  Do NOT use for general Solana swaps unrelated to Jito. Do NOT use for Marinade or other LSTs.
license: Apache-2.0
metadata:
  author: oker
  version: "0.1.0"
---

# Jito Plugin

Jito is Solana's largest liquid staking protocol. Users stake SOL to receive JitoSOL (LST) and earn staking yield plus MEV bundle tip revenue (~5.62% APY). Jito also offers a Restaking protocol for depositing JitoSOL into NCN Vaults for additional yield.

## Pre-flight Checks

Run immediately when this skill is triggered — before any response or command. Always follow in order.

1. **Check onchainos**: `which onchainos` — if not found, tell user to install from https://web3.okx.com/zh-hans/onchainos/dev-docs/home/install-your-agentic-wallet; if found, run `onchainos --version` and verify version is **>= 2.0.0**
2. **Check binary**: `which jito` — if not found, install via `plugin-store install jito`
3. **Check wallet login**: `onchainos wallet status` — must show `loggedIn: true` and have a Solana address; if not, run `onchainos wallet login`
4. **For stake/unstake**: verify sufficient SOL balance (minimum 0.01 SOL + gas)

## Commands

### `jito info` — Pool Info

**Triggers:** "Jito APY", "JitoSOL exchange rate", "Jito MEV rewards", "Jito pool info", "Jito TVL", "JitoSOL 收益", "Jito 质押信息"

```bash
jito info
```

Output: APY (including MEV share), SOL price, JitoSOL total supply, current epoch, MEV reward rate, key addresses.

---

### `jito stake` — Stake SOL for JitoSOL

**Triggers:** "stake SOL", "stake SOL to Jito", "buy JitoSOL", "deposit SOL Jito", "get JitoSOL", "质押 SOL", "买 JitoSOL"

```bash
jito stake --amount <SOL_AMOUNT>
jito --dry-run stake --amount 0.01
```

**Parameters:**
- `--amount <SOL>` — Amount of SOL to stake (UI units, e.g. `0.01`)

**Constraints:**
- Minimum: 0.01 SOL
- The serialized transaction expires in 60 seconds; broadcasting happens immediately after fetch
- `--dry-run` flag goes before the subcommand: `jito --dry-run stake --amount 0.01`

**Examples:**
```bash
# Stake 0.1 SOL
jito stake --amount 0.1

# Simulate without broadcasting
jito --dry-run stake --amount 0.5
```

---

### `jito unstake` — Instantly Exchange JitoSOL for SOL

**Triggers:** "unstake JitoSOL", "exchange JitoSOL for SOL", "sell JitoSOL", "JitoSOL to SOL", "兑换 JitoSOL", "卖 JitoSOL"

```bash
jito unstake --amount <JITOSOL_AMOUNT>
jito unstake --amount <JITOSOL_AMOUNT> --slippage <PERCENT>
jito --dry-run unstake --amount 0.5
```

**Parameters:**
- `--amount <JitoSOL>` — Amount of JitoSOL to swap (UI units, e.g. `0.5`)
- `--slippage <percent>` — Max slippage percentage (default: `1.0`)

**Notes:**
- Instant exchange via Jupiter DEX — no waiting period
- Traditional SPL Stake Pool withdrawal (2-3 day epoch wait) is not supported in v1

---

### `jito positions` — View Holdings

**Triggers:** "my JitoSOL balance", "Jito positions", "JitoSOL holdings", "我的 JitoSOL 余额", "Jito 持仓"

```bash
jito positions
```

Output: JitoSOL balance, USD value, current APY.

---

### `jito restake-vaults` — List Restaking Vaults

**Triggers:** "Jito restaking vaults", "Jito NCN vaults", "Jito vault list", "Jito 再质押 Vault 列表", "list Jito vaults"

```bash
jito restake-vaults
```

Output: Sample of 20 Vault addresses + link to full list at jito.network.

---

### `jito restake-deposit` — Deposit JitoSOL into a Vault

**Triggers:** "deposit JitoSOL vault", "Jito restaking deposit", "mint VRT", "存入 Jito Vault", "Jito restake deposit"

```bash
jito restake-deposit --vault <VAULT_ADDRESS> --amount <JITOSOL_AMOUNT>
jito --dry-run restake-deposit --vault <VAULT_ADDRESS> --amount 1.0
```

**Parameters:**
- `--vault <ADDRESS>` — Vault address (base58 Solana public key)
- `--amount <JitoSOL>` — Amount to deposit (UI units)

**Note (v1):** Live mode provides guidance to use the Jito web interface. Vault SDK integration is planned for v2.

---

### `jito restake-withdraw` — Initiate Vault Withdrawal

**Triggers:** "withdraw from Jito vault", "redeem VRT", "Jito restaking withdraw", "从 Jito Vault 取回", "Jito vault withdrawal"

```bash
jito restake-withdraw --vault <VAULT_ADDRESS> --amount <VRT_AMOUNT>
jito --dry-run restake-withdraw --vault <VAULT_ADDRESS> --amount 1.0
```

**Parameters:**
- `--vault <ADDRESS>` — Vault address (base58 Solana public key)
- `--amount <VRT>` — Amount of VRT tokens to redeem (UI units)

**Note:** This initiates an `EnqueueWithdrawal` request only. After the cooldown period, visit https://www.jito.network/restaking/ to complete the final withdrawal (`BurnWithdrawalTicket`).

---

## Key Addresses

| Name | Address |
|------|---------|
| JitoSOL Mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` |
| SPL Stake Pool Program | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` |
| Jito Stake Pool Account | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` |
| Jito Vault Program | `Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8` |
| Jito Restaking Program | `RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q` |
| onchainos investmentId | `22414` |

## Error Handling

| Error | Cause | Solution |
|-------|-------|---------|
| `Could not find Solana address` | onchainos not logged in | Run `onchainos wallet login` |
| `No serializedData in defi invest` | Amount too small or network error | Confirm amount >= 0.01 SOL; retry |
| `Minimum stake amount is 0.01 SOL` | Amount below minimum | Use `--amount >= 0.01` |
| `transaction simulation failed` | Tx expired (>60s) or insufficient balance | Re-run `jito stake` immediately; ensure enough SOL |
| `swap execute parse error` | Network error or invalid amount | Check JitoSOL balance with `jito positions` |

## Skill Routing

- For **general Solana token swaps** not involving JitoSOL: delegate to `okx-dex-swap`
- For **Marinade mSOL staking**: use a dedicated Marinade plugin (not this skill)
- For **checking Solana wallet SOL balance**: `onchainos wallet balance --chain 501`

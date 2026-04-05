---
name: swellchain-staking
description: >-
  Use when the user asks about Swell Network staking, Swell liquid staking,
  swETH staking, rswETH restaking, Swell Earn pool, SimpleStakingERC20,
  'stake ETH on Swell', 'deposit into Swell Earn', 'withdraw from Swell',
  'swETH withdrawal', 'finalize Swell withdrawal', 'check Swell balance',
  'Swell staking positions', 'Swell Network', 'swellchain staking',
  or mentions swETH, rswETH, swEXIT, SimpleStakingERC20 on Ethereum.
  Covers: stake, earn-deposit, earn-withdraw, request-withdrawal,
  finalize-withdrawal, balance, positions.
  Do NOT use for Lido stETH staking -- use lido skill instead.
  Do NOT use for Rocketpool rETH -- use rocketpool skill instead.
  Do NOT use for EigenLayer restaking directly -- use eigenlayer skill instead.
license: MIT
metadata:
  author: ganlinux
  version: "0.1.0"
---

# swellchain-staking

Swell Network liquid staking and restaking on Ethereum L1.
Supports swETH (LST, ~3% APR), rswETH (LRT, ~2.63% APR + EigenLayer rewards),
and SimpleStakingERC20 Earn pool for additional yield.

## Architecture

- Write ops (stake, earn-deposit, earn-withdraw, request-withdrawal, finalize-withdrawal) ->
  after user confirmation, submits via `onchainos wallet contract-call`
- Read ops (balance, positions) -> direct `eth_call` via public Ethereum RPC; no confirmation needed
- All operations on Ethereum Mainnet (chain_id=1)

## Execution Flow for Write Operations

1. Run with `--dry-run` first to preview calldata
2. **Ask user to confirm** before executing on-chain
3. Execute only after explicit user approval
4. Report transaction hash and outcome

---

## commands

### stake

Deposit ETH and receive swETH (liquid staking token). Uses `deposit()` payable on swETH proxy.
No ERC-20 approve needed -- ETH is sent as msg.value.

**Ask user to confirm** the staking transaction before proceeding.

```
swellchain-staking stake --amt <wei> [--from <address>] [--dry-run]
```

**Parameters:**
- `--amt`: Amount of ETH to stake in wei (e.g. `500000000000000000` for 0.5 ETH)
- `--from`: Optional sender address (defaults to logged-in wallet)
- `--dry-run`: Preview calldata without broadcasting

**Examples:**
```
swellchain-staking stake --amt 500000000000000000
swellchain-staking stake --amt 1000000000000000000 --dry-run
```

**On-chain operation:**
- Contract: `0xf951E335afb289353dc249e82926178EaC7DEd78` (swETH Proxy)
- Function: `deposit()` payable, selector `0xd0e30db0`
- ETH value sent as `--amt` parameter

---

### earn-deposit

Deposit swETH or rswETH into SimpleStakingERC20 Earn pool for additional yield and Swell points.
Performs two transactions: approve + deposit.

**Ask user to confirm** both the approve and deposit transactions before proceeding.

```
swellchain-staking earn-deposit --token <swETH|rswETH> --amt <wei> [--from <address>] [--dry-run]
```

**Parameters:**
- `--token`: Token to deposit: `swETH` or `rswETH` (default: `swETH`)
- `--amt`: Amount in wei
- `--from`: Optional sender/receiver address (defaults to logged-in wallet)
- `--dry-run`: Preview calldatas without broadcasting

**Examples:**
```
swellchain-staking earn-deposit --token swETH --amt 1000000000000000000
swellchain-staking earn-deposit --token rswETH --amt 500000000000000000 --dry-run
```

**On-chain operations (each requires user confirmation):**
1. **ERC-20 approve**: token.approve(SimpleStakingERC20, amount)
   - Contract: swETH or rswETH proxy address
2. **deposit**: SimpleStakingERC20.deposit(token, amount, receiver)
   - Contract: `0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774`
   - Function: `deposit(address,uint256,address)`, selector `0xf45346dc`

---

### earn-withdraw

Withdraw swETH or rswETH from SimpleStakingERC20 Earn pool. Instant withdrawal, no lock-up period.

**Ask user to confirm** the withdrawal transaction before proceeding.

```
swellchain-staking earn-withdraw --token <swETH|rswETH> --amt <wei> [--from <address>] [--dry-run]
```

**Parameters:**
- `--token`: Token to withdraw: `swETH` or `rswETH` (default: `swETH`)
- `--amt`: Amount in wei
- `--from`: Optional sender/receiver address (defaults to logged-in wallet)
- `--dry-run`: Preview calldata without broadcasting

**Examples:**
```
swellchain-staking earn-withdraw --token swETH --amt 1000000000000000000
swellchain-staking earn-withdraw --token rswETH --amt 500000000000000000
```

**On-chain operation:**
- Contract: `0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774` (SimpleStakingERC20)
- Function: `withdraw(address,uint256,address)`, selector `0x69328dec`

---

### request-withdrawal

Create a swETH withdrawal request to redeem swETH back to ETH.
Performs two transactions: approve swEXIT + createWithdrawRequest.
Creates a swEXIT NFT. Withdrawal takes 1-12 days to process.

**Ask user to confirm** both the approve and createWithdrawRequest transactions before proceeding.

```
swellchain-staking request-withdrawal --amt <wei> [--from <address>] [--dry-run]
```

**Parameters:**
- `--amt`: Amount of swETH to withdraw in wei
- `--from`: Optional sender address (defaults to logged-in wallet)
- `--dry-run`: Preview calldatas without broadcasting

**Examples:**
```
swellchain-staking request-withdrawal --amt 100000000000000000
swellchain-staking request-withdrawal --amt 500000000000000000 --dry-run
```

**On-chain operations (each requires user confirmation):**
1. **ERC-20 approve**: swETH.approve(swEXIT, amount)
   - Contract: `0xf951E335afb289353dc249e82926178EaC7DEd78` (swETH Proxy)
2. **createWithdrawRequest**: swEXIT.createWithdrawRequest(amount)
   - Contract: `0x48C11b86807627AF70a34662D4865cF854251663` (swEXIT Proxy)
   - Function: `createWithdrawRequest(uint256)`, selector `0x74dc9d1a`

**After submission:** Note the swEXIT NFT tokenId. Use `finalize-withdrawal` once processed.

---

### finalize-withdrawal

Finalize a processed swEXIT withdrawal to receive ETH.
Checks `getProcessedRateForTokenId` before submitting -- fails if not yet processed.

**Ask user to confirm** the finalization transaction before proceeding.

```
swellchain-staking finalize-withdrawal --token-id <id> [--from <address>] [--dry-run]
```

**Parameters:**
- `--token-id`: swEXIT NFT token ID to finalize
- `--from`: Optional sender address (defaults to logged-in wallet)
- `--dry-run`: Preview calldata without broadcasting

**Examples:**
```
swellchain-staking finalize-withdrawal --token-id 1234
swellchain-staking finalize-withdrawal --token-id 5678 --dry-run
```

**On-chain operation:**
- Contract: `0x48C11b86807627AF70a34662D4865cF854251663` (swEXIT Proxy)
- Function: `finalizeWithdrawal(uint256)`, selector `0x5e15c749`

---

### balance

Query swETH and rswETH balances plus current exchange rates for an address.

```
swellchain-staking balance --address <address>
```

**Parameters:**
- `--address`: Ethereum wallet address to query

**Examples:**
```
swellchain-staking balance --address 0xYourAddress
```

**Output:** swETH balance, rswETH balance, ETH equivalent values, exchange rates.

---

### positions

Show complete staking positions and pending withdrawals for an address.

```
swellchain-staking positions --address <address>
```

**Parameters:**
- `--address`: Ethereum wallet address to query

**Examples:**
```
swellchain-staking positions --address 0xYourAddress
```

**Output:** swETH/rswETH holdings with ETH values, Earn pool info, swEXIT withdrawal queue status.

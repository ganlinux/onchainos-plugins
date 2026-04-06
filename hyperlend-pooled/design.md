# HyperLend Pooled — Plugin Store PRD

> Enable AI Agents to supply, borrow, repay, and withdraw on HyperLend Core Pools (Aave V3.2 fork) on Hyperliquid EVM (chain ID 999) via onchainos CLI.

---

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `hyperlend-pooled` |
| dapp_name | HyperLend Pooled |
| record_id | `recvfIWnsW8qz0` |
| dapp_repo | https://github.com/hyperlendx/hyperlend-core |
| dapp_alias | hyperlend, hyperlend pooled, hyperlend core pools |
| one_liner | HyperLend Core Pools — Aave V3 fork on Hyperliquid EVM: supply, borrow, repay, withdraw |
| category | lending |
| tags | lending, borrowing, hyperliquid, evm, defi, yield, aave-v3-fork |
| target_chains | Hyperliquid EVM (chain ID 999) |
| target_protocols | HyperLend Core Pools |

---

## 1. Feasibility Research

### What is this DApp

HyperLend is a decentralized money market protocol built natively on HyperEVM (Hyperliquid's EVM-compatible smart contract layer). It launched on mainnet in March 2025 and is one of the core DeFi protocols on HyperEVM by TVL.

HyperLend operates a **three-tier architecture**:
1. **Core Pools** (this plugin) — multi-asset pooled lending based on **Aave V3.2** (friendly fork). Multiple tokens share a single liquidity pool. Supports supply, borrow, repay, withdraw, e-mode, and collateral management.
2. **Isolated Pools** — two-token isolated markets (FraxLend V3 fork), separate plugin.
3. **P2P Pools** — fully customized peer-to-peer lending agreements, separate plugin.

This plugin covers **Core Pools only**.

### Fork Details

HyperLend Core is a **friendly fork of Aave V3.2** (confirmed by GitHub repo: `hyperlendx/hyperlend-core`). Key differences from Aave V3.0:
- Stable rate borrowing removed (Aave V3.2 removed this)
- Liquid eModes feature included
- Native token is HYPE (18 decimals) — same role as ETH on Ethereum
- hTokens (yield-bearing supply tokens) instead of aTokens

The Pool ABI is **identical to Aave V3** for all core operations: `supply`, `borrow`, `repay`, `withdraw`, `setUserUseReserveAsCollateral`, `setUserEMode`, `getUserAccountData`.

### Feasibility Table

| Check | Result |
|-------|--------|
| Fork of known protocol? | Yes — Aave V3.2 (friendly fork, confirmed in GitHub README) |
| Chain supported by onchainos? | Chain ID 999 (HyperEVM mainnet) — standard EVM JSON-RPC, compatible |
| Pool contract address known? | Yes — `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` (from official docs) |
| PoolAddressesProvider known? | Yes — `0x72c98246a98bFe64022a3190e7710E157497170C` |
| ABI available? | Yes — identical to Aave V3 Pool ABI (verified function selectors below) |
| REST API available? | Yes — `https://api.hyperlend.finance` with `/data/markets` and `/data/markets/rates` |
| RPC endpoint? | `https://rpc.hyperliquid.xyz/evm` (public, rate-limited 100 req/min) |
| Write ops via onchainos? | Yes — EVM `wallet contract-call --chain 999 --to <pool> --input-data <calldata>` |
| ERC-20 approval needed? | Yes — for `supply` and `repay` (standard ERC-20 `approve` to Pool address) |
| Subgraph / indexer? | Yes — Ponder indexer at `ponder.hyperlend.finance`; REST API at `api.hyperlend.finance` |
| Supported assets? | USDC, wHYPE, HYPE (native wrapped), uBTC, uETH, USDT0, stHYPE, wstHYPE, kHYPE |
| Native HYPE wrapping needed? | Yes — native HYPE must be wrapped to wHYPE (`0x5555...5555`) before supplying |

### Integration Path

**Path: Direct EVM contract calls via onchainos**

Since HyperLend Core Pools is an Aave V3.2 fork, the integration path is:
1. **Read ops** (markets, rates, positions): call `api.hyperlend.finance` REST API or direct `eth_call` to Pool / ProtocolDataProvider
2. **Write ops** (supply, borrow, repay, withdraw): ABI-encode calldata and call `onchainos wallet contract-call --chain 999`

No intermediary transaction-building API is needed — calldata can be constructed directly from the well-known Aave V3 ABI.

---

## 2. Interface Mapping

### Operations Summary

| # | Operation | Type | Description |
|---|-----------|------|-------------|
| 1 | `get-markets` | Read | Fetch all Core Pool reserves: APY, TVL, utilization, risk params |
| 2 | `positions` | Read | Fetch user's supply/borrow positions and health factor |
| 3 | `supply` | Write | Supply asset to Core Pool, receive hToken |
| 4 | `borrow` | Write | Borrow asset against supplied collateral |
| 5 | `repay` | Write | Repay outstanding borrow debt |
| 6 | `withdraw` | Write | Withdraw supplied asset, burn hToken |
| 7 | `set-collateral` | Write | Enable/disable asset as collateral |
| 8 | `set-emode` | Write | Set efficiency mode category for correlated assets |

---

### 2.1 get-markets (Read)

**Method:** REST API

```
GET https://api.hyperlend.finance/data/markets?chain=hyperEvm
```

Returns all Core Pool reserves with:
- `symbol`, `address` (underlying token)
- `supplyApy`, `borrowApy`
- `totalSupplyUsd`, `totalBorrowUsd`
- `utilizationRate`
- `ltv`, `liquidationThreshold`, `liquidationBonus`
- `isActive`, `isFrozen`, `isBorrowingEnabled`

**Supplementary rate data:**
```
GET https://api.hyperlend.finance/data/markets/rates
```

**Fallback via eth_call** — `ProtocolDataProvider.getAllReservesTokens()`:
```
Contract: 0x5481bf8d3946E6A3168640c1D7523eB59F055a29
Selector: getAllReservesTokens() → (string symbol, address tokenAddress)[]
```

---

### 2.2 positions (Read)

**Method:** eth_call to Pool — `getUserAccountData(address)`

```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  getUserAccountData(address user)
Selector:  0xbf92857c
Input:     0xbf92857c + <wallet_address padded to 32 bytes>
Returns:   (totalCollateralBase, totalDebtBase, availableBorrowsBase,
            currentLiquidationThreshold, ltv, healthFactor)
           All values in USD with 8 decimals (base currency units).
           healthFactor has 18 decimals (1e18 = HF 1.0).
```

**Per-asset breakdown** via `ProtocolDataProvider.getUserReserveData(asset, user)`:
```
Contract: 0x5481bf8d3946E6A3168640c1D7523eB59F055a29
Function:  getUserReserveData(address asset, address user)
Selector:  0x28dd0f6e
Returns:   (currentATokenBalance, currentStableDebt [always 0 in V3.2],
            currentVariableDebt, scaledVariableDebt, liquidityRate,
            usageAsCollateralEnabled, ...)
```

**Via REST API (preferred for rich data):**
```
GET https://api.hyperlend.finance/data/markets?chain=hyperEvm&user=<wallet_address>
```

---

### 2.3 supply (Write)

**Step 1 — ERC-20 Approve:**
```
Contract: <asset_address>  (e.g., USDC: 0xb88339CB7199b77E23DB6E890353E22632Ba630f)
Function:  approve(address spender, uint256 amount)
Selector:  0x095ea7b3
Calldata:  0x095ea7b3
           + <pool_address padded 32 bytes>  (0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b)
           + <amount padded 32 bytes>         (raw token units)
```

onchainos call:
```bash
onchainos wallet contract-call \
  --chain 999 \
  --to <asset_address> \
  --input-data 0x095ea7b3<pool_addr_padded><amount_padded>
```

**Step 2 — Pool.supply (after 3-second delay):**
```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  supply(address asset, uint256 amount, address onBehalfOf, uint16 referralCode)
Selector:  0x617ba037
Params:
  asset        — ERC-20 token address
  amount       — raw token units (e.g., 1 USDC = 1_000_000 for 6 decimals)
  onBehalfOf   — user wallet address (same as caller)
  referralCode — 0 (unused)
```

onchainos call:
```bash
onchainos wallet contract-call \
  --chain 999 \
  --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
  --input-data 0x617ba037<asset_padded><amount_padded><onBehalfOf_padded><referralCode_padded>
```

**Note:** User receives hToken (hUSDC, hwHYPE, etc.) in return. hTokens are yield-bearing ERC-20s.

---

### 2.4 borrow (Write)

No approval needed. Pool mints VariableDebtToken directly.

```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  borrow(address asset, uint256 amount, uint256 interestRateMode, uint16 referralCode, address onBehalfOf)
Selector:  0xa415bcad
Params:
  asset            — ERC-20 token address to borrow
  amount           — raw token units
  interestRateMode — 2 (variable; stable rate was removed in V3.2)
  referralCode     — 0
  onBehalfOf       — user wallet address
```

onchainos call:
```bash
onchainos wallet contract-call \
  --chain 999 \
  --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
  --input-data 0xa415bcad<asset_padded><amount_padded><0000...0002><0000...0000><onBehalfOf_padded>
```

**Prerequisite:** User must have supplied collateral with `setUserUseReserveAsCollateral` enabled, and health factor must be > 1.0 after the borrow.

---

### 2.5 repay (Write)

**Step 1 — ERC-20 Approve** (same pattern as supply, see §2.3):
```
approve(<pool_address>, <amount>)  on the borrowed asset
```

**Step 2 — Pool.repay (after 3-second delay):**
```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  repay(address asset, uint256 amount, uint256 interestRateMode, address onBehalfOf)
Selector:  0x573ade81
Params:
  asset            — ERC-20 token address (same as borrowed)
  amount           — raw token units, or wallet balance for "repay all" (see pitfall below)
  interestRateMode — 2 (variable)
  onBehalfOf       — user wallet address
```

onchainos call:
```bash
onchainos wallet contract-call \
  --chain 999 \
  --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
  --input-data 0x573ade81<asset_padded><amount_padded><0000...0002><onBehalfOf_padded>
```

**Pitfall — "repay all":** Do NOT pass `uint256.max` for amount. Use the wallet's actual token balance as the repay amount. Interest accrues within seconds; `uint256.max` causes the pool to pull `debtAmount` which may exceed wallet balance → revert. See `kb/protocols/lending.md §repay-all-pitfall`.

---

### 2.6 withdraw (Write)

No approval needed. Pool burns hTokens directly.

```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  withdraw(address asset, uint256 amount, address to)
Selector:  0x69328dec
Params:
  asset  — ERC-20 token address
  amount — raw token units, or uint256.max to withdraw all
  to     — recipient (usually user wallet)
```

onchainos call:
```bash
onchainos wallet contract-call \
  --chain 999 \
  --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
  --input-data 0x69328dec<asset_padded><amount_padded><to_padded>
```

**Pitfall — withdraw all with outstanding debt:** `uint256.max` withdraw reverts if user has any outstanding debt (HF would drop to 0). Always clear ALL debt before full withdrawal. See `kb/protocols/lending.md §withdraw-all-pitfall`.

---

### 2.7 set-collateral (Write)

No approval needed.

```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  setUserUseReserveAsCollateral(address asset, bool useAsCollateral)
Selector:  0x5a3b74b9
Params:
  asset           — ERC-20 token address
  useAsCollateral — true to enable, false to disable
```

**Note:** Reverts if user has no hToken balance for the asset (not yet supplied). Always supply first.

---

### 2.8 set-emode (Write)

No approval needed.

```
Contract: 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b  (Pool)
Function:  setUserEMode(uint8 categoryId)
Selector:  0x28530a47
Params:
  categoryId — 0 = no e-mode, 1+ = specific correlated asset category
               (e.g., 1 = stablecoins, 2 = HYPE staking derivatives)
```

---

## 3. Contract Addresses

### Core Pool Contracts

| Contract | Address |
|----------|---------|
| **Pool** (main entry point) | `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` |
| PoolImplementation | `0xc19d68383Ed7AB130c15cEad839e67A7Ed9d7041` |
| PoolConfigurator | `0x8CB4310dD38F6fD59388C9DE225f328092bdC379` |
| PoolConfigurator (impl) | `0xdc1f036389fc0Ad122D96893576C1C6434215eAB` |
| PoolAddressesProvider | `0x72c98246a98bFe64022a3190e7710E157497170C` |
| PoolAddressesProviderRegistry | `0x24E301BcBa5C098B3b41eA61a52bFe95Cb728b20` |

### Data & Oracle Contracts

| Contract | Address |
|----------|---------|
| ProtocolDataProvider | `0x5481bf8d3946E6A3168640c1D7523eB59F055a29` |
| UiPoolDataProvider | `0x3Bb92CF81E38484183cc96a4Fb8fBd2d73535807` |
| UiIncentiveDataProvider | `0xD47dc1F30994539B3fA000C70bB5E5D0bE203b54` |
| Oracle | `0xC9Fb4fbE842d57EAc1dF3e641a281827493A630e` |
| DefaultInterestRateStrategy | `0xD01E9AA0ba6a4a06E756BC8C79579E6cef070822` |

### Token Implementations

| Contract | Address |
|----------|---------|
| hToken (impl) | `0x7D4b11BC3f57C2BE2274e5C8Aa8e93a5315bbEee` |
| VariableDebtToken (impl) | `0x849140d62D1A298218EC974D2339BFC61fdf7D5C` |

### Incentives & Rewards

| Contract | Address |
|----------|---------|
| RewardsController | `0x2aF0d6754A58723c50b5e73E45D964bFDD99fE2F` |
| RewardsController (impl) | `0x484b0C602819d5A85bFFaC26E5B28c69F38c2941` |

### Key Token Addresses (HyperEVM Mainnet, chain 999)

| Token | Address | Decimals | Notes |
|-------|---------|----------|-------|
| USDC (native Circle) | `0xb88339CB7199b77E23DB6E890353E22632Ba630f` | 6 | Primary stablecoin |
| wHYPE (Wrapped HYPE) | `0x5555555555555555555555555555555555555555` | 18 | ERC-20 wrapper for native HYPE |
| HYPE (native) | `0x2222222222222222222222222222222222222222` | 18 | Native gas token (system address) |
| uBTC | `0x20000000000000000000000000000000000000c5` | 8 | Unit Bitcoin (system address) |
| uETH | `0x20000000000000000000000000000000000000dD` | 18 | Unit Ethereum (system address) |
| USDC system bridge | `0x2000000000000000000000000000000000000000` | 6 | HyperCore↔HyperEVM USDC bridge |

> **Note on token addresses:** uBTC/uETH are system-level spot asset addresses. The actual ERC-20 contract addresses used in DeFi (for Core Pool reserves) may differ — resolve at runtime via `ProtocolDataProvider.getAllReservesTokens()` or `GET /data/markets`. Always use checksummed addresses (API is case-sensitive).

---

## 4. User Scenarios

### Scenario 1: View Core Pool Market Data

**User intent:** "Show me the current lending rates on HyperLend"

**Execution flow:**
1. Call `GET https://api.hyperlend.finance/data/markets?chain=hyperEvm`
2. Parse response to extract per-reserve: symbol, supplyApy, borrowApy, totalSupplyUsd, totalBorrowUsd, utilizationRate, ltv, liquidationThreshold
3. Present as formatted table

**Example output:**
```
HyperLend Core Pools — Market Rates
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Asset     Supply APY   Borrow APY   Total Supply    Utilization
USDC       8.2%         10.5%        $42.3M           78%
wHYPE      3.1%          5.8%        $18.7M           52%
uBTC       1.9%          4.2%         $8.1M           38%
uETH       2.4%          5.1%        $12.5M           47%
```

**No onchainos needed** — pure REST API call.

---

### Scenario 2: Check User Position and Health Factor

**User intent:** "What's my current position on HyperLend?"

**Execution flow:**
1. Resolve user wallet address
2. Call `eth_call` on Pool with `getUserAccountData(wallet)`:
   ```bash
   onchainos rpc eth-call \
     --chain 999 \
     --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
     --data 0xbf92857c<wallet_padded>
   ```
3. Decode 6 × uint256 return values:
   - `totalCollateralBase` (USD, 8 decimals)
   - `totalDebtBase` (USD, 8 decimals)
   - `availableBorrowsBase` (USD, 8 decimals)
   - `currentLiquidationThreshold` (basis points)
   - `ltv` (basis points)
   - `healthFactor` (18 decimals; 1e18 = HF 1.0)
4. For per-asset breakdown, call `getUserReserveData(asset, wallet)` on ProtocolDataProvider for each active reserve
5. Format and display with health factor warning if HF < 1.5

**Example output:**
```
HyperLend Position
━━━━━━━━━━━━━━━━━━━━━━━━━━
Collateral:   $1,250.00 (wHYPE + USDC)
Debt:          $400.00  (USDC)
Available:     $200.00  to borrow
Health Factor: 2.34 ✓ (safe)
LTV:           32%      (max 80%)

Supplied:
  wHYPE:  500 wHYPE  ≈ $850
  USDC:   400 USDC   ≈ $400
Borrowed:
  USDC:   400 USDC   ≈ $400
```

---

### Scenario 3: Supply USDC to Core Pool

**User intent:** "I want to supply 1000 USDC to HyperLend"

**Execution flow:**
1. Check user USDC balance (must be ≥ 1000 USDC = 1,000,000,000 raw units at 6 decimals)
2. Check if USDC approval already covers Pool: if not, approve
3. Step 1 — Approve USDC for Pool:
   ```bash
   onchainos wallet contract-call \
     --chain 999 \
     --to 0xb88339CB7199b77E23DB6E890353E22632Ba630f \
     --input-data 0x095ea7b3\
       000000000000000000000000 00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
       000000000000000000000000000000000000000000000000000000003b9aca00
   ```
   (amount = 1,000,000,000 = 0x3B9ACA00 for 1000 USDC × 10^6)
4. Wait 3 seconds (nonce collision prevention — see kb/protocols/lending.md)
5. Step 2 — Pool.supply:
   ```bash
   onchainos wallet contract-call \
     --chain 999 \
     --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
     --input-data 0x617ba037\
       000000000000000000000000 b88339CB7199b77E23DB6E890353E22632Ba630f \
       000000000000000000000000000000000000000000000000000000003b9aca00 \
       000000000000000000000000 <USER_WALLET_ADDRESS> \
       0000000000000000000000000000000000000000000000000000000000000000
   ```
6. Confirm transaction; user receives hUSDC tokens

**Expected result:** User sees hUSDC balance increase in wallet; supply earns variable APY.

---

### Scenario 4: Borrow USDC Against wHYPE Collateral

**User intent:** "I want to borrow 200 USDC using my wHYPE as collateral"

**Execution flow:**
1. Check user's `getUserAccountData` — confirm `availableBorrowsBase` ≥ $200
2. Verify health factor after borrow would be > 1.3 (agent should warn if < 1.5)
3. No approval needed for borrow
4. Pool.borrow calldata:
   ```bash
   onchainos wallet contract-call \
     --chain 999 \
     --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
     --input-data 0xa415bcad\
       000000000000000000000000 b88339CB7199b77E23DB6E890353E22632Ba630f \
       000000000000000000000000000000000000000000000000000000000BEBC200 \
       0000000000000000000000000000000000000000000000000000000000000002 \
       0000000000000000000000000000000000000000000000000000000000000000 \
       000000000000000000000000 <USER_WALLET_ADDRESS>
   ```
   (amount = 200,000,000 = 0x0BEBC200 for 200 USDC at 6 decimals; interestRateMode = 2)
5. Confirm transaction; user receives 200 USDC in wallet
6. VariableDebtToken minted automatically, interest begins accruing

**Risk warning:** Agent MUST inform user of liquidation risk if HYPE price drops. Recommend maintaining HF > 1.5.

---

### Scenario 5: Repay USDC Debt and Withdraw Collateral

**User intent:** "Repay my USDC debt and then withdraw my wHYPE"

**Execution flow:**

**Part A — Repay:**
1. Query current USDC debt via `getUserReserveData(USDC, wallet).currentVariableDebt`
2. Check wallet USDC balance — ensure balance ≥ debt (if not, acquire more USDC first)
3. Use wallet balance as repay amount (NOT uint256.max — avoids accrual revert)
4. Approve USDC for Pool (repay amount)
5. Wait 3 seconds
6. Pool.repay:
   ```bash
   onchainos wallet contract-call \
     --chain 999 \
     --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
     --input-data 0x573ade81\
       000000000000000000000000 b88339CB7199b77E23DB6E890353E22632Ba630f \
       <wallet_usdc_balance_padded> \
       0000000000000000000000000000000000000000000000000000000000000002 \
       000000000000000000000000 <USER_WALLET_ADDRESS>
   ```

**Part B — Withdraw (after repay confirmed):**
1. Verify zero debt: `getUserAccountData.totalDebtBase == 0`
2. Pool.withdraw with `uint256.max` to withdraw all wHYPE:
   ```bash
   onchainos wallet contract-call \
     --chain 999 \
     --to 0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b \
     --input-data 0x69328dec\
       000000000000000000000000 5555555555555555555555555555555555555555 \
       ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff \
       000000000000000000000000 <USER_WALLET_ADDRESS>
   ```
3. hWHYPE tokens burned; user receives wHYPE back

---

## 5. API Dependencies

### REST API

**Base URL:** `https://api.hyperlend.finance`

**Authentication:** No API key required for public endpoints. Rate limits apply (not documented officially).

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/data/markets` | GET | All Core Pool reserves: APY, TVL, utilization, risk params. Accepts `?chain=hyperEvm` |
| `/data/markets/rates` | GET | Borrow/supply APY for all assets and isolated markets |
| `/data/interestRateHistory` | GET | Hourly rate history (values in Ray units = 27 decimals) |
| `/data/transactions` | GET | Event data query by chain, contract, event type |

**Query params for `/data/markets`:**
- `chain=hyperEvm` — filter to Core Pools on HyperEVM
- `user=<address>` — include user-specific data (supply balance, borrow balance)

### Ponder Indexer

**Base URL:** `https://ponder.hyperlend.finance`

Ponder automatically generates a GraphQL API. The schema covers supply/borrow/repay/withdraw events, user positions, and protocol statistics. Useful for historical position data.

GraphQL endpoint: `https://ponder.hyperlend.finance/graphql` (assumed standard Ponder path)

### HyperLend RPC (Dedicated)

For eth_call operations, use the HyperLend-hosted RPC which has higher rate limits:

| RPC | URL | Notes |
|-----|-----|-------|
| Standard | `https://rpc.hyperlend.finance` | 30 req/sec per IP; max 50-block range for eth_getLogs |
| Archive | `https://rpc.hyperlend.finance/archive` | Full historical data (Nanoreth); for deep history queries |
| Public HyperEVM | `https://rpc.hyperliquid.xyz/evm` | 100 req/min per IP; official fallback |

### On-Chain Read Calls (eth_call)

| Contract | Function | Selector | Description |
|----------|----------|----------|-------------|
| Pool | `getUserAccountData(address)` | `0xbf92857c` | Account health factor and aggregate USD values |
| Pool | `getReserveData(address)` | `0x35ea6a75` | Reserve configuration and state |
| Pool | `getReservesList()` | `0xd1946dbc` | Array of all active reserve asset addresses |
| Pool | `getUserEMode(address)` | (standard Aave V3) | User's current e-mode category |
| ProtocolDataProvider | `getUserReserveData(address,address)` | `0x28dd0f6e` | Per-asset supply/borrow balance for user |
| ProtocolDataProvider | `getAllReservesTokens()` | (standard) | All reserve (symbol, address) pairs |
| ProtocolDataProvider | `getReserveConfigurationData(address)` | (standard) | LTV, liquidation threshold, etc. |
| PoolAddressesProvider | `getPool()` | `0x026b1d5f` | Resolve canonical Pool address at runtime |

---

## 6. Configuration Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `chain_id` | `999` | HyperEVM mainnet chain ID |
| `rpc_url` | `https://rpc.hyperlend.finance` | EVM JSON-RPC endpoint (higher limits than public) |
| `rpc_url_fallback` | `https://rpc.hyperliquid.xyz/evm` | Public HyperEVM RPC fallback |
| `pool_address` | `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` | HyperLend Core Pool (main entry point) |
| `pool_addresses_provider` | `0x72c98246a98bFe64022a3190e7710E157497170C` | PoolAddressesProvider (resolve pool at runtime) |
| `protocol_data_provider` | `0x5481bf8d3946E6A3168640c1D7523eB59F055a29` | For per-reserve and per-user data reads |
| `api_base_url` | `https://api.hyperlend.finance` | REST API base URL |
| `usdc_address` | `0xb88339CB7199b77E23DB6E890353E22632Ba630f` | USDC on HyperEVM mainnet (Circle native) |
| `whype_address` | `0x5555555555555555555555555555555555555555` | Wrapped HYPE ERC-20 contract |
| `health_factor_warning` | `1.5` | Warn user when HF drops below this threshold |
| `health_factor_min` | `1.1` | Refuse borrow if post-borrow HF would be below this |
| `dry_run` | `true` | Simulate transactions without broadcasting (onchainos --dry-run) |
| `approve_delay_secs` | `3` | Delay between approve and supply/repay to prevent nonce collision |
| `interest_rate_mode` | `2` | Always 2 (variable); stable rate deprecated in Aave V3.2 |

---

## 7. Technical Notes

### Aave V3.2 Fork — Key Behaviors

1. **Interest rate mode is always 2 (variable).** Stable rate (mode 1) was removed in Aave V3.2. Always pass `interestRateMode = 2` for `borrow` and `repay`.

2. **hTokens instead of aTokens.** HyperLend uses `hToken` (hUSDC, hwHYPE, etc.) instead of Aave's `aToken`. Functionally identical — yield-bearing ERC-20 representing a pool supply share.

3. **ERC-20 approval required before supply and repay.** Both ops pull tokens from user wallet via `transferFrom`. Add 3-second delay between approve and main call to prevent nonce/mempool issues.

4. **No approval needed for borrow and withdraw.** Pool mints/burns debt tokens and hTokens directly.

5. **set-collateral must come after supply.** `setUserUseReserveAsCollateral(asset, true)` reverts if user has zero hToken balance for that asset.

6. **Recommended test order:**
   ```
   get-markets → set-emode → supply → set-collateral → borrow → repay → withdraw
   ```

### Native HYPE vs wHYPE

- HYPE is the native gas token of HyperEVM (analogous to ETH on Ethereum).
- To supply HYPE to the Core Pool, users must first wrap it to wHYPE: call `deposit()` (selector `0xd0e30db0`) on `0x5555555555555555555555555555555555555555` with `msg.value = hype_amount`.
- wHYPE is then a standard ERC-20 that can be approved and supplied.

### repay-all Pitfall

Do NOT use `uint256.max` as the repay amount. Interest accrues per-second; at repay time, debt may be `1 wei` larger than wallet balance → `transferFrom` reverts. Use `walletBalance` as amount instead. Dust interest (sub-cent) remains as outstanding debt. See `kb/protocols/lending.md §repay-all-pitfall`.

### withdraw-all Pitfall

`uint256.max` withdraw with any outstanding debt reverts (HF would become 0). Always clear all debt before full withdrawal. See `kb/protocols/lending.md §withdraw-all-pitfall`.

### Health Factor Monitoring

```
HF = (Sum of collateral USD × liquidation threshold) / (Sum of debt USD)
HF returned by getUserAccountData: uint256 with 1e18 precision
HF < 1.0 → liquidation
```

Agents should refuse to execute borrows that would result in HF < 1.1, and warn at HF < 1.5.

### Pool Address Resolution

The Pool address should ideally be resolved at runtime from `PoolAddressesProvider.getPool()` (selector `0x026b1d5f`) in case of a proxy upgrade. The hardcoded address `0x00A89d7a5A02160f20150EbEA7a2b5E4879A1A8b` is the current mainnet deployment as of April 2026.

### Calldata ABI Encoding Reference

All Aave V3 Pool functions use standard ABI encoding (no tuples in core ops):

```
supply(address asset, uint256 amount, address onBehalfOf, uint16 referralCode):
  0x617ba037
  + 000000000000000000000000<asset_addr_hex>      (32 bytes, address left-padded)
  + <amount_uint256_hex>                           (32 bytes, big-endian)
  + 000000000000000000000000<onBehalfOf_hex>       (32 bytes, address left-padded)
  + 0000000000000000000000000000000000000000000000000000000000000000  (uint16 referral = 0)

borrow(address asset, uint256 amount, uint256 interestRateMode, uint16 referralCode, address onBehalfOf):
  0xa415bcad
  + 000000000000000000000000<asset_hex>
  + <amount_hex>
  + 0000000000000000000000000000000000000000000000000000000000000002  (mode = 2)
  + 0000000000000000000000000000000000000000000000000000000000000000  (referral = 0)
  + 000000000000000000000000<onBehalfOf_hex>

repay(address asset, uint256 amount, uint256 interestRateMode, address onBehalfOf):
  0x573ade81
  + 000000000000000000000000<asset_hex>
  + <amount_hex>
  + 0000000000000000000000000000000000000000000000000000000000000002
  + 000000000000000000000000<onBehalfOf_hex>

withdraw(address asset, uint256 amount, address to):
  0x69328dec
  + 000000000000000000000000<asset_hex>
  + <amount_hex>  (or ffffffff...ffff for uint256.max = withdraw all)
  + 000000000000000000000000<to_hex>
```

### Function Selectors (Verified with Foundry cast)

| Function Signature | Selector | Source |
|-------------------|----------|--------|
| `supply(address,uint256,address,uint16)` | `0x617ba037` | cast sig — verified |
| `borrow(address,uint256,uint256,uint16,address)` | `0xa415bcad` | cast sig — verified |
| `repay(address,uint256,uint256,address)` | `0x573ade81` | cast sig — verified |
| `withdraw(address,uint256,address)` | `0x69328dec` | cast sig — verified |
| `setUserUseReserveAsCollateral(address,bool)` | `0x5a3b74b9` | cast sig — verified |
| `setUserEMode(uint8)` | `0x28530a47` | cast sig — verified |
| `getUserAccountData(address)` | `0xbf92857c` | cast sig — verified |
| `getReserveData(address)` | `0x35ea6a75` | cast sig — verified |
| `getReservesList()` | `0xd1946dbc` | cast sig — verified |
| `getPool()` *(PoolAddressesProvider)* | `0x026b1d5f` | cast sig — verified |
| `approve(address,uint256)` | `0x095ea7b3` | standard ERC-20 |
| `deposit()` *(wHYPE wrap)* | `0xd0e30db0` | standard WETH9 |

---

## 8. References

- Official docs: https://docs.hyperlend.finance
- Contract addresses: https://docs.hyperlend.finance/developer-documentation/contract-addresses
- Core Pools docs: https://docs.hyperlend.finance/developer-documentation/core-pools
- GitHub (Core): https://github.com/hyperlendx/hyperlend-core
- Audits: https://github.com/hyperlendx/audits/tree/master/hyperlend
- DeFiLlama: https://defillama.com/protocol/hyperlend-pooled
- ChainList (chain 999): https://chainlist.org/chain/999
- HyperEVM block explorer: https://hyperevmscan.io
- Aave V3 pool patterns: `kb/protocols/lending.md`

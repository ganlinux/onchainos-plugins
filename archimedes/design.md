# Archimedes Finance — Plugin Design

> **Record ID**: `recvfIWnsWu56i`
> **Status**: Research complete — see §1 for important activity/deprecation notes
> **Last Updated**: 2026-04-05

---

## §0 Plugin Meta

| Field | Value |
|-------|-------|
| **plugin_name** | `archimedes` |
| **dapp_name** | Archimedes Finance |
| **route** | B (DApp接入插件 — Rust + onchainos) |
| **one_liner** | Deposit into Archimedes V2 protected yield vaults (ERC4626) and query positions on Ethereum mainnet |
| **category** | `defi-protocol` |
| **tags** | `yield, vault, erc4626, convex, aura, protected-pools, ethereum` |
| **target_chains** | `ethereum` (chain ID 1) |
| **target_protocols** | Archimedes Finance V2 (Protected Single Pools — MultiPoolStrategy) |

---

## §1 接入可行性调研

> **⚠️ CRITICAL STATUS NOTE — READ FIRST**
>
> Archimedes Finance has two distinct product generations:
>
> - **V1 (Leveraged Yield / lvUSD)**: Deployed Feb–Jul 2023 on Ethereum mainnet. **INACTIVE** — last transaction on LeverageEngine (`0x03dc7Fa99...`) was August 2023 (~2.5 years ago). Only ~50 total transactions ever. The protocol raised $4.9M seed (Hack VC, Feb 2023), launched the Curve lvUSD/3CRV pool, and processed a small number of leveraged OUSD positions before going dormant. V1 contracts should NOT be the primary implementation target.
>
> - **V2 (Protected Single Pools — MultiPoolStrategy)**: Launched June 2024 (beta). Uses ERC4626 vaults that deposit into Convex/Aura strategies. Factory deployed July 2023; individual strategies (e.g., `0xfA364CB...` WETH strategy, `0x2E04e0a...` alUSD strategy) had activity into September 2023. The WETH strategy (`0x83FeD51...`) had activity as recently as August 2025.
>
> **Recommendation**: Target V2 MultiPoolStrategy (ERC4626) for the plugin implementation. Focus on the WETH and USDC strategy vaults. Treat this as a **low-activity, beta protocol** — implement defensively with read-first and graceful handling of empty states.

### Feasibility Table

| Check | Result |
|-------|--------|
| Has Rust SDK? | **No** — No Rust SDK or any language SDK exists. Pure direct contract interaction via onchainos. |
| SDK tech stack? | N/A — Direct contract calls only. ERC4626 standard interface is well-defined and stable. |
| Has REST API? | **No** — No public REST API. All data via Ethereum RPC (eth_call). DefiLlama has a protocol page for TVL reference. |
| Has official Skill? | **No** |
| Community Skill? | **No** — No known open-source community skill implementations found. |
| Supported chains | **Ethereum mainnet only** (chain ID 1). All contracts deployed on Ethereum L1. |
| Needs onchainos broadcast? | **Yes** — All write operations (deposit, withdraw, redeem, approve) must go through `onchainos wallet contract-call`. |
| Protocol activity | **Low / Beta** — V2 vaults have minimal TVL ($50-$100 range per vault). V1 fully inactive. Treat as educational/niche implementation. |

### Access Path

**Route: Direct Contract Calls via onchainos** (no SDK, no REST API)

All operations use:
- Read: `onchainos wallet eth-call` (or `rpc::eth_call`) against the MultiPoolStrategy ERC4626 vault
- Write: `onchainos wallet contract-call` with calldata constructed from verified function selectors
- ERC-20 approve: `onchainos wallet contract-call` on the underlying token before deposit

---

## §2 Interface Mapping

### 2a. Operations Overview

| # | Operation | Type | Priority | Description |
|---|-----------|------|----------|-------------|
| 1 | `positions` | Read (chain) | P0 | Query user's vault share balance, underlying asset value, vault TVL |
| 2 | `deposit` | Write (chain) | P0 | Deposit underlying asset (WETH/USDC) into a MultiPoolStrategy vault |
| 3 | `withdraw` | Write (chain) | P0 | Withdraw underlying assets by specifying asset amount |
| 4 | `redeem` | Write (chain) | P1 | Redeem by specifying shares amount |
| 5 | `vaults` | Read (chain) | P1 | List known strategy vault addresses and their underlying assets |

---

### 2b. Read Operations

#### `positions` — Query User Position in a Vault

| Field | Value |
|-------|-------|
| Contract | MultiPoolStrategy vault (ERC4626, e.g. `0xfA364CBca915f17fEc356E35B61541fC6D4D8269`) |
| Method | `balanceOf(address)` → shares; then `convertToAssets(uint256)` → underlying value |
| Selector | `balanceOf`: `0x70a08231` ✅; `convertToAssets`: `0x07a2d13a` ✅; `totalAssets`: `0x01e1d114` ✅ |
| Params | `wallet_address` (address) |
| Returns | shares (uint256), underlying asset amount (uint256), total vault assets (uint256) |

Encoding pattern for `balanceOf(address wallet)`:
```
calldata = "0x70a08231" + left-pad(wallet_addr[2..], 64 chars)
```

Encoding pattern for `convertToAssets(uint256 shares)`:
```
calldata = "0x07a2d13a" + left-pad(shares.to_hex(), 64 chars)
```

#### `totalAssets` — Vault TVL

```
calldata = "0x01e1d114"   (no params)
```
Returns uint256 — total underlying tokens managed by vault.

---

### 2c. Write Operations (EVM — strict format)

| Operation | Contract Address | Function Signature (canonical ABI) | Selector (cast sig ✅) | ABI Parameter Order |
|-----------|-----------------|-------------------------------------|----------------------|---------------------|
| approve WETH (pre-deposit) | WETH `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | `approve(address,uint256)` | `0x095ea7b3` ✅ | spender(=vault), amount |
| deposit | MultiPoolStrategy vault | `deposit(uint256,address)` | `0x6e553f65` ✅ | assets, receiver(=wallet) |
| withdraw | MultiPoolStrategy vault | `withdraw(uint256,address,address,uint256)` | `0xa318c1a4` ✅ | assets, receiver(=wallet), owner(=wallet), minimumReceive |
| redeem | MultiPoolStrategy vault | `redeem(uint256,address,address,uint256)` | `0x9f40a7b3` ✅ | shares, receiver(=wallet), owner(=wallet), minimumReceive |

> **Note on `withdraw` / `redeem` extra parameter**: The Archimedes MultiPoolStrategy adds a fourth parameter `minimumReceive` (uint256) to ERC4626's standard `withdraw`/`redeem`. This is a slippage guard. Set to `0` for no minimum (user-configurable), or compute as `expectedAmount * (1 - slippage)`.

> **Note on `deposit`**: The standard ERC4626 `deposit(uint256 assets, address receiver)` selector `0x6e553f65` is used here. This is verified. The vault auto-routes the deposit into underlying Convex/Aura adapters via the offchain monitor's `adjustIn()` call — no user action required after deposit.

#### ERC-20 Approve selector verification:
```bash
cast sig "approve(address,uint256)"  # → 0x095ea7b3
```

#### Known Vault Addresses (Ethereum Mainnet)

| Strategy | Vault Address | Underlying Asset | Underlying Address | Status |
|----------|--------------|------------------|--------------------|--------|
| WETH — ETH+ (Convex) | `0xfA364CBca915f17fEc356E35B61541fC6D4D8269` | WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | Active (last tx Aug 2025) |
| WETH — Aura Weighted | `0x83FeD5139eD14162198Bd0a54637c22cA854E2f6` | WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | Low activity (last tx Sep 2023) |
| alUSD — FRAXBP (Convex) | `0x2E04e0aEa173F95A23043576138539fBa60D930a` | crvFRAX | `0x3175Df0976dFA876431C2E9eE6Bc45b65d3473CC` | Low activity (last tx Sep 2023) |

**Factory (for dynamic vault discovery)**:
`0x0f152c86FdaDD9B58F7DCE26D819D76a70AD348F` — MultiPoolStrategyFactory (last tx Jul 2023, inactive — do not rely on for address discovery)

**Adapter contracts** (internal, not user-facing):
- ETH+ adapter: `0xaa04430d364458a7fc98643585ea2e45a4955acd`
- Aura WETH adapter: `0xbBc69AF6bE2edea71509329af928817cdFA7aBf1`
- FRAXBP adapter: `0x14E1183DBc21dd5072b65e280bBb57054a47F5b0`

#### V1 Legacy Contracts (Reference Only — DO NOT IMPLEMENT)

| Contract | Address | Status |
|----------|---------|--------|
| LeverageEngine (proxy) | `0x03dc7Fa99B986B7E6bFA195f39085425d8172E29` | INACTIVE (last tx Aug 2023) |
| LeverageEngine (impl) | `0x7fb10F39135e67baf6c7fbc9fc484c4744d8d611` | INACTIVE |
| Coordinator (proxy) | `0x58c968fADa478adb995b59Ba9e46e3Db4d6B579d` | INACTIVE |
| lvUSD Token | `0x94A18d9FE00bab617fAD8B49b11e9F1f64Db6b36` | INACTIVE |
| ARCH Token | `0x73C69d24ad28e2d43D03CBf35F79fE26EBDE1011` | Low liquidity ($3/day volume) |
| Curve lvUSD/3CRV Pool | `0xe9123CBC5d1EA65301D417193c40A72Ac8D53501` | INACTIVE |
| VaultOUSD | `0x4c12c57C37Ff008450A2597e810B51B2BbA0383A` | INACTIVE (~63 OUSD remaining) |

---

### 2d. onchainos Command Mapping

| Operation | onchainos Command | Key Flags |
|-----------|------------------|-----------|
| Read balanceOf | `onchainos wallet eth-call` | `--chain 1 --to <vault> --data <calldata>` |
| Read totalAssets | `onchainos wallet eth-call` | `--chain 1 --to <vault> --data 0x01e1d114` |
| Approve token | `onchainos wallet contract-call` | `--chain 1 --to <token> --input-data <approve_calldata>` |
| Deposit | `onchainos wallet contract-call` | `--chain 1 --to <vault> --input-data <deposit_calldata>` |
| Withdraw | `onchainos wallet contract-call` | `--chain 1 --to <vault> --input-data <withdraw_calldata>` |
| Redeem | `onchainos wallet contract-call` | `--chain 1 --to <vault> --input-data <redeem_calldata>` |

> All write operations require a 3-second delay between ERC-20 approve and the vault call to avoid nonce collision. See lending.md §ERC-20 Approval for Supply and Repay.

---

## §3 User Scenarios

### Scenario 1: Check My Archimedes Position (Happy Path — Read)

**User says**: "Show me my Archimedes Finance positions"

**Agent action sequence**:
1. (Chain read) For each known vault address, call `balanceOf(wallet)` via `onchainos wallet eth-call --chain 1 --to <vault> --data 0x70a08231<padded_wallet>`
2. For vaults with non-zero shares, call `convertToAssets(shares)` to get underlying value
3. Call `totalAssets()` on each active vault to show TVL
4. (Optional) Fetch WETH/USD price from DefiLlama or CoinGecko to show USD value
5. Return summary: vault name, shares held, underlying asset amount, estimated USD value, vault TVL

**Expected output**:
```
Archimedes Finance Positions (Ethereum):
- WETH ETH+ Strategy (0xfA364C...): 0.015 shares → 0.0148 WETH (~$43.20)
  Vault TVL: 0.0247 WETH total
- WETH Aura Strategy: 0 shares (no position)
```

---

### Scenario 2: Deposit WETH into Protected Pool (Happy Path — Write)

**User says**: "Deposit 0.01 WETH into Archimedes WETH strategy"

**Agent action sequence**:
1. (Chain read) Check WETH balance: `onchainos wallet eth-call --chain 1 --to 0xC02aaA... --data 0x70a08231<wallet>`
2. (Validation) Ensure balance >= 0.01 WETH (= 10000000000000000 wei). If not, error out.
3. (Chain read) Preview shares to be received: call `previewDeposit(assets)` on vault — selector `0xef8b30f7` ✅
4. (Chain write) Approve vault to spend WETH:
   ```
   onchainos wallet contract-call --chain 1 \
     --to 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2 \
     --input-data 0x095ea7b3<padded_vault_addr><padded_amount>
   ```
5. (Wait 3 seconds)
6. (Chain write) Deposit into vault:
   ```
   onchainos wallet contract-call --chain 1 \
     --to 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 \
     --input-data 0x6e553f65<padded_amount><padded_wallet>
   ```
7. Return: txHash, shares received, vault address

**Notes**:
- `deposit(uint256 assets, address receiver)` — assets is in wei (18 decimals for WETH)
- receiver = user's wallet address
- Funds sit idle in vault until offchain monitor triggers `adjustIn()` — this is expected behavior

---

### Scenario 3: Withdraw from Protected Pool (Write with Slippage Guard)

**User says**: "Withdraw 0.01 WETH from my Archimedes WETH position"

**Agent action sequence**:
1. (Chain read) Check current shares balance in vault: `balanceOf(wallet)` → N shares
2. (Chain read) Call `previewWithdraw(0.01 WETH in wei)` — selector `0x0a28a477` ✅ — to check if withdrawal is feasible
3. (Validation) If vault has insufficient liquidity (previewWithdraw fails or returns 0), inform user that vault may be rebalancing
4. Compute `minimumReceive` = `0.01 WETH * (1 - slippage)` where slippage default = 0.5%
   - Example: 0.01 WETH = 10000000000000000 wei; minimumReceive = 9950000000000000 wei
5. (Chain write) Call withdraw:
   ```
   onchainos wallet contract-call --chain 1 \
     --to 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 \
     --input-data 0xa318c1a4<assets><receiver><owner><minimumReceive>
   ```
   where: assets = 10000000000000000, receiver = wallet, owner = wallet, minimumReceive = 9950000000000000
6. Return: txHash, assets received, shares burned

**Edge case**: If `minimumReceive` slippage check reverts, reduce amount or set minimumReceive = 0 (user override).

---

### Scenario 4: Redeem All Shares (Write — Full Exit)

**User says**: "Exit my entire Archimedes WETH position"

**Agent action sequence**:
1. (Chain read) Get total shares: `balanceOf(wallet)` → total_shares
2. (Chain read) Get expected assets: `convertToAssets(total_shares)` → expected_assets
3. Compute minimumReceive = `expected_assets * (1 - 0.005)` (0.5% slippage tolerance)
4. (Chain write) Call redeem:
   ```
   onchainos wallet contract-call --chain 1 \
     --to <vault_address> \
     --input-data 0x9f40a7b3<total_shares><receiver><owner><minimumReceive>
   ```
5. Return: txHash, assets received

---

### Scenario 5: List Available Vaults (Read — Discovery)

**User says**: "What Archimedes vaults can I deposit into?"

**Agent action sequence**:
1. (Static lookup) Read hardcoded vault list from config (factory is inactive, so dynamic discovery is not viable)
2. For each vault, call `totalAssets()` to show current TVL
3. For each vault, call `symbol()` (ERC20: `0x95d89b41` ✅) and `asset()` (`0x38d52e0f` ✅) to get display info
4. Return formatted vault list with underlying asset, current TVL

---

## §4 External API Dependencies

| API | Endpoint | Purpose | Rate Limit | Free? | Notes |
|-----|----------|---------|------------|-------|-------|
| Ethereum RPC | `https://ethereum.publicnode.com` | All eth_call reads | ~100 req/sec | Yes | Use publicnode for mainnet (avoid llamarpc rate limits — see lending.md §Compound V3 Ethereum RPC) |
| DefiLlama TVL | `https://api.llama.fi/protocol/archimedes-finance` | Protocol TVL overview (optional, display only) | 300/5min | Yes | Low priority — useful for context |
| CoinGecko (optional) | `https://api.coingecko.com/api/v3/simple/price?ids=weth&vs_currencies=usd` | WETH/USD price for USD display | 30/min | Yes | Only needed for USD value formatting |

> **No subgraph**: Archimedes does not have a TheGraph subgraph. All position data must be fetched directly from contracts.

---

## §5 Configuration Parameters

| Parameter | Type | Default | Range | Description |
|-----------|------|---------|-------|-------------|
| `chain_id` | u64 | `1` | fixed | Ethereum mainnet |
| `rpc_url` | String | `https://ethereum.publicnode.com` | — | Ethereum RPC endpoint |
| `vault_weth_eth_plus` | String | `0xfA364CBca915f17fEc356E35B61541fC6D4D8269` | — | WETH ETH+ strategy vault |
| `vault_weth_aura` | String | `0x83FeD5139eD14162198Bd0a54637c22cA854E2f6` | — | WETH Aura strategy vault |
| `vault_alusd_fraxbp` | String | `0x2E04e0aEa173F95A23043576138539fBa60D930a` | — | alUSD FRAXBP strategy vault |
| `weth_address` | String | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | — | WETH token on Ethereum |
| `max_slippage_bps` | u64 | `50` | 1–500 | Max slippage in basis points (50 = 0.5%) for withdraw/redeem minimumReceive |
| `approve_delay_secs` | u64 | `3` | 1–10 | Delay between approve and deposit/withdraw tx to avoid nonce collision |
| `dry_run` | bool | `false` | — | If true, simulate all writes without broadcasting |

---

## §6 Protocol Mechanics (Background)

### V2 Protected Single Pools Architecture

```
User Wallet
    │
    ├─ approve(vault, amount) → WETH token
    └─ deposit(assets, receiver) → MultiPoolStrategy vault (ERC4626)
                                          │
                                          ├─ Idle funds sit in vault
                                          │
                                          └─ Offchain Monitor (block-by-block)
                                                │
                                                ├─ adjust() → ConvexAdapter / AuraAdapter
                                                │              └─ Deposits LP into Convex/Aura
                                                │
                                                ├─ doHardWork() → claim rewards → swap → redeposit
                                                │
                                                └─ Health checks: pool balance, TVL > $1M,
                                                   Archimedes control < 33%, slippage < 1-2%
                                                   If unhealthy → auto-withdraw to vault
```

### Key Design Properties

1. **ERC4626 standard**: Deposit assets → receive shares. Shares appreciate as vault earns yield. Standard `deposit`, `withdraw`, `redeem` flow.
2. **Offchain monitor**: Funds are deployed into Convex/Aura by an offchain bot, not automatically on deposit. Idle funds in vault earn no yield until `adjust()` is triggered.
3. **Auto-protection**: If underlying pool depegs or TVL drops, monitor withdraws funds back to vault automatically, protecting principal.
4. **Non-standard withdraw params**: `withdraw(assets, receiver, owner, minimumReceive)` and `redeem(shares, receiver, owner, minimumReceive)` have an extra 4th param vs standard ERC4626.
5. **Gas efficiency**: Rewards are compounded by the protocol (gas covered by protocol fees). Users pay gas only for deposit/withdraw.

### V1 Leveraged Yield (Archived — Do Not Implement)

V1 used a leverage engine where users deposited OUSD as collateral, bought leverage allocation using ARCH tokens, and received an NFT representing a leveraged position. Positions were unwound via `unwindLeveragedPosition(positionId, minArchAmount)`. The protocol is fully dormant since August 2023.

---

## §7 Risk Flags & Implementation Notes

1. **Very low TVL**: As of research (Apr 2026), individual vaults hold $50–$100 equivalent. Test with tiny amounts (0.001 WETH).
2. **Beta protocol**: Contracts were labeled "beta" and "unaudited" at launch. Halborn audited V1 only.
3. **Factory is inactive**: Cannot use factory for dynamic vault discovery. Hardcode vault addresses in config.
4. **Non-standard ERC4626**: The 4th `minimumReceive` parameter on `withdraw`/`redeem` breaks standard ERC4626 compatibility. Must use the custom selectors above.
5. **Offchain monitor dependency**: Yield only accrues after offchain bot calls `adjust()`. Users may deposit and see no yield if monitor is not running.
6. **Ethereum RPC**: Use `https://ethereum.publicnode.com` — not `eth.llamarpc.com` (rate-limited). See lending.md.
7. **WETH only for active vault**: The `0xfA364CB` vault is the most recently active (Aug 2025). The other two vaults are dormant. Focus testing on `0xfA364CB`.

---

## §8 Test Scenario Checklist

| # | Scenario | Precondition | Key Steps | Expected Result | Priority |
|---|----------|--------------|-----------|----------------|----------|
| 1 | positions (no deposit) | Fresh wallet | Call balanceOf on all vaults | Returns 0 for all, no error | P0 |
| 2 | positions (after deposit) | Wallet has shares | Call balanceOf + convertToAssets | Returns shares and asset amount | P0 |
| 3 | deposit 0.001 WETH | Wallet has >= 0.001 WETH | approve → deposit | TxHash, shares received > 0 | P0 |
| 4 | withdraw partial | Wallet has vault shares | withdraw(partial_amount) | TxHash, assets returned | P0 |
| 5 | redeem all shares | Wallet has vault shares | balanceOf → redeem(all) | TxHash, all assets returned, shares → 0 | P0 |
| 6 | dry-run deposit | Any state | deposit with dry_run=true | Simulation result, no tx | P0 |
| 7 | insufficient balance | WETH balance < requested | deposit → validation | Error: insufficient balance | P1 |
| 8 | list vaults | Any state | vaults command | All 3 vaults with TVL | P1 |

---

## §9 Open Questions

| # | Question | Blocking? | Status |
|---|----------|-----------|--------|
| 1 | Is the offchain monitor still running for `0xfA364CB` vault? (Last tx Aug 2025 suggests yes, but uncertain) | No — deposit/withdraw still works regardless | Open |
| 2 | Does the `0xfA364CB` vault accept WETH directly or requires ETH wrapping first? | No — vault `asset()` call will confirm underlying token | Resolve in Phase 3 testing |
| 3 | Are there any new vaults deployed by the factory that are not in the hardcoded list? | No — factory is inactive, no new vaults | Likely closed |
| 4 | Does the protocol have plans for V3 or new vault deployments? | No — implement with hardcoded vaults is fine | Open |

---

## Appendix A: Selector Reference (all verified with `cast sig`)

| Function | Selector | Verified |
|----------|----------|---------|
| `balanceOf(address)` | `0x70a08231` | ✅ |
| `totalAssets()` | `0x01e1d114` | ✅ |
| `totalSupply()` | `0x18160ddd` | ✅ |
| `previewDeposit(uint256)` | `0xef8b30f7` | ✅ |
| `previewWithdraw(uint256)` | `0x0a28a477` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | ✅ |
| `convertToShares(uint256)` | `0xc6e6f592` | ✅ |
| `deposit(uint256,address)` | `0x6e553f65` | ✅ |
| `withdraw(uint256,address,address,uint256)` | `0xa318c1a4` | ✅ |
| `redeem(uint256,address,address,uint256)` | `0x9f40a7b3` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | ✅ |
| `symbol()` | `0x95d89b41` | ✅ |
| `asset()` (ERC4626) | `0x38d52e0f` | ✅ |

---

## Appendix B: V1 Contract Reference (Archived)

For historical reference only. Do not implement.

| Contract | Address | Key Functions |
|----------|---------|---------------|
| LeverageEngine | `0x03dc7Fa99B986B7E6bFA195f39085425d8172E29` | `createLeveragedPosition(uint256,uint256,uint256)` → `0x7862a8ba`, `unwindLeveragedPosition(uint256,uint256)` → `0xdafccdd9` |
| Coordinator | `0x58c968fADa478adb995b59Ba9e46e3Db4d6B579d` | `acceptLeverageAmount`, `depositCollateralUnderNFT`, `borrowUnderNFT`, `repayUnderNFT`, `getLeveragedOUSD`, `unwindLeveragedOUSD` |
| ARCH Token | `0x73C69d24ad28e2d43D03CBf35F79fE26EBDE1011` | ERC-20, required as fee for V1 leverage |
| Curve lvUSD/3CRV Pool | `0xe9123CBC5d1EA65301D417193c40A72Ac8D53501` | Standard Curve pool interface |

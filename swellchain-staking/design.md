# Swellchain Staking — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `swellchain-staking` |
| dapp_name | Swellchain Staking |
| category | liquid-staking |
| tags | staking, restaking, liquid-staking, ethereum, swellchain, l2, swETH, rswETH, earnETH |
| target_chains | ethereum (chain_id=1), swellchain (chain_id=1923) |
| target_protocols | Swell Network (swETH LST, rswETH LRT, earnETH Vault) |
| priority | P1 (#148) |
| onchainos_broadcast | 是 |

---

## 1. Background

### DApp 介绍

Swellchain Staking 是 Swell Network 的 L2（Swellchain，chain_id=1923）原生质押生态，分三层产品：

1. **swETH（液态质押，LST）**：用户在以太坊 L1 存入 ETH，铸造 swETH（repricing token，随时间自动增值，当前 APR ~3%）。合约自动将 ETH 部署到 Beacon Chain 验证节点。
2. **rswETH（液态再质押，LRT）**：存入 ETH（或已有 swETH），在 EigenLayer 进行再质押，铸造 rswETH（当前 APR ~2.63%）。rswETH 是 Swellchain L2 的原生 Gas 代币。
3. **earnETH Vault（收益聚合器）**：基于 Nucleus/BoringVault 架构，用户存入 swETH/rswETH/stETH/weETH/mETH 等 LST/LRT，自动分配到 DeFi 协议和 Swell L2，赚取多层收益（当前 APR ~1%+ 积分）。

**重要区别**（避免与已接入 PR 重复）：
- PR #141 Swell ETH Staking：接入了 **swETH** 的基础 stake/unstake；
- PR #179 Swell Restaking：接入了 **rswETH** 的 EigenLayer restake；
- **本 Plugin（swellchain-staking）**：接入 earnETH 聚合 Vault 的 deposit/withdraw，以及 SimpleStakingERC20 合约（用于 SWELL 生态代币质押），差异化点在于 Swellchain L2 生态的 yield 产品和 L1→L2 bridge 流程。

> 注意：如果审查发现 swETH/rswETH 的 deposit/withdraw 已完整实现于上述 PR，则本 plugin 专注于 **earnETH Vault** 接入。若 PR 只实现了部分操作，补全即可。本 design.md 覆盖完整操作集，由 Developer Agent 按情况选择实现哪些。

### 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 否。Swell Network 无官方 Rust SDK |
| SDK 支持哪些技术栈？ | 无 SDK，仅有 swell-chain-docs（文档）和 v3-core-public（Solidity 合约开源）|
| 有 REST API？ | 无公开 REST API。链上数据通过 `eth_call` 读取，汇率通过合约 view 函数获取 |
| 有官方 Skill？ | 无官方 Claude Skill |
| 开源社区有类似 Skill？ | 无已知社区 Skill |
| 支持哪些链？ | Ethereum Mainnet (chain_id=1)，Swellchain L2 (chain_id=1923) |
| 是否需要 onchainos 广播？ | **是**。所有写操作均需通过 `onchainos wallet contract-call` 提交 |

### 接入路径

无 SDK，无 REST API → **直接合约交互**：

1. **链下查询**：通过 `eth_call` 读取汇率（`swETHToETHRate()`、`ethToSwETHRate()` 等）
2. **链上操作**：Rust 编码 calldata → `onchainos wallet contract-call --to <contract> --input-data <calldata>`
3. **ERC-20 approve**：先通过 `onchainos wallet contract-call` 调用 token 的 `approve(address,uint256)`，再执行 deposit
4. **合约来源**：GitHub [SwellNetwork/v3-core-public](https://github.com/SwellNetwork/v3-core-public)

---

## 2. DApp 核心能力 & 接口映射

### 2.1 合约地址

#### Ethereum Mainnet (chain_id = 1)

| 合约 | 地址 | 来源 |
|------|------|------|
| swETH Token (Proxy) | `0xf951E335afb289353dc249e82926178EaC7DEd78` | Etherscan verified |
| swETH Implementation | `0xce95ba824ae9a4df9b303c0bbf4d605ba2affbfc` | Etherscan (proxy impl) |
| rswETH Token (Proxy) | `0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0` | Etherscan verified |
| rswETH Implementation | `0x4796d939b22027c2876d5ce9fde52da9ec4e2362` | Etherscan (proxy impl) |
| swEXIT (Withdrawal NFT, Proxy) | `0x48C11b86807627AF70a34662D4865cF854251663` | Ethplorer / Etherscan |
| SimpleStakingERC20 | `0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774` | Etherscan verified |
| earnETH BoringVault | `0x9Ed15383940CC380fAEF0a75edacE507cC775f22` | Etherscan verified (Nucleus) |
| SWELL Token | `0x0a6e7ba5042b38349e437ec6db6214aec7b35676` | Etherscan |

> **注意**：earnETH BoringVault 的 `enter()`/`exit()` 受 Auth 权限保护，外部用户不能直接调用。实际用户入口是 **Teller 合约**（TellerWithMultiAssetSupport），Teller 地址需从链上 `beforeTransferHook` 或 Nucleus 文档获取。当前 earnETH Vault 通过 Teller 路由用户存款，Developer 需在实现时从链上动态查询 Teller 地址，或参考 [nucleus-boring-vault](https://github.com/Ion-Protocol/nucleus-boring-vault) 部署配置。若 Teller 不可公开调用，则采用 SimpleStakingERC20 作为本 plugin 的核心接入合约（支持 swETH/rswETH 的 deposit/withdraw）。

#### Swellchain L2 (chain_id = 1923)

| 合约 | 地址 | 来源 |
|------|------|------|
| L1StandardBridgeProxy (on L1) | `0x7aA4960908B13D104bf056B23E2C76B43c5AACc8` | build.swellnetwork.io |
| OptimismPortalProxy (on L1) | `0x758E0EE66102816F5C3Ec9ECc1188860fbb87812` | build.swellnetwork.io |
| RPC | `https://swell-mainnet.alt.technology` | chainid.network |
| Block Explorer | `https://explorer.swellnetwork.io` | 官方 |

> rswETH 是 Swellchain 的原生 Gas 代币。Swellchain 上的 DeFi（如 Euler 杠杆质押）不在本 plugin 范围内。

### 2.2 操作表

| # | 操作 | 类型 | 合约 | 链 |
|---|------|------|------|----|
| 1 | stake-eth (存 ETH → 铸造 swETH) | 链上写 | swETH Proxy | Ethereum L1 |
| 2 | unstake-sweth (请求 swETH → ETH 赎回) | 链上写 | swEXIT Proxy | Ethereum L1 |
| 3 | claim-sweth (领取已完成的赎回 ETH) | 链上写 | swEXIT Proxy | Ethereum L1 |
| 4 | restake-eth (存 ETH → 铸造 rswETH) | 链上写 | rswETH Proxy | Ethereum L1 |
| 5 | deposit-earn (存 swETH/rswETH → earnETH) | 链上写 | SimpleStakingERC20 | Ethereum L1 |
| 6 | withdraw-earn (取回 swETH/rswETH) | 链上写 | SimpleStakingERC20 | Ethereum L1 |
| 7 | get-sweth-rate (查询 swETH 汇率/APR) | 链下查询 | swETH Proxy (eth_call) | Ethereum L1 |
| 8 | get-rsweth-rate (查询 rswETH 汇率/APR) | 链下查询 | rswETH Proxy (eth_call) | Ethereum L1 |
| 9 | get-balance (查询持仓余额) | 链下查询 | ERC-20 balanceOf | Ethereum L1 |

### 2.3 链下查询

#### get-sweth-rate

| 字段 | 值 |
|------|-----|
| 方法 | `eth_call` |
| 合约 | `0xf951E335afb289353dc249e82926178EaC7DEd78` (swETH Proxy) |
| 函数 | `swETHToETHRate()` → `uint256` (1 swETH 对应的 ETH，1e18 精度) |
| 函数 | `ethToSwETHRate()` → `uint256` (1 ETH 对应的 swETH，1e18 精度) |
| 函数 | `getRate()` → `uint256` (与 swETHToETHRate 相同，Chainlink 兼容接口) |
| 返回 | 原始 uint256 / 1e18 = 汇率小数 |

#### get-rsweth-rate

| 字段 | 值 |
|------|-----|
| 方法 | `eth_call` |
| 合约 | `0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0` (rswETH Proxy) |
| 函数 | `rswETHToETHRate()` → `uint256` |
| 函数 | `ethToRswETHRate()` → `uint256` |
| 函数 | `getRate()` → `uint256` |
| 返回 | 原始 uint256 / 1e18 = 汇率小数 |

#### get-balance

| 字段 | 值 |
|------|-----|
| 方法 | `eth_call` |
| 函数 | `balanceOf(address)` → `uint256` |
| 适用 | swETH、rswETH、earnETH（ERC-20 标准） |

#### get-withdraw-status (swEXIT)

| 字段 | 值 |
|------|-----|
| 方法 | `eth_call` |
| 合约 | `0x48C11b86807627AF70a34662D4865cF854251663` (swEXIT) |
| 函数 | `getLastTokenIdCreated()` → `uint256` |
| 函数 | `getLastTokenIdProcessed()` → `uint256` |
| 函数 | `getProcessedRateForTokenId(uint256 tokenId)` → `(bool isProcessed, uint256 processedRate)` |

### 2.4 链上写操作

**EVM 链操作表（所有操作均通过 `onchainos wallet contract-call` 提交）**

#### 操作 1：stake-eth（ETH → swETH）

| 列 | 值 |
|----|-----|
| 合约地址 | `0xf951E335afb289353dc249e82926178EaC7DEd78` |
| 来源 | Etherscan 验证合约 |
| 函数签名 | `deposit()` |
| Selector | `0xd0e30db0` ✅ (`cast sig "deposit()"`) |
| 参数 | 无（ETH 作为 msg.value 传入） |
| 备注 | payable，用户发送的 ETH 金额即为质押量 |

onchainos 命令：
```bash
onchainos wallet contract-call \
  --chain ethereum \
  --to 0xf951E335afb289353dc249e82926178EaC7DEd78 \
  --value <amount_in_wei> \
  --input-data 0xd0e30db0 \
  --from <user_address>
```

---

#### 操作 2：unstake-sweth（请求赎回 swETH → ETH，铸造 swEXIT NFT）

**前置**：需先 approve swEXIT 合约使用 swETH

| 列 | 值 |
|----|-----|
| 合约地址 | `0x48C11b86807627AF70a34662D4865cF854251663` |
| 来源 | Ethplorer / Etherscan 验证 |
| 函数签名 | `createWithdrawRequest(uint256)` |
| Selector | `0x74dc9d1a` ✅ (`cast sig "createWithdrawRequest(uint256)"`) |
| ABI 参数顺序 | `amount` (uint256)：要赎回的 swETH 数量（wei 精度） |
| 备注 | 调用后铸造 swEXIT NFT，赎回期 1–12 天；每次最大 500 ETH |

**Step 0：approve swETH → swEXIT**
```bash
# ERC-20 approve: swETH.approve(swEXIT_contract, amount)
onchainos wallet contract-call \
  --chain ethereum \
  --to 0xf951E335afb289353dc249e82926178EaC7DEd78 \
  --input-data <encode approve(address,uint256) with spender=0x48C11b86807627AF70a34662D4865cF854251663 amount=<amount_wei>> \
  --from <user_address>
```

**Step 1：createWithdrawRequest**
```bash
onchainos wallet contract-call \
  --chain ethereum \
  --to 0x48C11b86807627AF70a34662D4865cF854251663 \
  --input-data <encode createWithdrawRequest(uint256) with amount=<amount_wei>> \
  --from <user_address>
```

---

#### 操作 3：claim-sweth（领取已完成赎回的 ETH）

| 列 | 值 |
|----|-----|
| 合约地址 | `0x48C11b86807627AF70a34662D4865cF854251663` |
| 来源 | Ethplorer / Etherscan 验证 |
| 函数签名 | `finalizeWithdrawal(uint256)` |
| Selector | `0x5e15c749` ✅ (`cast sig "finalizeWithdrawal(uint256)"`) |
| ABI 参数顺序 | `tokenId` (uint256)：swEXIT NFT 的 tokenId |
| 备注 | 仅在 `getProcessedRateForTokenId(tokenId).isProcessed == true` 后可调用 |

```bash
onchainos wallet contract-call \
  --chain ethereum \
  --to 0x48C11b86807627AF70a34662D4865cF854251663 \
  --input-data <encode finalizeWithdrawal(uint256) with tokenId=<token_id>> \
  --from <user_address>
```

---

#### 操作 4：restake-eth（ETH → rswETH）

| 列 | 值 |
|----|-----|
| 合约地址 | `0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0` |
| 来源 | Etherscan 验证合约 |
| 函数签名 | `deposit()` |
| Selector | `0xd0e30db0` ✅ (`cast sig "deposit()"`) |
| 参数 | 无（ETH 作为 msg.value 传入） |
| 备注 | payable，与 swETH 的 deposit 函数签名相同，但合约地址不同 |

```bash
onchainos wallet contract-call \
  --chain ethereum \
  --to 0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0 \
  --value <amount_in_wei> \
  --input-data 0xd0e30db0 \
  --from <user_address>
```

---

#### 操作 5：deposit-earn（存 swETH/rswETH → SimpleStakingERC20）

**前置**：需先 approve SimpleStakingERC20 使用对应 token

| 列 | 值 |
|----|-----|
| 合约地址 | `0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774` |
| 来源 | Etherscan "Swell Network: Simple Staking ERC20" |
| 函数签名 | `deposit(address,uint256,address)` |
| Selector | `0xf45346dc` ✅ (`cast sig "deposit(address,uint256,address)"`) |
| ABI 参数顺序 | `_token` (address), `_amount` (uint256), `_receiver` (address) |
| 备注 | `_token` 传入 swETH 或 rswETH 地址；`_receiver` 通常为 msg.sender |

**Step 0：approve**（以 swETH 为例）
```bash
# swETH.approve(SimpleStakingERC20, amount)
onchainos wallet contract-call \
  --chain ethereum \
  --to 0xf951E335afb289353dc249e82926178EaC7DEd78 \
  --input-data <encode approve(address,uint256) with spender=0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774 amount=<amount_wei>> \
  --from <user_address>
```

**Step 1：deposit**
```bash
onchainos wallet contract-call \
  --chain ethereum \
  --to 0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774 \
  --input-data <encode deposit(address,uint256,address) with _token=0xf951..., _amount=<wei>, _receiver=<user>> \
  --from <user_address>
```

---

#### 操作 6：withdraw-earn（从 SimpleStakingERC20 取回 swETH/rswETH）

| 列 | 值 |
|----|-----|
| 合约地址 | `0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774` |
| 来源 | Etherscan 验证合约 |
| 函数签名 | `withdraw(address,uint256,address)` |
| Selector | `0x69328dec` ✅ (`cast sig "withdraw(address,uint256,address)"`) |
| ABI 参数顺序 | `_token` (address), `_amount` (uint256), `_receiver` (address) |
| 备注 | 即时取回，无锁定期；`_receiver` 通常为 msg.sender |

```bash
onchainos wallet contract-call \
  --chain ethereum \
  --to 0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774 \
  --input-data <encode withdraw(address,uint256,address) with _token=<token_addr>, _amount=<wei>, _receiver=<user>> \
  --from <user_address>
```

---

### 2.5 Function Selector 验证汇总

所有 selector 均通过 `cast sig` 验证：

| 操作 | 函数签名 | Selector |
|------|---------|---------|
| stake-eth / restake-eth | `deposit()` | `0xd0e30db0` ✅ |
| depositWithReferral | `depositWithReferral(address)` | `0xc18d7cb7` ✅ |
| unstake-sweth | `createWithdrawRequest(uint256)` | `0x74dc9d1a` ✅ |
| claim-sweth | `finalizeWithdrawal(uint256)` | `0x5e15c749` ✅ |
| processWithdrawals | `processWithdrawals(uint256)` | `0x152fcb0c` ✅ |
| deposit-earn | `deposit(address,uint256,address)` | `0xf45346dc` ✅ |
| withdraw-earn | `withdraw(address,uint256,address)` | `0x69328dec` ✅ |
| get rate | `swETHToETHRate()` | `0xd68b2cb6` ✅ |
| get rate | `ethToSwETHRate()` | `0x0de3ff57` ✅ |
| get rate | `rswETHToETHRate()` | `0xa7b9544e` ✅ |
| get rate | `ethToRswETHRate()` | `0x780a47e0` ✅ |
| get rate | `getRate()` | `0x679aefce` ✅ |
| withdraw status | `getLastTokenIdCreated()` | `0x061a499f` ✅ |
| withdraw status | `getLastTokenIdProcessed()` | `0xb61d5978` ✅ |
| withdraw status | `getProcessedRateForTokenId(uint256)` | `0xde886fb0` ✅ |

---

## 3. 用户场景

### 场景 1：用户质押 ETH 获取 swETH

**用户说**："帮我把 0.5 ETH 质押到 Swell，获取 swETH"

**Agent 动作序列**：
1. [链下查询] 调用 `ethToSwETHRate()` 读取当前汇率，告知用户预计获得 ~X swETH
2. [链下查询] 调用 `swETHToETHRate()` 显示当前 APR（根据汇率变化推算，约 3%）
3. [链上操作] 构造 `deposit()` calldata = `0xd0e30db0`，通过 onchainos 发送 0.5 ETH
   ```
   onchainos wallet contract-call \
     --chain ethereum \
     --to 0xf951E335afb289353dc249e82926178EaC7DEd78 \
     --value 500000000000000000 \
     --input-data 0xd0e30db0 \
     --from <user_address>
   ```
4. [链下查询] 等待 tx 确认后，调用 `balanceOf(user)` 验证 swETH 余额增加
5. 告知用户：质押成功，获得 X.XXXX swETH，当前 APR ~3%

---

### 场景 2：用户从 Swell 赎回质押的 ETH（两步：请求 + 领取）

**用户说**："我想取回我在 Swell 质押的 0.1 swETH，换回 ETH"

**Agent 动作序列**：
1. [链下查询] 调用 `balanceOf(user)` 确认用户有足够 swETH 余额
2. [链下查询] 调用 `swETHToETHRate()` 计算预计可取 ETH 数量
3. [链上操作 — Step A: approve] 构造 `approve(0x48C11b..., amount)` calldata，调用 swETH 合约
   ```
   onchainos wallet contract-call \
     --chain ethereum \
     --to 0xf951E335afb289353dc249e82926178EaC7DEd78 \
     --input-data <approve calldata> \
     --from <user_address>
   ```
4. [链上操作 — Step B: createWithdrawRequest] 构造 `createWithdrawRequest(uint256)` calldata
   ```
   onchainos wallet contract-call \
     --chain ethereum \
     --to 0x48C11b86807627AF70a34662D4865cF854251663 \
     --input-data <createWithdrawRequest(0.1e18) calldata> \
     --from <user_address>
   ```
5. 告知用户：赎回请求已创建，NFT tokenId=X，预计 1–12 天后可领取
6. [用户后续询问] "我的赎回申请好了吗？"
7. [链下查询] 调用 `getProcessedRateForTokenId(tokenId)` 检查 `isProcessed`
8. [如果 isProcessed=true，链上操作] 构造 `finalizeWithdrawal(tokenId)` 调用 swEXIT
   ```
   onchainos wallet contract-call \
     --chain ethereum \
     --to 0x48C11b86807627AF70a34662D4865cF854251663 \
     --input-data <finalizeWithdrawal(tokenId) calldata> \
     --from <user_address>
   ```
9. 告知用户：ETH 已转入钱包

---

### 场景 3：用户将 rswETH 存入 Swell Earn 获取额外收益

**用户说**："我有 1 rswETH，帮我存入 Swell Earn 赚积分和额外收益"

**Agent 动作序列**：
1. [链下查询] 调用 rswETH 合约 `balanceOf(user)` 确认余额 ≥ 1 rswETH
2. [链下查询] 调用 `rswETHToETHRate()` 显示当前 rswETH 价值（约 1.045 ETH）
3. 告知用户：将把 1 rswETH 存入 SimpleStakingERC20，获得 Swell 积分和额外 DeFi 收益
4. [链上操作 — Step A: approve] 调用 rswETH 合约的 `approve(SimpleStakingERC20, 1e18)`
   ```
   onchainos wallet contract-call \
     --chain ethereum \
     --to 0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0 \
     --input-data <approve(0x38d43a6C..., 1000000000000000000) calldata> \
     --from <user_address>
   ```
5. [链上操作 — Step B: deposit] 调用 SimpleStakingERC20 的 `deposit(rswETH, 1e18, user)`
   ```
   onchainos wallet contract-call \
     --chain ethereum \
     --to 0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774 \
     --input-data <deposit(0xFAe103DC..., 1000000000000000000, <user>) calldata> \
     --from <user_address>
   ```
6. 告知用户：1 rswETH 已存入 Swell Earn，开始自动累积收益和积分

---

### 场景 4：查询当前质押状态和收益

**用户说**："显示我在 Swell 的所有质押状态"

**Agent 动作序列**：
1. [链下查询] 调用 swETH `balanceOf(user)` 获取 swETH 余额
2. [链下查询] 调用 rswETH `balanceOf(user)` 获取 rswETH 余额
3. [链下查询] 调用 SimpleStakingERC20 持仓（通过事件或 Etherscan API 查询存款记录）
4. [链下查询] 调用 `swETHToETHRate()` 和 `rswETHToETHRate()` 获取当前汇率
5. [链下查询] 调用 swEXIT `getLastTokenIdCreated()` 和 `getLastTokenIdProcessed()` 检查是否有待处理的赎回请求
6. 汇总展示：
   - swETH 余额: X.XXX（约 Y.YYY ETH，APR ~3%）
   - rswETH 余额: A.AAA（约 B.BBB ETH，APR ~2.63%）
   - Earn 存款: C.CCC swETH + D.DDD rswETH
   - 待处理赎回: [有/无]

---

## 4. 外部 API 依赖

| API | 用途 | 认证 | 备注 |
|-----|------|------|------|
| Ethereum RPC (`eth_call`) | 读取汇率、余额、提款状态 | 无（公共节点）或 API Key | 使用 infura/alchemy/public RPC |
| Swellchain RPC | 查询 L2 数据（如需） | 无 | `https://swell-mainnet.alt.technology` |
| Etherscan API（可选） | 查询 swEXIT NFT 的 tokenId | API Key | 辅助查询，非必须 |

> Swell Network **没有公开 REST API**。所有数据通过链上 `eth_call` 读取。APR 数据可通过汇率历史推算，或从 DefiLlama/第三方接口获取：`https://yields.llama.fi/pools`（筛选 project=swell）

---

## 5. 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `ethereum_rpc_url` | String | `https://eth-mainnet.g.alchemy.com/v2/...` | Ethereum L1 RPC |
| `swellchain_rpc_url` | String | `https://swell-mainnet.alt.technology` | Swellchain L2 RPC（备用） |
| `sweth_contract` | Address | `0xf951E335afb289353dc249e82926178EaC7DEd78` | swETH Token Proxy |
| `rsweth_contract` | Address | `0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0` | rswETH Token Proxy |
| `swexit_contract` | Address | `0x48C11b86807627AF70a34662D4865cF854251663` | swEXIT Withdrawal NFT |
| `simple_staking_contract` | Address | `0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774` | SimpleStakingERC20 |
| `dry_run` | bool | `false` | 若 `true`，仅编码 calldata，不通过 onchainos 广播 |

---

## 6. 已知注意事项 & 风险

### 6.1 unstake 需两步
swETH 赎回分两步：`createWithdrawRequest`（创建请求，1–12 天等待）→ `finalizeWithdrawal`（领取）。Agent 需告知用户等待期，并在用户查询时通过 `getProcessedRateForTokenId` 检查状态。

### 6.2 swEXIT tokenId 获取
`createWithdrawRequest` 不直接返回 tokenId，需通过监听 `Transfer` 事件或调用 `getLastTokenIdCreated()` 推断。Agent 实现时应记录 tx hash，通过 receipt 解析 event log 获取 tokenId。

### 6.3 SimpleStakingERC20 的 deposit/withdraw 与 earnETH BoringVault 的关系
- `SimpleStakingERC20` 是 Swell 官方合约，当前持有 rswETH 约 $2.6M + swETH 约 $810k，**可公开调用**，无需特殊权限。
- `earnETH BoringVault`（`0x9Ed15383...`）的 `enter()`/`exit()` 需要 Auth 权限，外部用户需通过 Teller 合约。若 Teller 地址未知，**优先使用 SimpleStakingERC20** 作为 earnETH 的可访问替代方案。

### 6.4 rswETH 赎回（未接入）
rswETH 的赎回机制（21 天 EigenLayer 解锁期）与 swETH 不同，且没有类似 swEXIT 的简单接口（v2 机制复杂）。本 plugin 暂**不接入 rswETH 赎回**，仅接入 rswETH deposit。用户如需赎回 rswETH 可通过二级市场（DEX）交换。

### 6.5 approve 精度
approve 时传入精确金额（非 `type(uint256).max`），避免无限授权带来的安全风险。

### 6.6 与已有 PR 的边界
- 若 PR #141 已完整实现 `stake-eth`（swETH deposit），本 plugin **跳过该操作**，专注 SimpleStakingERC20 的 `deposit-earn`/`withdraw-earn`。
- 若 PR #179 已完整实现 `restake-eth`（rswETH deposit），本 plugin **跳过该操作**。
- Developer 在实现前需确认两个 PR 已实现的功能范围。

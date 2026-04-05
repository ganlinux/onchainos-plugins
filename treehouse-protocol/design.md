# Plugin Design: treehouse-protocol

## §0 Plugin Meta

| 字段 | 值 |
|------|-----|
| `plugin_name` | `treehouse-protocol` |
| `dapp_name` | Treehouse Protocol |
| `target_chains` | Ethereum (1), Avalanche (43114) |
| `target_protocols` | tETH (liquid staking yield token), tAVAX (Avalanche liquid staking yield token) |
| `plugin_version` | 0.1.0 |
| `接入路径` | API（链下查询走 on-chain view calls；链上写操作走 onchainos） |

> **范围说明**：Treehouse Protocol 在 Ethereum 上主要产品是 **tETH**（ETH 流动质押收益代币），在 Avalanche 上是 **tAVAX**。Mantle 上的 cmETH 实际上是 mETH Protocol 的 restaking 产品（Treehouse 只是托管其 Vault），机制完全不同，本期不接入。tETH 的赎回机制涉及 Curve AMM 链上 swap（小额）和 7 天等待期标准赎回（大额），本期聚焦 deposit + 小额 Curve swap 赎回 + 余额查询。

---

## §1 接入可行性调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无。Treehouse Protocol 无官方 Rust SDK |
| SDK 支持哪些技术栈？ | 无 SDK。TypeScript SDK 未公开发布；仅有智能合约直接交互 |
| 有 REST API？ | 无公开 REST API。链下数据（余额、APY）通过链上 view call 获取；机构级 API 在 institutions.treehouse.finance，但未公开文档 |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无。GitHub 无公开的 treehouse-finance / treehouse-protocol SDK 仓库 |
| 支持哪些链？ | Ethereum (1)、Avalanche (43114)、Arbitrum One（tETH 桥接代币，非原生 vault）、Base（tETH 桥接代币）、Mantle（cmETH 托管 vault，不在本期范围） |
| 是否需要 onchainos 广播？ | Yes。所有写操作（deposit ETH、deposit ERC-20 token、Curve swap 赎回）均需链上广播 |

### 接入路径

**路径：直接合约调用（via onchainos `contract-call`）**

无 SDK、无公开 REST API，直接调用已验证的合约函数：
- Ethereum：通过 `tETH Router (0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5)` 存入；通过 Curve StableSwap pool `(0xA10d15538E09479186b4D3278BA5c979110dDdB1)` 赎回（小额）
- Avalanche：通过 `tAVAX Router (0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e)` 存入

---

## §2 接口映射

### 2a. 需要接入的操作表

| 操作 | 链 | 链上/链下 | 说明 |
|------|-----|-----------|------|
| `deposit-eth` | Ethereum | 链上 | 存入原生 ETH，获得 tETH |
| `deposit-token` | Ethereum | 链上 | 存入 WETH / stETH / wstETH，获得 tETH；需先 approve |
| `redeem` | Ethereum | 链上 | 通过 Curve pool swap tETH → wstETH（小额快速赎回，≤200 wstETH） |
| `deposit-avax` | Avalanche | 链上 | 存入原生 AVAX，获得 tAVAX |
| `deposit-token-avax` | Avalanche | 链上 | 存入 sAVAX，获得 tAVAX；需先 approve |
| `get-balance` | Ethereum / Avalanche | 链下 | 查询用户 tETH / tAVAX 余额 |
| `get-exchange-rate` | Ethereum / Avalanche | 链下 | 查询 tETH / tAVAX → wstETH / sAVAX 当前兑换率 |
| `get-apy` | Ethereum | 链下 | 查询 tETH 当前 APY（通过 Strategy 合约或 DeFiLlama API） |

### 2b. 链下查询表

#### `get-balance`（tETH 余额）

- **合约地址**: `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8`（tETH Token, Ethereum）
- **函数**: `balanceOf(address account) → uint256`
- **参数**: `account`：用户钱包地址
- **返回**: tETH 余额（18 decimals）

#### `get-balance`（tAVAX 余额）

- **合约地址**: `0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646`（tAVAX Token, Avalanche）
- **函数**: `balanceOf(address account) → uint256`
- **参数**: `account`：用户钱包地址
- **返回**: tAVAX 余额（18 decimals）

#### `get-exchange-rate`（tETH → wstETH）

- **合约地址**: `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8`（tETH Token/ERC4626 implementation, Ethereum）
- **函数**: `convertToAssets(uint256 shares) → uint256`（ERC-4626 standard）
- **参数**: `shares`：tETH 数量（如 `1e18` 查 1 tETH 对应的 wstETH）
- **返回**: wstETH 数量（18 decimals）

> 注意：tETH 合约是 ERC1967 Proxy，实际 implementation 在 `0xd1a622566f277aa76c3c47a30469432aaec95e38`（TAsset 合约），但调用代理地址即可。

#### `get-exchange-rate`（tAVAX → sAVAX）

- **合约地址**: `0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646`（tAVAX Token, Avalanche）
- **函数**: `convertToAssets(uint256 shares) → uint256`（ERC-4626 standard）
- **参数**: `shares`：tAVAX 数量
- **返回**: sAVAX 数量（18 decimals）

#### `get-apy`

- 链上无直接 APY view 函数，APY 通过计算 `convertToAssets` 随时间的变化率得出，或查询 DeFiLlama API：
  - **Endpoint**: `https://yields.llama.fi/pools`（GET，过滤 `project=treehouse-protocol`）
  - 无需认证，公开 API
  - 返回字段：`apy`（当前年化收益率）, `tvlUsd`

### 2c. 链上写操作表

#### Ethereum — deposit-eth（存入原生 ETH，获得 tETH）

| 字段 | 值 |
|------|-----|
| 操作 | `deposit-eth` |
| 合约地址 | `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5`（tETH Router，固定地址） |
| 函数签名 | `depositETH()` |
| Selector | `0xf6326fb3` ✅（`cast sig "depositETH()"` 验证） |
| ABI 参数 | 无参数；ETH 通过 `msg.value` 传入 |
| onchainos 命令 | `wallet contract-call --to 0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5 --value <amount_in_wei> --data 0xf6326fb3 --from <wallet>` |

#### Ethereum — approve（ERC-20 存入前 approve，如 WETH/stETH/wstETH）

| 字段 | 值 |
|------|-----|
| 操作 | `approve` |
| 合约地址 | ERC-20 token 地址（WETH: `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` / stETH: `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` / wstETH: `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0`） |
| 函数签名 | `approve(address,uint256)` |
| Selector | `0x095ea7b3` ✅（`cast sig "approve(address,uint256)"` 验证） |
| ABI 参数 | `spender`（= Router `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5`），`amount`（uint256） |
| onchainos 命令 | `wallet contract-call --to <token_address> --data <encoded_approve_calldata> --from <wallet>` |

#### Ethereum — deposit-token（存入 WETH / stETH / wstETH，获得 tETH）

| 字段 | 值 |
|------|-----|
| 操作 | `deposit-token` |
| 合约地址 | `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5`（tETH Router） |
| 函数签名 | `deposit(address,uint256)` |
| Selector | `0x47e7ef24` ✅（`cast sig "deposit(address,uint256)"` 验证） |
| ABI 参数 | `_asset`（address，token 地址），`_amount`（uint256，token 数量，18 decimals） |
| ABI 参数顺序 | `_asset`, `_amount` |
| onchainos 命令 | `wallet contract-call --to 0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5 --data <encoded_deposit_calldata> --from <wallet>` |

> 前置条件：需先发送 `approve` 交易授权 Router 使用 token

#### Ethereum — redeem（小额快速赎回：tETH → wstETH via Curve pool）

| 字段 | 值 |
|------|-----|
| 操作 | `redeem` |
| 合约地址 | `0xA10d15538E09479186b4D3278BA5c979110dDdB1`（tETH/wstETH Curve StableSwapNG pool） |
| 函数签名 | `exchange(int128,int128,uint256,uint256)` |
| Selector | `0x3df02124` ✅（`cast sig "exchange(int128,int128,uint256,uint256)"` 验证） |
| ABI 参数 | `i`（int128，卖出 coin index：tETH = `0`），`j`（int128，买入 coin index：wstETH = `1`），`dx`（uint256，卖出 tETH 数量），`min_dy`（uint256，最小接收 wstETH，设 slippage 保护，建议 99% 预期值） |
| ABI 参数顺序 | `i`, `j`, `dx`, `min_dy` |
| onchainos 命令 | `wallet contract-call --to 0xA10d15538E09479186b4D3278BA5c979110dDdB1 --data <encoded_exchange_calldata> --from <wallet>` |

> 前置条件：需先 approve tETH 给 Curve pool
> 限制：仅适用于 ≤200 wstETH 的赎回量（Redemption Band）；超出此范围需走标准 7 天赎回流程（本期不接入）

#### Ethereum — approve tETH（Curve swap 前 approve）

| 字段 | 值 |
|------|-----|
| 操作 | `approve-teth` |
| 合约地址 | `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8`（tETH Token） |
| 函数签名 | `approve(address,uint256)` |
| Selector | `0x095ea7b3` ✅ |
| ABI 参数 | `spender`（= Curve pool `0xA10d15538E09479186b4D3278BA5c979110dDdB1`），`amount`（uint256） |

#### Avalanche — deposit-avax（存入原生 AVAX，获得 tAVAX）

| 字段 | 值 |
|------|-----|
| 操作 | `deposit-avax` |
| 合约地址 | `0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e`（tAVAX Router，固定地址） |
| 函数签名 | `depositAVAX()` |
| Selector | `0xa0d065c3` ✅（`cast sig "depositAVAX()"` 验证） |
| ABI 参数 | 无参数；AVAX 通过 `msg.value` 传入 |
| onchainos 命令 | `wallet contract-call --to 0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e --value <amount_in_wei> --data 0xa0d065c3 --from <wallet> --chain avalanche` |

#### Avalanche — deposit-token-avax（存入 sAVAX，获得 tAVAX）

| 字段 | 值 |
|------|-----|
| 操作 | `deposit-token-avax` |
| 合约地址 | `0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e`（tAVAX Router） |
| 函数签名 | `deposit(address,uint256)` |
| Selector | `0x47e7ef24` ✅ |
| ABI 参数 | `_asset`（address，如 sAVAX），`_amount`（uint256） |
| onchainos 命令 | `wallet contract-call --to 0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e --data <encoded_calldata> --from <wallet> --chain avalanche` |

> 前置条件：需先 approve tAVAX Router 使用 sAVAX（sAVAX 地址：`0x2b2C81e08f1Af8835a78Bb2A90AE924ACE0eA4bE`）

---

## §3 用户场景

### 场景 1：用户质押 ETH 获取 tETH 收益

**用户输入**:
> "帮我把 1 ETH 存入 Treehouse Protocol 获取 tETH"

**Agent 动作序列**:
1. [链下查询] 调用 `tETH Router.depositETH()` 的前置检查：读取 `depositCapInEth` view function，验证当前存款未超过上限
2. [链下查询] 预估将收到的 tETH 数量：调用 `convertToAssets(1e18)` 获取当前汇率（wstETH per tETH），换算出 1 ETH → tETH 预期数量（提示用户）
3. [链上操作] 调用 tETH Router `depositETH()`，附 `msg.value = 1 ETH`：
   ```
   wallet contract-call --to 0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5
     --value 1000000000000000000
     --data 0xf6326fb3
     --from <wallet>
   ```
4. [链下查询] 等待 tx 确认后，调用 `balanceOf(<wallet>)` 查询 tETH 新余额，向用户汇报

**预期结果**: 用户收到 tETH（≈ 1 ETH 等值），自动开始赚取 staking APY + MEY。

---

### 场景 2：用户存入 wstETH 获取 tETH

**用户输入**:
> "用我的 0.5 wstETH 存入 Treehouse 换 tETH"

**Agent 动作序列**:
1. [链下查询] 检查用户 wstETH 余额：`balanceOf(<wallet>)` on wstETH (`0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0`)，确认 ≥ 0.5 wstETH
2. [链下查询] 检查 tETH Router 当前 allowance：`allowance(<wallet>, 0xeFA3fa8e...)`，如不足则需 approve
3. [链上操作] Approve wstETH 给 Router（如 allowance 不足）：
   ```
   wallet contract-call --to 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0
     --data <encoded: approve(0xeFA3fa8e..., 500000000000000000)>
     --from <wallet>
   ```
4. [链上操作] 调用 tETH Router `deposit(wstETH_address, amount)`：
   ```
   wallet contract-call --to 0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5
     --data <encoded: deposit(0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0, 500000000000000000)>
     --from <wallet>
   ```
5. [链下查询] 确认后查询 tETH 新余额汇报给用户

---

### 场景 3：用户快速赎回 tETH → wstETH

**用户输入**:
> "帮我把 100 tETH 换回 wstETH"

**Agent 动作序列**:
1. [链下查询] 检查用户 tETH 余额（`balanceOf`），确认 ≥ 100 tETH
2. [链下查询] 评估赎回金额：调用 `convertToAssets(100e18)` 估算对应的 wstETH 数量；检查是否在 Redemption Band（≤200 wstETH），本场景适用 Curve swap
3. [链下查询] 调用 Curve pool 的 `get_dy(0, 1, 100e18)` 获取预期收到的 wstETH 数量，计算 `min_dy`（设 1% slippage：`min_dy = expected_dy * 99 / 100`）
4. [链上操作] Approve tETH 给 Curve pool（检查 allowance 后，如不足）：
   ```
   wallet contract-call --to 0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8
     --data <encoded: approve(0xA10d15538..., 100000000000000000000)>
     --from <wallet>
   ```
5. [链上操作] 调用 Curve pool `exchange(0, 1, 100e18, min_dy)`：
   ```
   wallet contract-call --to 0xA10d15538E09479186b4D3278BA5c979110dDdB1
     --data <encoded: exchange(0, 1, 100000000000000000000, <min_dy>)>
     --from <wallet>
   ```
6. [链下查询] 确认后查询用户 wstETH 余额，汇报到账数量

---

### 场景 4：用户在 Avalanche 存入 AVAX 获取 tAVAX

**用户输入**:
> "在 Avalanche 上质押 10 AVAX 到 Treehouse 获取 tAVAX"

**Agent 动作序列**:
1. [链下查询] 检查用户 AVAX 余额是否充足（> 10 AVAX + gas）
2. [链下查询] 调用 tAVAX Token `convertToAssets(1e18)` 查询当前汇率（预计收益提示）
3. [链上操作] 调用 tAVAX Router `depositAVAX()`，附 `msg.value = 10 AVAX`：
   ```
   wallet contract-call --to 0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e
     --value 10000000000000000000
     --data 0xa0d065c3
     --from <wallet>
     --chain avalanche
   ```
4. [链下查询] 确认后查询用户 tAVAX 余额（`balanceOf` on `0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646`）并汇报

---

### 场景 5：查询 tETH 持仓和当前 APY

**用户输入**:
> "查看我在 Treehouse 上的 tETH 余额和当前收益率"

**Agent 动作序列**:
1. [链下查询] 调用 `balanceOf(<wallet>)` on tETH Token (`0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8`)，获取 tETH 余额
2. [链下查询] 调用 `convertToAssets(tETH_balance)` 计算对应的 wstETH 价值
3. [链下查询] 请求 DeFiLlama Yields API：`GET https://yields.llama.fi/pools`，过滤 `project=treehouse-protocol` 获取当前 APY
4. [链下查询] （可选）通过链上计算：采样当前 `totalAssets()` 和 24 小时前的值，估算日收益率换算为 APY
5. 汇报：tETH 余额、对应的 wstETH/ETH 价值估算、当前 APY

---

## §4 外部 API 依赖

| API | 用途 | Endpoint | 认证 | 备注 |
|-----|------|----------|------|------|
| DeFiLlama Yields API | 获取 tETH / tAVAX 当前 APY 和 TVL | `GET https://yields.llama.fi/pools` | 无需认证 | 过滤 `project=treehouse-protocol`；字段 `apy`、`tvlUsd` |
| Ethereum RPC | 链上 view calls（余额、汇率） | 配置的 Ethereum RPC endpoint | 依配置 | `eth_call` 查询 ERC-20 balanceOf、convertToAssets |
| Avalanche RPC | 链上 view calls（tAVAX 余额、汇率） | 配置的 Avalanche C-Chain RPC endpoint | 依配置 | 同上 |

> **注意**：Treehouse Protocol 无公开 REST API 文档。机构级 API（institutions.treehouse.finance）需申请访问，不用于本 skill。

---

## §5 配置参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `default_chain` | `string` | `"ethereum"` | 默认使用的链（`ethereum` 或 `avalanche`） |
| `slippage_bps` | `u32` | `100` | Curve swap 的最大滑点（基点，100 = 1%） |
| `dry_run` | `bool` | `false` | 若 true，仅模拟不广播交易（输出 calldata） |
| `rpc_url_ethereum` | `string` | （使用全局 RPC 配置） | Ethereum RPC URL，用于 view calls |
| `rpc_url_avalanche` | `string` | （使用全局 RPC 配置） | Avalanche C-Chain RPC URL |

---

## §6 合约地址汇总

### Ethereum (Chain ID: 1)

| 合约 | 地址 | 说明 |
|------|------|------|
| tETH Token (Proxy) | `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8` | ERC-20 tETH；ERC1967 Proxy |
| tETH Token (Implementation) | `0xd1a622566f277aa76c3c47a30469432aaec95e38` | TAsset 实现合约（ERC-4626） |
| tETH Router | `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5` | 接受 ETH/WETH/stETH/wstETH 存款 |
| tETH Vault | `0x551d155760ae96050439ad24ae98a96c765d761b` | 策略 vault（管理员操作） |
| tETH Strategy | `0x60d2D94aCB969CA54e781007eE89F04c1A2e5943` | 投资策略执行（Aave + Lido）|
| tETH/wstETH Curve Pool | `0xA10d15538E09479186b4D3278BA5c979110dDdB1` | CurveStableSwapNG；用于小额赎回 |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | 存款支持的 token |
| stETH (Lido) | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` | 存款支持的 token |
| wstETH (Lido) | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` | 存款支持的 token；赎回收到的 token |

### Avalanche C-Chain (Chain ID: 43114)

| 合约 | 地址 | 说明 |
|------|------|------|
| tAVAX Token | `0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646` | ERC-20 tAVAX |
| tAVAX Router | `0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e` | 接受 AVAX/sAVAX 存款 |
| tAVAX Vault | `0x3fc60aAc1d843e4e181C7Ab727A4027cb1Ac99ED` | 策略 vault |
| tAVAX-sAVAX LP | `0x832f8E068e92d56B94205eA605e5cDAa7CdEd1f0` | LP token |
| sAVAX (Benqi) | `0x2b2C81e08f1Af8835a78Bb2A90AE924ACE0eA4bE` | 存款支持的 token |

---

## §7 已验证 Function Selectors

| 函数签名 | Selector | 验证方式 | 用于 |
|---------|----------|----------|------|
| `depositETH()` | `0xf6326fb3` | `cast sig "depositETH()"` ✅ | tETH Router（Ethereum） |
| `deposit(address,uint256)` | `0x47e7ef24` | `cast sig "deposit(address,uint256)"` ✅ | tETH / tAVAX Router |
| `depositAVAX()` | `0xa0d065c3` | `cast sig "depositAVAX()"` ✅ | tAVAX Router（Avalanche） |
| `approve(address,uint256)` | `0x095ea7b3` | `cast sig "approve(address,uint256)"` ✅ | ERC-20 approve |
| `balanceOf(address)` | `0x70a08231` | `cast sig "balanceOf(address)"` ✅ | ERC-20 余额查询 |
| `convertToAssets(uint256)` | `0x07a2d13a` | `cast sig "convertToAssets(uint256)"` ✅ | ERC-4626 汇率查询 |
| `totalAssets()` | `0x01e1d114` | `cast sig "totalAssets()"` ✅ | ERC-4626 TVL 查询 |
| `exchange(int128,int128,uint256,uint256)` | `0x3df02124` | `cast sig "exchange(int128,int128,uint256,uint256)"` ✅ | Curve StableSwap 赎回 |

---

## §8 已知限制和陷阱

### 赎回限制（Redemption Band）
- tETH 小额赎回（≤200 wstETH）通过 Curve pool swap 完成，速度快（单笔 tx）
- 大额赎回（>200 wstETH）需走标准流程：等待约 7 天，手续费 0.05%；本期 **不实现** 标准赎回
- Fast Redemption（无数量限制，但收 2% 手续费）机制需调用专用合约，合约地址文档未公开，本期 **不实现**

### deposit 函数限制
- `deposit(address, uint256)` 只接受白名单 token（`getAllowableAssets()` 查询）；Ethereum 支持 WETH/stETH/wstETH
- `depositETH()` / `depositAVAX()` 只接受原生币
- 存款有上限（`depositCapInEth`），需在存款前检查

### 合约暂停风险
- Router 合约有暂停功能（`whenNotPaused`），链上操作前应检查 `paused()` 状态

### Avalanche tAVAX 赎回
- tAVAX 赎回机制未找到 Curve pool 对应，可能走 tAVAX-sAVAX LP 或标准赎回；**本期 tAVAX 只实现 deposit，不实现 redeem**

### Mantle cmETH 不在本期范围
- cmETH 是 mETH Protocol（Mantle 官方）的 restaking 产品，通过 BoringVault Teller 合约存入；与 Treehouse tETH/tAVAX 机制完全不同，本期不接入

# Reservoir Protocol — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `reservoir-protocol` |
| dapp_name | Reservoir Protocol |
| category | defi-protocol |
| tags | stablecoin, cdp, mint, yield, rwa, ethereum, berachain, base |
| target_chains | ethereum (1), base (8453), berachain (80094) |
| target_protocols | Reservoir Protocol |
| onchainos_broadcast | 是 |

---

## 1. Background

### 1.1 DApp 介绍

Reservoir Protocol 是一个去中心化稳定币协议，由 Fortunafi 团队开发，于 2024 年 8 月在以太坊主网正式上线。核心产品是 **rUSD**（Reservoir USD），一个以 USDC、USDT、USD1 等稳定币及真实世界资产（RWA）为支撑的稳定币。

**重要说明：** Reservoir Protocol (reservoir.xyz) 是稳定币 / CDP 协议，与 NFT 聚合工具 Reservoir (reservoir.tools/reservoirprotocol 在 GitHub) 是完全不同的两个项目。本 PRD 指的是前者。

**协议核心产品：**
- **rUSD**：以 USDC/USDT/RWA 1:1 支撑的稳定币（18 decimals，Ethereum 主网）
- **srUSD**：收益型稳定币，通过存入 rUSD 铸造，收益率由治理设定（目前约 7–8% APY）
- **wsrUSD**：srUSD 的 ERC-20 wrapped 版本，跨链使用
- **trUSD**：固定期限收益型稳定币（Term Issuer 管理）

**机制：**
1. 用户通过 Peg Stability Module（PSM）存入 USDC → 铸造等额 rUSD
2. 存入的 USDC 由协议通过 Asset Adapters 部署到 Morpho、Pendle 等收益来源
3. 用户可将 rUSD 存入 Saving Module → 按当前汇率铸造 srUSD
4. 赎回时：srUSD → rUSD（通过 Saving Module），rUSD → USDC（通过 PSM）

**链上现状（2026-04-05）：**
- rUSD 总供应量：约 633,871 rUSD
- PSM USDC 余额：约 521,191 USDC
- srUSD 当前价格：约 1.135 rUSD / srUSD（自 1.0 起持续增值）
- TVL：约 $526M（跨链，主要在 Ethereum）

### 1.2 接入可行性调研

| 维度 | 状态 |
|------|------|
| 官方 SDK | 无 |
| 官方 API | 无 |
| 开源合约 | 是（github.com/reservoir-protocol/reservoir，已审计） |
| 合约可读性 | 高（函数语义清晰） |
| 跨链部署 | Ethereum（核心），Base、Berachain 仅有 rUSD/srUSD/wsrUSD OFT |
| 开源社区同类 Skill | 无 |

### 1.3 接入路径

**路径：直接合约交互（无 SDK/API）**

所有链上操作通过 `onchainos wallet contract-call` 直接调用合约：
- 铸造 rUSD：ERC-20 approve USDC → Credit Enforcer `mintStablecoin`
- 铸造 srUSD：ERC-20 approve rUSD → Credit Enforcer `mintSavingcoin`（或直接 Saving Module）
- 赎回 rUSD→USDC：ERC-20 approve rUSD → PSM `redeem`
- 赎回 srUSD→rUSD：Saving Module `redeem`（无需额外 approve，合约直接 burn）

**注意：** Credit Enforcer 是用户铸造 rUSD 和 srUSD 的统一入口，内部自动与 PSM、Saving Module 交互并进行偿债能力验证。

**目前仅 Ethereum 主网有完整的 PSM + Credit Enforcer + Saving Module 部署，Base 和 Berachain 仅有 rUSD/srUSD OFT（桥接版本）。插件核心功能锁定 Ethereum mainnet（chain 1）。**

---

## 2. DApp 核心能力 & 接口映射

### 2.1 合约地址（Ethereum Mainnet，Chain 1）

| 合约名 | 地址 | 说明 |
|--------|------|------|
| rUSD Token | `0x09D4214C03D01F49544C0448DBE3A27f768F2b34` | ERC-20，18 decimals |
| srUSD Token | `0x738d1115B90efa71AE468F1287fc864775e23a31` | ERC-20，18 decimals |
| wsrUSD Token | `0xd3fd63209fa2d55b07a0f6db36c2f43900be3094` | wrapped srUSD |
| Saving Module | `0x5475611Dffb8ef4d697Ae39df9395513b6E947d7` | srUSD mint/redeem |
| PSM (USDC) | `0x4809010926aec940b550D34a46A52739f996D75D` | rUSD ↔ USDC 1:1 |
| PSM (USDT) | `0xeaE91B4C84e1EDfA5d78dcae40962C7655A549B9` | rUSD ↔ USDT 1:1 |
| Credit Enforcer | `0x04716DB62C085D9e08050fcF6F7D775A03d07720` | 用户铸造入口 |
| USDC (Ethereum) | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48` | 6 decimals |

**跨链 rUSD 地址（OFT，仅转账/持有，无 PSM 功能）：**

| 链 | rUSD 地址 | srUSD 地址 |
|----|-----------|------------|
| Base (8453) | `0x09D4214C03D01F49544C0448DBE3A27f768F2b34` | 无 |
| Berachain (80094) | `0x09D4214C03D01F49544C0448DBE3A27f768F2b34` | `0x5475611Dffb8ef4d697Ae39df9395513b6E947d7` |

### 2.2 操作列表

#### 操作 1：查询用户持仓（链下读取）

**目的：** 查询用户 rUSD、srUSD、wsrUSD 余额及 srUSD 当前价格

| 字段 | 详情 |
|------|------|
| 操作类型 | 链下 eth_call |
| 合约 | rUSD Token / srUSD Token / Saving Module |
| 函数 | `balanceOf(address)` / `currentPrice()` / `previewRedeem(uint256)` |

**调用序列：**

```
1. rUSD 余额：
   合约: 0x09D4214C03D01F49544C0448DBE3A27f768F2b34
   函数: balanceOf(address user)
   Selector: 0x70a08231
   返回: uint256 (18 decimals)

2. srUSD 余额：
   合约: 0x738d1115B90efa71AE468F1287fc864775e23a31
   函数: balanceOf(address user)
   Selector: 0x70a08231
   返回: uint256 (18 decimals)

3. srUSD 当前价格（rUSD/srUSD 汇率）：
   合约: 0x5475611Dffb8ef4d697Ae39df9395513b6E947d7 (Saving Module)
   函数: currentPrice()
   Selector: 0x9d1b464a
   返回: uint256（1e8 精度，例: 113561126 = 1.13561126 rUSD/srUSD）

4. srUSD 兑换预览（持有 N srUSD 可换多少 rUSD）：
   合约: 0x5475611Dffb8ef4d697Ae39df9395513b6E947d7
   函数: previewRedeem(uint256 srUSDAmount)
   Selector: 0x4cdad506
   参数: srUSDAmount（18 decimals）
   返回: uint256 rUSD 数量（18 decimals）
```

**Selector 验证（`cast sig` 已确认）：**
- `balanceOf(address)` → `0x70a08231` ✅
- `currentPrice()` → `0x9d1b464a` ✅
- `previewRedeem(uint256)` → `0x4cdad506` ✅

---

#### 操作 2：USDC 铸造 rUSD（链上操作）

**目的：** 用户存入 USDC，1:1 铸造等额 rUSD

| 字段 | 详情 |
|------|------|
| 操作类型 | 链上写入（2 步：approve + mintStablecoin） |
| 合约 | USDC Token → Credit Enforcer |
| 用户审批对象 | Credit Enforcer `0x04716DB62C085D9e08050fcF6F7D775A03d07720` |

**步骤 1：USDC approve（用户授权 Credit Enforcer 消费 USDC）**

| 字段 | 值 |
|------|-----|
| 合约地址 | `0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48`（USDC，Ethereum） |
| 函数签名 | `approve(address,uint256)` |
| Selector | `0x095ea7b3` ✅（cast sig 验证） |
| ABI 参数顺序 | `spender: address`（Credit Enforcer），`amount: uint256`（USDC 6 decimals） |

```bash
# ABI encode: approve(creditEnforcer, amount_usdc_6dec)
# calldata = 0x095ea7b3
#          + 000...04716DB62C085D9e08050fcF6F7D775A03d07720  (spender, 32 bytes)
#          + 000...amount_in_usdc_6dec                        (amount, 32 bytes)

onchainos wallet contract-call \
  --chain 1 \
  --to 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --input-data <approve_calldata>
```

**步骤 2：Credit Enforcer mintStablecoin（铸造 rUSD）**

| 字段 | 值 |
|------|-----|
| 合约地址 | `0x04716DB62C085D9e08050fcF6F7D775A03d07720`（Credit Enforcer） |
| 函数签名 | `mintStablecoin(uint256)` |
| Selector | `0xa0b4dbb1` ✅（cast sig 验证） |
| ABI 参数顺序 | `amount: uint256`（USDC 金额，**6 decimals**，USDC 精度） |

```bash
# ABI encode: mintStablecoin(amount_usdc_6dec)
# calldata = 0xa0b4dbb1
#          + 000...amount_in_6dec                             (amount, 32 bytes)

onchainos wallet contract-call \
  --chain 1 \
  --to 0x04716DB62C085D9e08050fcF6F7D775A03d07720 \
  --input-data <mintStablecoin_calldata>
```

**注意事项：**
- amount 参数单位为 **USDC 的 6 decimals**（例如：100 USDC = `100000000`）
- Credit Enforcer 内部自动从用户地址 transferFrom USDC，调用 PSM.allocate，然后铸造等额 rUSD（18 decimals）
- Credit Enforcer 会检查 PSM 债务上限和协议偿债比率，不满足时 revert
- approve 与 mintStablecoin 之间加 3 秒延迟，避免 nonce 碰撞

**备用：mintStablecoin(address,uint256)** — 可将 rUSD 铸造给指定地址

| Selector | `0xa7379086` ✅ |
| ABI 顺序 | `to: address`, `amount: uint256`（6 decimals） |

---

#### 操作 3：rUSD 赎回为 USDC（链上操作）

**目的：** 用户将 rUSD 1:1 赎回为 USDC

| 字段 | 详情 |
|------|------|
| 操作类型 | 链上写入（2 步：approve + redeem） |
| 合约 | rUSD Token → PSM (USDC) |
| 用户审批对象 | PSM `0x4809010926aec940b550D34a46A52739f996D75D` |

**步骤 1：rUSD approve（用户授权 PSM）**

| 字段 | 值 |
|------|-----|
| 合约地址 | `0x09D4214C03D01F49544C0448DBE3A27f768F2b34`（rUSD） |
| 函数签名 | `approve(address,uint256)` |
| Selector | `0x095ea7b3` ✅ |
| ABI 参数顺序 | `spender: address`（PSM USDC），`amount: uint256`（rUSD 18 decimals） |

**步骤 2：PSM redeem（赎回 USDC）**

| 字段 | 值 |
|------|-----|
| 合约地址 | `0x4809010926aec940b550D34a46A52739f996D75D`（PSM USDC） |
| 函数签名 | `redeem(uint256)` |
| Selector | `0xdb006a75` ✅（cast sig 验证） |
| ABI 参数顺序 | `amount: uint256`（rUSD 数量，**18 decimals**） |

```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x4809010926aec940b550D34a46A52739f996D75D \
  --input-data <redeem_calldata>
```

**注意事项：**
- PSM 赎回可能受流动性限制（PSM 余额不足时 revert）
- 赎回前可查询 `PSM.underlyingBalance()` 确认 USDC 余额（Selector: `0x59356c5c` ✅）
- 赎回返回 USDC（6 decimals），数量 = rUSD amount（18 decimals）/ 1e12

---

#### 操作 4：rUSD 铸造 srUSD（链上操作）

**目的：** 用户存入 rUSD，铸造收益型 srUSD

| 字段 | 详情 |
|------|------|
| 操作类型 | 链上写入（2 步：approve + mintSavingcoin） |
| 合约 | rUSD Token → Credit Enforcer |
| 用户审批对象 | Credit Enforcer `0x04716DB62C085D9e08050fcF6F7D775A03d07720` |

**步骤 1：rUSD approve**

| 合约地址 | `0x09D4214C03D01F49544C0448DBE3A27f768F2b34`（rUSD） |
|---------|------|
| 函数签名 | `approve(address,uint256)` |
| Selector | `0x095ea7b3` ✅ |
| ABI 顺序 | `spender: address`（Credit Enforcer），`amount: uint256`（18 decimals） |

**步骤 2：Credit Enforcer mintSavingcoin（铸造 srUSD）**

| 字段 | 值 |
|------|-----|
| 合约地址 | `0x04716DB62C085D9e08050fcF6F7D775A03d07720`（Credit Enforcer） |
| 函数签名 | `mintSavingcoin(uint256)` |
| Selector | `0x660cf34e` ✅（cast sig 验证） |
| ABI 参数顺序 | `amount: uint256`（rUSD 数量，**18 decimals**） |

```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x04716DB62C085D9e08050fcF6F7D775A03d07720 \
  --input-data <mintSavingcoin_calldata>
```

**铸造预览（链下）：**
- 调用 `SavingModule.previewMint(uint256 rUSDAmount)` 查询将获得多少 srUSD
- Selector: `0xb3d7f6b9` ✅

---

#### 操作 5：srUSD 赎回为 rUSD（链上操作）

**目的：** 用户赎回 srUSD，换回 rUSD（含累积收益）

| 字段 | 详情 |
|------|------|
| 操作类型 | 链上写入（1 步，无需额外 approve） |
| 合约 | Saving Module |
| 说明 | Saving Module 直接 burn caller 的 srUSD，mint rUSD 给 caller |

**Saving Module redeem**

| 字段 | 值 |
|------|-----|
| 合约地址 | `0x5475611Dffb8ef4d697Ae39df9395513b6E947d7`（Saving Module） |
| 函数签名 | `redeem(uint256)` |
| Selector | `0xdb006a75` ✅（cast sig 验证） |
| ABI 参数顺序 | `amount: uint256`（srUSD 数量，**18 decimals**） |

```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x5475611Dffb8ef4d697Ae39df9395513b6E947d7 \
  --input-data <redeem_srUSD_calldata>
```

**注意事项：**
- 无需 approve（Saving Module 直接 burn caller 地址的 srUSD）
- 赎回包含 redeemFee（通过 `SavingModule.redeemFee()` 查询，Selector: `0xf9f8bdb7` ✅）
- 赎回获得的 rUSD = srUSDAmount × currentPrice / 1e8（减去 redeemFee）

---

### 2.3 函数 Selector 汇总表

| 操作 | 合约 | 函数签名（canonical） | Selector（cast sig ✅） | ABI 参数顺序 |
|------|------|----------------------|------------------------|-------------|
| 查询余额 | rUSD/srUSD ERC-20 | `balanceOf(address)` | `0x70a08231` | `user: address` |
| ERC-20 授权 | USDC / rUSD | `approve(address,uint256)` | `0x095ea7b3` | `spender: address`, `amount: uint256` |
| 铸造 rUSD | Credit Enforcer | `mintStablecoin(uint256)` | `0xa0b4dbb1` | `amount: uint256`（USDC 6 dec） |
| 铸造 rUSD→recipient | Credit Enforcer | `mintStablecoin(address,uint256)` | `0xa7379086` | `to: address`, `amount: uint256`（6 dec） |
| 赎回 rUSD→USDC | PSM (USDC) | `redeem(uint256)` | `0xdb006a75` | `amount: uint256`（rUSD 18 dec） |
| 赎回 rUSD→USDC→addr | PSM (USDC) | `redeem(address,uint256)` | `0x1e9a6950` | `to: address`, `amount: uint256` |
| 铸造 srUSD | Credit Enforcer | `mintSavingcoin(uint256)` | `0x660cf34e` | `amount: uint256`（rUSD 18 dec） |
| 铸造 srUSD→recipient | Credit Enforcer | `mintSavingcoin(address,uint256)` | `0xb255f4e2` | `to: address`, `amount: uint256` |
| 赎回 srUSD→rUSD | Saving Module | `redeem(uint256)` | `0xdb006a75` | `amount: uint256`（srUSD 18 dec） |
| srUSD 当前汇率 | Saving Module | `currentPrice()` | `0x9d1b464a` | 无参数 |
| srUSD 铸造预览 | Saving Module | `previewMint(uint256)` | `0xb3d7f6b9` | `rUSDAmount: uint256` |
| srUSD 赎回预览 | Saving Module | `previewRedeem(uint256)` | `0x4cdad506` | `srUSDAmount: uint256` |
| PSM USDC 余额 | PSM (USDC) | `underlyingBalance()` | `0x59356c5c` | 无参数 |
| 赎回手续费 | Saving Module | `redeemFee()` | `0xf9f8bdb7` | 无参数 |
| 当前利率 | Saving Module | `currentRate()` | `0xf9f8bdb7` | 无参数 |

> 所有 Selector 均通过 `cast sig "functionName(type)"` 命令行工具验证。

---

## 3. 用户场景

### 场景 1：用 USDC 铸造 rUSD（基础稳定币获取）

**用户意图：** "我有 1000 USDC，我想铸造 rUSD 稳定币"

**前置条件：** 用户持有 USDC（Ethereum 主网）

**操作步骤：**

1. **查询 USDC 余额**（链下 eth_call）
   ```
   USDC.balanceOf(userAddress) → 确认余额 ≥ 1000 USDC
   ```

2. **授权 Credit Enforcer 消费 USDC**
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
     --input-data 0x095ea7b3
       000000000000000000000000 04716DB62C085D9e08050fcF6F7D775A03d07720
       000000000000000000000000 000000000000000000000000000000003B9ACA00
       # 3B9ACA00 = 1,000,000,000 = 1000 USDC (6 decimals)
   ```
   → 等待 3 秒（避免 nonce 碰撞）

3. **调用 mintStablecoin 铸造 rUSD**
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x04716DB62C085D9e08050fcF6F7D775A03d07720 \
     --input-data 0xa0b4dbb1
       000000000000000000000000 000000000000000000000000000000003B9ACA00
   ```
   → 成功后用户持有 1000 rUSD（1000 × 10^18）

**预期结果：** 用户获得 1000 rUSD，USDC 减少 1000

---

### 场景 2：rUSD 存入获取收益（rUSD → srUSD）

**用户意图：** "我有 1000 rUSD，我想存入获取约 7% APY 的收益"

**前置条件：** 用户持有 rUSD（Ethereum 主网）

**操作步骤：**

1. **查询当前 srUSD 汇率**（链下）
   ```
   SavingModule.currentPrice() → 例: 113561126 = 1.135 rUSD/srUSD
   SavingModule.previewMint(1000e18) → 例: 880583026272564433712 ≈ 880.58 srUSD
   ```

2. **授权 Credit Enforcer 消费 rUSD**
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x09D4214C03D01F49544C0448DBE3A27f768F2b34 \
     --input-data 0x095ea7b3
       000...04716DB62C085D9e08050fcF6F7D775A03d07720   # Credit Enforcer
       000...0000000000000000000000000000003635C9ADC5DEA00000  # 1000 rUSD (18 dec)
   ```
   → 等待 3 秒

3. **调用 mintSavingcoin 铸造 srUSD**
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x04716DB62C085D9e08050fcF6F7D775A03d07720 \
     --input-data 0x660cf34e
       000...0000000000000000000000000000003635C9ADC5DEA00000  # 1000 rUSD (18 dec)
   ```
   → 用户获得约 880.58 srUSD

**预期结果：** 用户的 rUSD 转化为 srUSD，随时间增值（价格只升不降）

---

### 场景 3：赎回 srUSD 提取收益（srUSD → rUSD → USDC）

**用户意图：** "我持有 880 srUSD，想提取回 USDC"

**前置条件：** 用户持有 srUSD（Ethereum 主网）

**操作步骤：**

1. **查询 srUSD 余额及赎回预览**（链下）
   ```
   srUSD.balanceOf(userAddress) → 880.58 srUSD
   SavingModule.previewRedeem(880.58e18) → 约 1000.x rUSD（含收益）
   ```

2. **赎回 srUSD 为 rUSD**（无需 approve，Saving Module 直接 burn）
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x5475611Dffb8ef4d697Ae39df9395513b6E947d7 \
     --input-data 0xdb006a75
       000...srUSD_amount_18dec
   ```
   → 用户获得 rUSD

3. **查询 PSM USDC 流动性**（链下，确认可赎回）
   ```
   PSM.underlyingBalance() → 确认 USDC 余额充足
   ```

4. **授权 PSM 消费 rUSD**
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x09D4214C03D01F49544C0448DBE3A27f768F2b34 \
     --input-data 0x095ea7b3
       000...4809010926aec940b550D34a46A52739f996D75D  # PSM USDC
       000...rUSD_amount_18dec
   ```

5. **PSM 赎回 rUSD 为 USDC**
   ```
   onchainos wallet contract-call \
     --chain 1 \
     --to 0x4809010926aec940b550D34a46A52739f996D75D \
     --input-data 0xdb006a75
       000...rUSD_amount_18dec
   ```
   → 用户获得等额 USDC（6 decimals）

**预期结果：** 用户获得 USDC，金额 > 初始投入（含 srUSD 收益）

---

### 场景 4：查询协议状态（只读）

**用户意图：** "查询 Reservoir Protocol 当前 srUSD 收益率和协议 USDC 流动性"

**操作（全部链下 eth_call）：**
```
1. SavingModule.currentPrice() → srUSD 对 rUSD 汇率（1e8 精度）
2. SavingModule.currentRate() → 当前年化利率
3. PSM.underlyingBalance() → PSM USDC 余额
4. rUSD.totalSupply() → rUSD 总供应量
```

**返回示例：**
- srUSD 价格：1.1356 rUSD（自创建以来增值 13.56%）
- 当前年化：约 7.75%（currentRate / 1e8）
- PSM USDC 流动性：约 521,191 USDC 可即时赎回

---

## 4. 外部 API 依赖

| 依赖项 | 类型 | 说明 |
|--------|------|------|
| Ethereum RPC | JSON-RPC | `eth_call` 用于所有链下查询。推荐使用 `https://ethereum.publicnode.com`（避免 llamarpc.com 限速） |
| onchainos wallet contract-call | CLI | 所有链上写入操作 |
| 无第三方 API | — | 协议无官方 REST API，数据直接从合约读取 |

**RPC 配置推荐：**

| Chain | RPC |
|-------|-----|
| Ethereum (1) | `https://ethereum.publicnode.com` |
| Base (8453) | `https://base.publicnode.com` 或 onchainos 默认 |
| Berachain (80094) | onchainos 默认 |

---

## 5. 配置参数

```yaml
plugin_name: reservoir-protocol
version: "0.1.0"

chains:
  - id: 1          # Ethereum mainnet（核心功能：mint/redeem/srUSD）
  - id: 8453       # Base（仅 rUSD 持有/转账，无 PSM）
  - id: 80094      # Berachain（rUSD + srUSD OFT）

contracts:
  ethereum:
    rusd: "0x09D4214C03D01F49544C0448DBE3A27f768F2b34"
    srusd: "0x738d1115B90efa71AE468F1287fc864775e23a31"
    wsrusd: "0xd3fd63209fa2d55b07a0f6db36c2f43900be3094"
    saving_module: "0x5475611Dffb8ef4d697Ae39df9395513b6E947d7"
    psm_usdc: "0x4809010926aec940b550D34a46A52739f996D75D"
    psm_usdt: "0xeaE91B4C84e1EDfA5d78dcae40962C7655A549B9"
    credit_enforcer: "0x04716DB62C085D9e08050fcF6F7D775A03d07720"
    usdc: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"

decimals:
  rusd: 18
  srusd: 18
  usdc: 6  # USDC 在 Ethereum 上为 6 decimals，mintStablecoin 参数单位!
  srusd_price_precision: 1e8  # currentPrice() 返回值精度

rpc:
  ethereum: "https://ethereum.publicnode.com"
```

---

## 6. 关键注意事项 & 已知坑

### 6.1 decimals 差异

- USDC：6 decimals；rUSD、srUSD：18 decimals
- `mintStablecoin(amount)` 的 amount 以 **USDC 的 6 decimals** 为单位
- `mintSavingcoin(amount)` 和 `PSM.redeem(amount)` 的 amount 以 **rUSD 的 18 decimals** 为单位
- 混淆 decimals 会导致铸造数量错误（差 1e12 倍）

### 6.2 Approve → TX 延迟

参考 `kb/protocols/lending.md`：approve 与主操作之间需 3 秒延迟，避免 nonce 碰撞（替换交易未确认错误）。

### 6.3 PSM 流动性限制

PSM.redeem 受 USDC 余额限制。赎回前通过 `PSM.underlyingBalance()` 检查。如 PSM 余额不足，用户可选择等待或使用 DEX 将 rUSD 兑换为 USDC。

### 6.4 Credit Enforcer 比率检查

Credit Enforcer 在铸造时检查协议的资产比率、流动性比率和净资产比率。如协议余量不足，`mintStablecoin` 或 `mintSavingcoin` 会 revert。需在 UI 提示用户。

### 6.5 srUSD redeem 手续费

Saving Module 的 `redeem` 可能收取手续费（通过 `redeemFee()` 查询）。在返回结果前展示实际到账 rUSD 数量（使用 `previewRedeem`）。

### 6.6 跨链限制

Berachain 和 Base 上的 rUSD/srUSD 是 OFT（LayerZero 跨链 token），**无 PSM 和 Credit Enforcer**。这些链上的用户只能转账或持有，无法直接铸造/赎回——需先桥接到 Ethereum 主网。

### 6.7 非传统 CDP

Reservoir 的铸造机制是 **PSM 模式（1:1 swap）**，而非超额抵押 CDP（如 MakerDAO/Liquity）。用户无需管理抵押率或担心清算。这是重要的用户教育点。

---

## 7. 实现建议

### 7.1 Skill 命令设计

```
reservoir-protocol positions        # 查询用户 rUSD/srUSD 余额及当前汇率
reservoir-protocol mint-rusd        # USDC → rUSD (approve + mintStablecoin)
reservoir-protocol redeem-rusd      # rUSD → USDC (approve + PSM.redeem)
reservoir-protocol stake            # rUSD → srUSD (approve + mintSavingcoin)
reservoir-protocol unstake          # srUSD → rUSD (SavingModule.redeem)
reservoir-protocol info             # 协议状态：APY、PSM 流动性、总供应量
```

### 7.2 测试顺序

参考 `kb/protocols/lending.md` 的测试顺序原则：
```
info → positions → mint-rusd → stake → unstake → redeem-rusd
```

### 7.3 Rust ABI 编码示例（mintStablecoin）

```rust
// mintStablecoin(uint256 amount) - amount in USDC 6 decimals
let selector = hex::decode("a0b4dbb1").unwrap();
let amount_usdc = 1000u64 * 1_000_000u64; // 1000 USDC
let amount_padded = format!("{:0>64x}", amount_usdc);
let calldata = format!("0x{}{}", hex::encode(&selector), amount_padded);

// mintSavingcoin(uint256 amount) - amount in rUSD 18 decimals
let selector = hex::decode("660cf34e").unwrap();
let amount_rusd = ethers::utils::parse_ether("1000").unwrap(); // 1000 rUSD
```

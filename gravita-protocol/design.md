# Gravita Protocol — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `gravita-protocol` |
| dapp_name | Gravita Protocol |
| category | defi-protocol |
| tags | cdp, stablecoin, borrow, vessel, ethereum, linea, lst |
| target_chains | ethereum, linea |
| target_protocols | Gravita |
| priority | P2 |
| onchainos_broadcast | 是 |

---

## 1. Background

### DApp 介绍

Gravita Protocol 是一个去中心化 CDP（Collateralized Debt Position）借贷协议，允许用户以 ETH 流动性质押代币（LST）为抵押品免息借出 GRAI 稳定币。GRAI 与美元 1:1 软锚定。

核心设计：
- **Vessel**：等同于 MakerDAO 的 Vault / Liquity 的 Trove，每个地址每种抵押品只能开一个 Vessel
- **免息借贷**：不收年化利率，一次性收取 Borrowing Fee（前端显示为 0%–最高 10%，pro-rata 退还）
- **抵押品类型**：WETH（最大 LTV 90%）、rETH（85%）、wstETH（85%）、bLUSD（99%）等 LST
- **清算机制**：ICR（个人抵押率）跌破 MCR 时触发全额清算，清算资金来自 Stability Pool
- **最低借款**：每种抵押品约 2,000 GRAI

部署链：Ethereum Mainnet、Linea（另有 Arbitrum、Mantle、Optimism、Polygon zkEVM，本期接入 Ethereum + Linea）

### 接入可行性调研

| 维度 | 结论 |
|------|------|
| 官方 SDK | 无 |
| 官方 REST API | 无 |
| 合约开源 | 是（GitHub: Gravita-Protocol/Gravita-SmartContracts） |
| 合约已审计 | 是（Dedaub 2023-04） |
| 链上操作方式 | 直接与 BorrowerOperations 合约交互 |
| 需要 ERC-20 approve | 是（开仓/还款前需 approve BorrowerOperations） |
| onchainos 广播 | 是，通过 `onchainos wallet contract-call` |

### 接入路径

无 SDK，无 REST API → **直接合约交互**：

1. 链下查询：`eth_call` 读取 VesselManager / AdminContract 状态
2. 链上操作：Rust encode calldata → `onchainos wallet contract-call --to <contract> --input-data <calldata>`
3. ERC-20 approve 也通过 `onchainos wallet contract-call` 调用 token 合约的 `approve(address,uint256)`

---

## 2. DApp 核心能力 & 接口映射

### 2.1 合约地址

#### Ethereum Mainnet (chain_id = 1)

| 合约 | 地址 | 来源 |
|------|------|------|
| BorrowerOperations | `0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2` | docs.gravitaprotocol.com/smart-contracts |
| VesselManager | `0xdB5DAcB1DFbe16326C3656a88017f0cB4ece0977` | 同上 |
| VesselManagerOperations | `0xc49B737fa56f9142974a54F6C66055468eC631d0` | 同上 |
| AdminContract | `0xf7Cc67326F9A1D057c1e4b110eF6c680B13a1f53` | 同上 |
| GRAI (DebtToken) | `0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4` | 同上 |
| FeeCollector | `0x4928c8F8c20A1E3C295DddBe05095A9aBBdB3d14` | 同上 |
| SortedVessels | `0xF31D88232F36098096d1eB69f0de48B53a1d18Ce` | 同上 |
| StabilityPool | `0x4F39F12064D83F6Dd7A2BDb0D53aF8be560356A6` | 同上 |

**Ethereum 支持的抵押品代币：**

| 代币 | 地址 | 最大 LTV |
|------|------|---------|
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | 90% |
| wstETH | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` | 85% |
| rETH | `0xae78736Cd615f374D3085123A210448E74Fc6393` | 85% |

#### Linea (chain_id = 59144)

| 合约 | 地址 | 来源 |
|------|------|------|
| BorrowerOperations | `0x40E0e274A42D9b1a9D4B64dC6c46D21228d45C20` | docs.gravitaprotocol.com/smart-contracts |
| VesselManager | `0xdC44093198ee130f92DeFed22791aa8d8df7fBfA` | 同上 |
| VesselManagerOperations | `0x53525a62e55B6002792B993a2C27Af70d12443e4` | 同上 |
| AdminContract | `0xC8a25eA0Cbd92A6F787AeED8387E04559053a9f8` | 同上 |
| GRAI (DebtToken) | `0x894134a25a5faC1c2C26F1d8fBf05111a3CB9487` | 同上 |
| FeeCollector | `0x9D8bB5496332cbeeD59f1211f28dB8b5Eb214B6D` | 同上 |
| SortedVessels | `0xF0e0915D233C616CB727E0b2Ca29ff0cbD51B66A` | 同上 |
| StabilityPool | `0x42865C7FA0b84cf76C8e8256f3356226EDC3b1be` | 同上 |

**Linea 支持的抵押品代币：**

| 代币 | 地址 | 最大 LTV |
|------|------|---------|
| wstETH | `0xB5beDd42000b71FddE22D3eE8a79Bd49A568fC8F` | 85% |

> 注：Linea 上具体支持的抵押品以 AdminContract 链上实际配置为准，运行时通过 `eth_call` 查询验证。

---

### 2.2 操作列表

#### 操作 1：查询 Vessel 状态（链下读取）

**目的**：查询用户在某个抵押品上的 Vessel 当前债务、抵押量、是否激活。

| 字段 | 值 |
|------|----|
| 合约 | VesselManager |
| 函数 | `getVesselDebt(address,address)` / `getVesselColl(address,address)` / `getVesselStatus(address,address)` / `getEntireDebtAndColl(address,address)` |
| 查询方式 | `eth_call` |

**函数签名与 Selector（cast sig 验证）：**

| 函数（canonical 格式） | Selector | 参数顺序 |
|----------------------|----------|---------|
| `getVesselDebt(address,address)` | `0x7f8da425` | `(_asset, _borrower)` |
| `getVesselColl(address,address)` | `0x41f0f4bd` | `(_asset, _borrower)` |
| `getVesselStatus(address,address)` | `0xd9721b63` | `(_asset, _borrower)` → returns uint256: 0=nonExistent, 1=active, 2=closedByOwner, 3=closedByLiquidation, 4=closedByRedemption |
| `getEntireDebtAndColl(address,address)` | `0x26f7a0d4` | `(_asset, _borrower)` → returns (debt, coll, pendingDebtReward, pendingCollReward) |
| `isVesselActive(address,address)` | `0x3670757d` | `(_asset, _borrower)` → returns bool |

**onchainos 命令（只读，不广播）：**
```bash
# 查询 Vessel 债务（Ethereum，WETH 抵押品）
onchainos wallet contract-call \
  --chain 1 \
  --to 0xdB5DAcB1DFbe16326C3656a88017f0cB4ece0977 \
  --input-data 0x7f8da425\
<asset_addr_padded_32bytes>\
<borrower_addr_padded_32bytes> \
  --dry-run
```

实现说明：`eth_call` 直接发送，返回 uint256（18 位小数，单位 wei）。

---

#### 操作 2：开仓 — 存入抵押品并借出 GRAI（链上操作）

**目的**：用户第一次为某抵押品开 Vessel，存入抵押品，借出指定数量 GRAI。

**前置条件**：
1. 用户持有足够数量的抵押品代币（如 wstETH）
2. 必须先 `approve(BorrowerOperations, assetAmount)` 抵押品代币
3. BorrowerOperations 合约未持有此用户该抵押品的活跃 Vessel（否则用 adjustVessel）

**步骤：**
1. ERC-20 approve（抵押品代币 → BorrowerOperations）
2. openVessel

**Step 1 — ERC-20 approve：**

| 字段 | 值 |
|------|----|
| 合约 | 抵押品 token 合约（如 wstETH） |
| 函数（canonical） | `approve(address,uint256)` |
| Selector（cast sig ✅） | `0x095ea7b3` |
| ABI 参数顺序 | `(spender=BorrowerOperations, amount=assetAmount)` |

```bash
# Step 1: approve wstETH → BorrowerOperations (Ethereum)
onchainos wallet contract-call \
  --chain 1 \
  --to 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0 \
  --input-data <approve_calldata>
```

**Step 2 — openVessel：**

| 字段 | 值 |
|------|----|
| 合约 | BorrowerOperations |
| 函数（canonical） | `openVessel(address,uint256,uint256,address,address)` |
| Selector（cast sig ✅） | `0xd92ff442` |
| ABI 参数顺序 | `(_asset, _assetAmount, _debtTokenAmount, _upperHint, _lowerHint)` |

参数说明：
- `_asset`：抵押品 token 地址（如 wstETH）
- `_assetAmount`：存入的抵押品数量（uint256，18 位小数）
- `_debtTokenAmount`：希望借出的 GRAI 数量（uint256，18 位小数，最低约 2000 GRAI）
- `_upperHint` / `_lowerHint`：SortedVessels 链表插入提示（可传 `address(0)` 即 `0x000...000`）

```bash
# Step 2: openVessel (Ethereum, wstETH 抵押，借 2000 GRAI)
onchainos wallet contract-call \
  --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data <openVessel_calldata>
```

> **注意**：Step 1 approve 与 Step 2 openVessel 之间须等待至少 3 秒（防止 nonce 冲突），参考 `kb/protocols/lending.md` 中的 ERC-20 approve 延迟说明。

---

#### 操作 3：还款并关仓 — repayDebtTokens + closeVessel（链上操作）

**目的**：用户全额还清 GRAI 债务并关闭 Vessel，取回所有抵押品。

**前置条件**：
1. 用户持有足够 GRAI（≥ 当前 Vessel 债务，含 gas compensation 200 GRAI）
2. 须先 approve GRAI → BorrowerOperations

**步骤（全额还款 = 直接 closeVessel，一步完成还款+关仓）：**

| 字段 | 值 |
|------|----|
| 合约 | BorrowerOperations |
| 函数（canonical） | `closeVessel(address)` |
| Selector（cast sig ✅） | `0xe687854f` |
| ABI 参数顺序 | `(_asset)` |

> `closeVessel` 内部自动 pull GRAI（从调用者地址），因此需先 approve GRAI。

**Step 1 — approve GRAI → BorrowerOperations：**

| 函数（canonical） | Selector | ABI 参数 |
|-----------------|----------|---------|
| `approve(address,uint256)` | `0x095ea7b3` | `(spender=BorrowerOperations, amount=debtAmount)` |

```bash
# Step 1: approve GRAI token → BorrowerOperations
onchainos wallet contract-call \
  --chain 1 \
  --to 0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4 \
  --input-data <approve_calldata>
```

**Step 2 — closeVessel：**

```bash
# Step 2: closeVessel (Ethereum, wstETH 抵押品)
onchainos wallet contract-call \
  --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data <closeVessel_calldata>
```

---

#### 操作 4：追加抵押品（链上操作）

**目的**：提高 Vessel 的抵押率，降低清算风险。

**前置条件**：须先 approve 抵押品代币 → BorrowerOperations

| 字段 | 值 |
|------|----|
| 合约 | BorrowerOperations |
| 函数（canonical） | `addColl(address,uint256,address,address)` |
| Selector（cast sig ✅） | `0x48a4a39d` |
| ABI 参数顺序 | `(_asset, _assetSent, _upperHint, _lowerHint)` |

```bash
# approve 抵押品 + addColl (Linea, wstETH)
# Step 1: approve
onchainos wallet contract-call \
  --chain 59144 \
  --to 0xB5beDd42000b71FddE22D3eE8a79Bd49A568fC8F \
  --input-data <approve_calldata>

# Step 2: addColl
onchainos wallet contract-call \
  --chain 59144 \
  --to 0x40E0e274A42D9b1a9D4B64dC6c46D21228d45C20 \
  --input-data <addColl_calldata>
```

---

#### 操作 5：部分还款 — repayDebtTokens（链上操作）

**目的**：用户部分还款，降低债务，无需关仓。

**前置条件**：须先 approve GRAI → BorrowerOperations

| 字段 | 值 |
|------|----|
| 合约 | BorrowerOperations |
| 函数（canonical） | `repayDebtTokens(address,uint256,address,address)` |
| Selector（cast sig ✅） | `0x7703d730` |
| ABI 参数顺序 | `(_asset, _debtTokenAmount, _upperHint, _lowerHint)` |

```bash
# Step 1: approve GRAI → BorrowerOperations
onchainos wallet contract-call \
  --chain 1 \
  --to 0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4 \
  --input-data <approve_calldata>

# Step 2: repayDebtTokens
onchainos wallet contract-call \
  --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data <repayDebtTokens_calldata>
```

---

#### 操作 6：提取抵押品 — withdrawColl（链上操作）

**目的**：在保持抵押率安全的前提下，从 Vessel 取回部分抵押品。

**无需 approve**（withdrawColl 不 pull token，而是 push 给用户）

| 字段 | 值 |
|------|----|
| 合约 | BorrowerOperations |
| 函数（canonical） | `withdrawColl(address,uint256,address,address)` |
| Selector（cast sig ✅） | `0x49b010c5` |
| ABI 参数顺序 | `(_asset, _collWithdrawal, _upperHint, _lowerHint)` |

```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data <withdrawColl_calldata>
```

---

#### 操作 7：借出更多 GRAI — withdrawDebtTokens（链上操作）

**目的**：在现有 Vessel 上增加债务，借出更多 GRAI（LTV 不能超过最大值）。

**无需 approve**（协议直接 mint GRAI 给用户）

| 字段 | 值 |
|------|----|
| 合约 | BorrowerOperations |
| 函数（canonical） | `withdrawDebtTokens(address,uint256,address,address)` |
| Selector（cast sig ✅） | `0xb5c5c9fc` |
| ABI 参数顺序 | `(_asset, _debtTokenAmount, _upperHint, _lowerHint)` |

```bash
onchainos wallet contract-call \
  --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data <withdrawDebtTokens_calldata>
```

---

#### 操作 8：查询抵押品参数（链下读取）

**目的**：获取某抵押品的 MCR、最低净债务、借款手续费率等风险参数。

| 函数（canonical） | Selector（cast sig ✅） | 合约 | 参数 | 返回 |
|-----------------|----------------------|------|------|------|
| `getMcr(address)` | `0x78aaf4de` | AdminContract | `(_collateral)` | uint256（1e18 精度，如 1.111e18 = 111.1% MCR） |
| `getMinNetDebt(address)` | `0x86d10e8c` | AdminContract | `(_collateral)` | uint256（1e18，最低净借款 GRAI） |
| `getBorrowingFee(address)` | `0x300581d9` | AdminContract | `(_collateral)` | uint256（1e18，一次性 fee 比例） |

```bash
# 查询 wstETH 的 MCR（Ethereum）
# eth_call → AdminContract.getMcr(wstETH_address)
```

---

### 2.3 全部 Selector 汇总

| 函数 | 合约 | Selector（cast sig ✅） |
|------|------|----------------------|
| `openVessel(address,uint256,uint256,address,address)` | BorrowerOperations | `0xd92ff442` |
| `adjustVessel(address,uint256,uint256,uint256,bool,address,address)` | BorrowerOperations | `0xda82ae4e` |
| `closeVessel(address)` | BorrowerOperations | `0xe687854f` |
| `addColl(address,uint256,address,address)` | BorrowerOperations | `0x48a4a39d` |
| `withdrawColl(address,uint256,address,address)` | BorrowerOperations | `0x49b010c5` |
| `withdrawDebtTokens(address,uint256,address,address)` | BorrowerOperations | `0xb5c5c9fc` |
| `repayDebtTokens(address,uint256,address,address)` | BorrowerOperations | `0x7703d730` |
| `approve(address,uint256)` | ERC-20 token | `0x095ea7b3` |
| `getVesselDebt(address,address)` | VesselManager | `0x7f8da425` |
| `getVesselColl(address,address)` | VesselManager | `0x41f0f4bd` |
| `getVesselStatus(address,address)` | VesselManager | `0xd9721b63` |
| `getEntireDebtAndColl(address,address)` | VesselManager | `0x26f7a0d4` |
| `isVesselActive(address,address)` | VesselManager | `0x3670757d` |
| `getMcr(address)` | AdminContract | `0x78aaf4de` |
| `getMinNetDebt(address)` | AdminContract | `0x86d10e8c` |
| `getBorrowingFee(address)` | AdminContract | `0x300581d9` |

---

## 3. 用户场景

### 场景 1：首次开仓——以 wstETH 抵押借 GRAI（Ethereum）

**背景**：用户持有 1 wstETH（价值约 $3,500），希望借出 2,000 GRAI，同时保持抵押率约 175%。

**操作流程：**

1. **查询 Vessel 状态**：调用 `isVesselActive(wstETH, userAddr)` 确认未开仓
2. **查询参数**：调用 AdminContract `getMcr(wstETH)` 确认 MCR，`getMinNetDebt(wstETH)` 确认 ≥ 2,000 GRAI
3. **ERC-20 approve**：`wstETH.approve(BorrowerOperations, 1e18)` → `onchainos wallet contract-call --to wstETH --input-data <approve>`
4. **等待 3 秒**
5. **开仓**：`BorrowerOperations.openVessel(wstETH, 1e18, 2000e18, 0x0, 0x0)` → `onchainos wallet contract-call --to BorrowerOperations --input-data <openVessel>`
6. **确认**：调用 `getVesselDebt(wstETH, userAddr)` 确认债务 ≈ 2200 GRAI（含 200 GRAI gas compensation）

**onchainos 命令序列：**
```bash
# approve wstETH
onchainos wallet contract-call --chain 1 \
  --to 0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0 \
  --input-data 0x095ea7b3\
000000000000000000000000 2bca0300c2aa65de6f19c2d241b54a445c9990e2\
0000000000000000000000000000000000000000000000000de0b6b3a7640000

# openVessel (3s later)
onchainos wallet contract-call --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data <openVessel_encoded_calldata>
```

---

### 场景 2：追加抵押品，提高安全性（Linea）

**背景**：用户在 Linea 的 wstETH Vessel 抵押率偏低（接近清算线），希望追加 0.5 wstETH 降低风险。

**操作流程：**

1. **查询当前状态**：`getEntireDebtAndColl(wstETH, userAddr)` 获取 (debt, coll)，计算当前 ICR
2. **approve wstETH**：`wstETH.approve(BorrowerOperations_Linea, 0.5e18)`
3. **等待 3 秒**
4. **addColl**：`BorrowerOperations.addColl(wstETH, 0.5e18, 0x0, 0x0)`
5. **验证**：`getVesselColl(wstETH, userAddr)` 确认抵押品增加

**onchainos 命令序列：**
```bash
# approve wstETH on Linea
onchainos wallet contract-call --chain 59144 \
  --to 0xB5beDd42000b71FddE22D3eE8a79Bd49A568fC8F \
  --input-data <approve_calldata>

# addColl (3s later)
onchainos wallet contract-call --chain 59144 \
  --to 0x40E0e274A42D9b1a9D4B64dC6c46D21228d45C20 \
  --input-data <addColl_calldata>
```

---

### 场景 3：全额还款并关仓，取回抵押品（Ethereum）

**背景**：用户想关闭 rETH Vessel，取回所有抵押品。当前债务 2,500 GRAI（含 200 GRAI gas compensation）。

**操作流程：**

1. **查询债务**：`getVesselDebt(rETH, userAddr)` 获取精确债务数量
2. **确认 GRAI 余额**：链下检查用户 GRAI 余额 ≥ debt
3. **approve GRAI**：`GRAI.approve(BorrowerOperations, debtAmount)`
4. **等待 3 秒**
5. **closeVessel**：`BorrowerOperations.closeVessel(rETH)`
6. **确认**：`isVesselActive(rETH, userAddr)` 返回 false

**onchainos 命令序列：**
```bash
# approve GRAI
onchainos wallet contract-call --chain 1 \
  --to 0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4 \
  --input-data <approve_calldata>

# closeVessel (3s later)
onchainos wallet contract-call --chain 1 \
  --to 0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2 \
  --input-data 0xe687854f\
000000000000000000000000ae78736cd615f374d3085123a210448e74fc6393
```

---

### 场景 4：部分还款降低 LTV（Ethereum）

**背景**：用户当前借款 5,000 GRAI，想还款 2,000 GRAI 降低 LTV，不关仓。

**操作流程：**

1. **approve GRAI**：`GRAI.approve(BorrowerOperations, 2000e18)`
2. **等待 3 秒**
3. **repayDebtTokens**：`BorrowerOperations.repayDebtTokens(wstETH, 2000e18, 0x0, 0x0)`
4. **确认**：`getVesselDebt(wstETH, userAddr)` 减少约 2,000 GRAI

---

## 4. 外部 API 依赖

| 依赖 | 类型 | 用途 | 备注 |
|------|------|------|------|
| EVM RPC（Ethereum） | JSON-RPC `eth_call` | 查询 Vessel 状态、合约参数 | 建议用 `https://ethereum.publicnode.com`（避免 llamarpc 429） |
| EVM RPC（Linea） | JSON-RPC `eth_call` | 查询 Linea 上的 Vessel 状态 | `https://rpc.linea.build` |
| onchainos wallet | CLI | 广播链上交易 | `onchainos wallet contract-call --chain <id> --to <addr> --input-data <hex>` |

**无需任何中心化 REST API。**所有协议参数均通过链上 `eth_call` 实时读取。

---

## 5. 配置参数

### plugin.yaml（建议配置）

```yaml
name: gravita-protocol
chains:
  - chain_id: 1
    rpc_url: "https://ethereum.publicnode.com"
    contracts:
      borrower_operations: "0x2bCA0300c2aa65de6F19c2d241B54a445C9990E2"
      vessel_manager: "0xdB5DAcB1DFbe16326C3656a88017f0cB4ece0977"
      admin_contract: "0xf7Cc67326F9A1D057c1e4b110eF6c680B13a1f53"
      grai_token: "0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4"
  - chain_id: 59144
    rpc_url: "https://rpc.linea.build"
    contracts:
      borrower_operations: "0x40E0e274A42D9b1a9D4B64dC6c46D21228d45C20"
      vessel_manager: "0xdC44093198ee130f92DeFed22791aa8d8df7fBfA"
      admin_contract: "0xC8a25eA0Cbd92A6F787AeED8387E04559053a9f8"
      grai_token: "0x894134a25a5faC1c2C26F1d8fBf05111a3CB9487"

supported_collateral:
  ethereum:
    - symbol: WETH
      address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      decimals: 18
      max_ltv: 0.90
    - symbol: wstETH
      address: "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0"
      decimals: 18
      max_ltv: 0.85
    - symbol: rETH
      address: "0xae78736Cd615f374D3085123A210448E74Fc6393"
      decimals: 18
      max_ltv: 0.85
  linea:
    - symbol: wstETH
      address: "0xB5beDd42000b71FddE22D3eE8a79Bd49A568fC8F"
      decimals: 18
      max_ltv: 0.85
```

### 关键参数说明

| 参数 | 值 | 说明 |
|------|-----|------|
| 借款年利率 | 0% | 无持续利息，仅一次性 fee |
| 借款一次性费用 | 0–10%（最高） | pro-rata 退还（6 个月内关仓退回剩余） |
| 最低借款 | ~2,000 GRAI | 通过 AdminContract.getMinNetDebt 链上查询 |
| Gas Compensation | 200 GRAI | 开仓时额外锁定，关仓时归还 |
| 最小抵押率（MCR） | ~111%（wstETH/rETH） / ~111%（WETH） | 通过 AdminContract.getMcr 链上查询 |
| 链上操作延迟 | approve 后等 3 秒 | 防止 nonce 冲突（同 Aave V3 模式） |

---

## 6. 已知坑 & 实现注意事项

### 6.1 approve 延迟
approve 和主操作（openVessel / repayDebtTokens / addColl）不能在同一秒广播，须间隔至少 3 秒。参考 `kb/protocols/lending.md` §ERC-20 Approval。

### 6.2 SortedVessels hint
`openVessel` / `addColl` / `withdrawColl` / `withdrawDebtTokens` / `repayDebtTokens` 均需传 `_upperHint` 和 `_lowerHint`。可以先传 `address(0)` / `address(0)` 作为简化实现（链上会做 fallback 搜索，gas 略高），后续可通过 SortedVessels 查询优化。

### 6.3 closeVessel 需要有 GRAI 余额
closeVessel 时合约自动 pull 全额债务 GRAI（包含 200 GRAI gas compensation）。若余额不足会 revert。实现时须先查 `getVesselDebt` 确认用户余额充足。

### 6.4 最低借款限制
借出 GRAI 净额须 ≥ `getMinNetDebt(_asset)` 返回值（约 2,000e18）。低于此值 openVessel / withdrawDebtTokens 会 revert。

### 6.5 withdrawColl 和 withdrawDebtTokens 无需 approve
这两个操作是 push（协议向用户转出），不 pull 用户代币，无需 approve。

### 6.6 Linea 合约地址与 Ethereum 不同
所有合约地址均不同，需按链分别配置。GRAI 在 Linea 上的地址也不同（`0x894134...` vs Ethereum 的 `0x15f744...`）。

### 6.7 selector 验证
全部 selector 已通过 `cast sig "functionName(types)"` 验证，与 Foundry cast sig 一致。若开发时遇到 `execution reverted` 且无 error data，优先检查 selector 正确性（参考 `kb/protocols/lending.md` §Compound V3）。

---

## 7. 参考资料

- 官方文档：https://docs.gravitaprotocol.com
- 合约地址页：https://docs.gravitaprotocol.com/gravita-docs/about-gravita-protocol/smart-contracts
- GitHub：https://github.com/Gravita-Protocol/Gravita-SmartContracts
- 审计报告：https://dedaub.com/audits/gravita/gravita-apr-03-2023/
- Vessel 介绍：https://docs.gravitaprotocol.com/gravita-docs/how-does-gravita-work/vessels-and-collateral
- GRAI on Etherscan：https://etherscan.io/token/0x15f74458aE0bFdAA1a96CA1aa779D715Cc1Eefe4

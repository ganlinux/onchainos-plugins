# Lista CDP — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `lista-cdp` |
| dapp_name | Lista CDP (Lista DAO) |
| category | defi-protocol |
| tags | cdp, stablecoin, liquid-staking, borrow, lisusd, slisbnb, bnb, bsc |
| target_chains | bsc |
| target_protocols | Lista DAO |
| priority | P2 |
| onchainos_broadcast | 是 |

---

## 1. Background

### DApp 介绍

Lista CDP 是 Lista DAO 在 BNB Smart Chain（BSC）上的超额抵押 CDP（Collateralized Debt Position）协议，允许用户以 BNB/slisBNB 等资产为抵押品铸造稳定币 lisUSD。协议架构模仿 MakerDAO/Hay，核心组件：

- **StakeManager**：用户存入 BNB，获得 slisBNB（Liquid Staking Token），享受 BNB 质押收益
- **Interaction**：CDP 的统一入口合约，封装 deposit / withdraw / borrow / payback
- **Vat**：MakerDAO 风格的核心 CDP 引擎，记录抵押品和债务
- **GemJoin**：抵押品适配器（每种抵押品一个实例）
- **HayJoin**：lisUSD 代币的 join/exit 适配器
- **Jug**：稳定费率管理（年化 APR）
- **Spot**：价格预言机接口
- **Jar**：lisUSD 储蓄池（类似 MakerDAO DSR）

核心参数（链上实时值，2026-04-05 查询）：
- **最大 LTV**：80%（collateralRate = 0.8e18）→ 最低抵押率约 125%
- **清算触发**：抵押率跌破 125% 时触发
- **借款年化 APR**：约 4.35%（`borrowApr()` 返回 4.354e18 / 1e20 = 4.354%）
- **最低质押 BNB**：0.001 BNB（`minBnb()` = 1e15）
- **最低借款**：~15 lisUSD（来源：官方文档；链上 dust 参数通过 Jug/Vat 查询）
- **slisBNB 兑换率**：1 BNB ≈ 0.9659 slisBNB；1 slisBNB ≈ 1.0353 BNB（含质押收益）

### 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无 |
| SDK 支持哪些技术栈？ | 仅 TypeScript（hardhat scripts，非 SDK） |
| 有 REST API？ | 无官方 REST API |
| 有官方 Skill？ | 无 |
| 开源社区有类似 Skill？ | 无已知公开 Skill |
| 支持哪些链？ | BSC Mainnet (chain_id = 56)；slisBNB OFT Adaptor 支持 Ethereum（本期只接 BSC） |
| 是否需要 onchainos 广播？ | **是**：所有写操作（stake/unstake/deposit/borrow/repay/withdraw）均需通过 `onchainos wallet contract-call` 广播 |

**协议分类确认**：Lista CDP 是**真正的超额抵押 CDP**（不是 PSM），抵押率 ≥ 125%，lisUSD 与抵押品价值挂钩，存在清算风险。

### 接入路径

无 SDK，无 REST API → **直接合约交互**：

1. 链下查询：`eth_call` 读取 Interaction / StakeManager / Vat 状态
2. 链上操作：Rust encode calldata → `onchainos wallet contract-call --chain 56 --to <contract> --input-data <calldata>`
3. ERC-20 approve（slisBNB → Interaction，lisUSD → Interaction）通过 `onchainos wallet contract-call` 调用 token.approve

---

## 2. DApp 核心能力 & 接口映射

### 2.1 合约地址

#### BSC Mainnet (chain_id = 56)

**核心 CDP 合约：**

| 合约 | 地址 | 来源 |
|------|------|------|
| Interaction | `0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4` | docs.bsc.lista.org/for-developer/cdp/smart-contract |
| Vat | `0x33A34eAB3ee892D40420507B820347b1cA2201c4` | 同上 |
| Jug | `0x787BdEaa29A253e40feB35026c3d05C18CbCA7B3` | 同上 |
| Spot | `0x49bc2c4E5B035341b7d92Da4e6B267F7426F3038` | 同上 |
| Vow | `0x2078A1969Ea581D618FDBEa2C0Dc13Fc15CB9fa7` | 同上 |
| Dog | `0xd57E7b53a1572d27A04d9c1De2c4D423f1926d0B` | 同上 |
| HayJoin (lisUSD Join) | `0x4C798F81de7736620Cd8e6510158b1fE758e22F7` | 同上 |
| Jar (lisUSD Savings) | `0x0a1Fd12F73432928C190CAF0810b3B767A59717e` | 同上 |
| AuctionProxy | `0x272d6589cecc19165cfcd0466f73a648cb1ea700` | 同上 |
| ResilientOracle | `0xf3afD82A4071f272F403dC176916141f44E6c750` | 同上 |

**Token 合约：**

| 代币 | 地址 | 说明 |
|------|------|------|
| slisBNB | `0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B` | Lista Liquid Staking BNB，18 decimals |
| lisUSD | `0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5` | Lista 稳定币，18 decimals |

**Liquid Staking 合约：**

| 合约 | 地址 | 说明 |
|------|------|------|
| StakeManager | `0x1adB950d8bB3dA4bE104211D5AB038628e477fE6` | BNB → slisBNB 质押管理 |
| slisBNBProvider | `0xfD31e1C5e5571f8E7FE318f80888C1e6da97819b` | slisBNB 抵押提供者 |

**主要抵押品 GemJoin（slisBNB CDP）：**

| 合约 | 地址 | 说明 |
|------|------|------|
| GemJoin(slisBNB) | `0x91e49983598685dd5acac90ceb4061a772f6e5ae` | slisBNB 抵押适配器 |
| Clipper(slisBNB) | `0xba92899ea8bebb717cfc60507251acbb79a3b959` | slisBNB 清算模块 |
| Oracle(slisBNB) | `0x8ecf78fb59e5a4c26cb218d34db29c4696af89f6` | slisBNB 价格预言机 |

> 注：Lista CDP 支持多种抵押品（BTCB、wstETH、USDT、solvBTC 等）。本期接入主要场景为 **slisBNB** 抵押品。其他抵押品复用相同 Interaction 合约，仅 GemJoin/token 地址不同。

---

### 2.2 操作列表

#### 操作 1：质押 BNB 获取 slisBNB（链上写操作）

**目的**：将原生 BNB 质押到 Lista StakeManager，获得 slisBNB 液态质押代币，享受质押收益，同时可用 slisBNB 作为 CDP 抵押品。

**前置条件**：
- 用户持有 ≥ 0.001 BNB（`minBnb` = 1e15）
- 有足够 BNB 支付 gas

**函数签名：**

| 字段 | 值 |
|------|----|
| 合约 | StakeManager (`0x1adB950d8bB3dA4bE104211D5AB038628e477fE6`) |
| 函数（canonical） | `deposit()` |
| Selector（cast sig ✅） | `0xd0e30db0` |
| ABI 参数 | 无（payable，通过 `--amt` 传入 BNB wei） |

```bash
# 质押 0.1 BNB → slisBNB
onchainos wallet contract-call \
  --chain 56 \
  --to 0x1adB950d8bB3dA4bE104211D5AB038628e477fE6 \
  --input-data 0xd0e30db0 \
  --amt 100000000000000000
```

**返回**：mint 等值 slisBNB 给调用者（`convertBnbToSnBnb(amount)` 换算，当前 1 BNB ≈ 0.9659 slisBNB）。

---

#### 操作 2：申请赎回 slisBNB → BNB（链上写操作，两步）

**目的**：将 slisBNB 兑换回原生 BNB。协议采用两步流程：先 requestWithdraw，等待解锁期后 claimWithdraw。

**Step 1 — requestWithdraw：**

| 字段 | 值 |
|------|----|
| 合约 | StakeManager (`0x1adB950d8bB3dA4bE104211D5AB038628e477fE6`) |
| 函数（canonical） | `requestWithdraw(uint256)` |
| Selector（cast sig ✅） | `0x745400c9` |
| ABI 参数顺序 | `(amount)` — slisBNB 数量（uint256，18 decimals） |

```bash
# 申请赎回 0.1 slisBNB（= 100000000000000000 wei）
onchainos wallet contract-call \
  --chain 56 \
  --to 0x1adB950d8bB3dA4bE104211D5AB038628e477fE6 \
  --input-data 0x745400c9\
0000000000000000000000000000000000000000000000000de0b6b3a7640000
```

**Step 2 — claimWithdraw（等待解锁期后）：**

| 字段 | 值 |
|------|----|
| 合约 | StakeManager |
| 函数（canonical） | `claimWithdraw(uint256)` |
| Selector（cast sig ✅） | `0xb13acedd` |
| ABI 参数顺序 | `(idx)` — withdraw 请求索引（uint256） |

```bash
# 领取 withdraw（idx = 0 表示第一个请求）
onchainos wallet contract-call \
  --chain 56 \
  --to 0x1adB950d8bB3dA4bE104211D5AB038628e477fE6 \
  --input-data 0xb13acedd\
0000000000000000000000000000000000000000000000000000000000000000
```

---

#### 操作 3：存入 slisBNB 抵押品（CDP deposit，链上写操作）

**目的**：将 slisBNB 存入 Lista CDP 作为抵押品，准备借出 lisUSD。

**前置条件**：
1. 用户持有足够 slisBNB
2. 必须先 approve slisBNB → Interaction

**Step 1 — ERC-20 approve slisBNB → Interaction：**

| 字段 | 值 |
|------|----|
| 合约 | slisBNB (`0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B`) |
| 函数（canonical） | `approve(address,uint256)` |
| Selector（cast sig ✅） | `0x095ea7b3` |
| ABI 参数顺序 | `(spender=Interaction, amount)` |

```bash
# Step 1: approve slisBNB → Interaction (max amount)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B \
  --input-data 0x095ea7b3\
000000000000000000000000B68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4\
ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
```

**Step 2 — deposit：**

| 字段 | 值 |
|------|----|
| 合约 | Interaction (`0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4`) |
| 函数（canonical） | `deposit(address,address,uint256)` |
| Selector（cast sig ✅） | `0x8340f549` |
| ABI 参数顺序 | `(participant, token, dink)` |

参数说明：
- `participant`：用户地址（msg.sender）
- `token`：抵押品 token 地址（slisBNB = `0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B`）
- `dink`：存入数量（uint256，18 decimals）

```bash
# Step 2: deposit 0.5 slisBNB (3秒后)
# calldata: deposit(userAddr, slisBNB, 0.5e18)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <deposit_calldata>
```

> **注意**：approve 与 deposit 之间须等待至少 3 秒，防止 nonce 冲突。参考 `kb/protocols/lending.md` ERC-20 approve 延迟说明。

---

#### 操作 4：借出 lisUSD（CDP borrow，链上写操作）

**目的**：基于已存入的 slisBNB 抵押品，借出 lisUSD 稳定币。

**前置条件**：
1. 用户已通过 `deposit` 存入足够抵押品
2. 借款数量 ≥ 15 lisUSD（最低借款阈值）
3. 借款后抵押率须 ≥ 125%（max LTV = 80%）

**无需 approve**（协议直接 mint lisUSD 给用户）

| 字段 | 值 |
|------|----|
| 合约 | Interaction (`0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4`) |
| 函数（canonical） | `borrow(address,uint256)` |
| Selector（cast sig ✅） | `0x4b8a3529` |
| ABI 参数顺序 | `(token, hayAmount)` |

参数说明：
- `token`：抵押品 token 地址（slisBNB）
- `hayAmount`：借出的 lisUSD 数量（uint256，18 decimals，最低 15e18）

```bash
# 借出 100 lisUSD
# calldata: borrow(slisBNB_addr, 100e18)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <borrow_calldata>
```

---

#### 操作 5：还款 lisUSD（CDP payback，链上写操作）

**目的**：偿还已借出的 lisUSD，减少 CDP 债务。

**前置条件**：
1. 用户持有足够 lisUSD
2. 必须先 approve lisUSD → Interaction

**Step 1 — ERC-20 approve lisUSD → Interaction：**

| 字段 | 值 |
|------|----|
| 合约 | lisUSD (`0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5`) |
| 函数（canonical） | `approve(address,uint256)` |
| Selector（cast sig ✅） | `0x095ea7b3` |
| ABI 参数顺序 | `(spender=Interaction, amount)` |

```bash
# Step 1: approve lisUSD → Interaction
onchainos wallet contract-call \
  --chain 56 \
  --to 0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5 \
  --input-data 0x095ea7b3\
000000000000000000000000B68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4\
ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
```

**Step 2 — payback：**

| 字段 | 值 |
|------|----|
| 合约 | Interaction |
| 函数（canonical） | `payback(address,uint256)` |
| Selector（cast sig ✅） | `0x35ed8ab8` |
| ABI 参数顺序 | `(token, hayAmount)` |

参数说明：
- `token`：抵押品 token 地址（slisBNB）
- `hayAmount`：还款的 lisUSD 数量（uint256，18 decimals）；全额还款时传入实际 borrowed 余额

```bash
# Step 2: payback 100 lisUSD (3秒后)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <payback_calldata>
```

---

#### 操作 6：取回抵押品（CDP withdraw，链上写操作）

**目的**：从 CDP 中取回已存入的 slisBNB 抵押品。取回后抵押率须维持 ≥ 125%（若有未还债务）或全额还款后可全部取回。

**无需 approve**（Interaction 直接 push token 给用户）

| 字段 | 值 |
|------|----|
| 合约 | Interaction (`0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4`) |
| 函数（canonical） | `withdraw(address,address,uint256)` |
| Selector（cast sig ✅） | `0xd9caed12` |
| ABI 参数顺序 | `(participant, token, dink)` |

参数说明：
- `participant`：用户地址（msg.sender）
- `token`：抵押品 token 地址（slisBNB）
- `dink`：取回数量（uint256，18 decimals）

```bash
# 取回 0.5 slisBNB
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <withdraw_calldata>
```

---

#### 操作 7：查询 CDP 仓位状态（链下读取）

**目的**：查询用户的 CDP 状态：已锁定抵押品数量、已借债务、可借额度、清算价格。

| 函数（canonical） | Selector（cast sig ✅） | 合约 | 参数 | 返回 |
|-----------------|----------------------|------|------|------|
| `locked(address,address)` | `0xdb20266f` | Interaction | `(token, usr)` | uint256：锁定的抵押品数量（18 decimals） |
| `borrowed(address,address)` | `0xb0a02abe` | Interaction | `(token, usr)` | uint256：已借 lisUSD（18 decimals） |
| `availableToBorrow(address,address)` | `0xdc7e91dd` | Interaction | `(token, usr)` | int256：还可借出的 lisUSD（18 decimals） |
| `currentLiquidationPrice(address,address)` | `0xfc085c11` | Interaction | `(token, usr)` | uint256：当前清算触发价格 |
| `collateralRate(address)` | `0x37ffefd4` | Interaction | `(token)` | uint256：最大 LTV（0.8e18 = 80%） |
| `borrowApr(address)` | `0x9c2b9b63` | Interaction | `(token)` | uint256：当前年化利率（1e20 精度） |
| `convertSnBnbToBnb(uint256)` | `0xa999d3ac` | StakeManager | `(amount)` | uint256：对应 BNB 数量 |
| `convertBnbToSnBnb(uint256)` | `0x91c3a07c` | StakeManager | `(amount)` | uint256：对应 slisBNB 数量 |

```bash
# 查询用户仓位（locked 抵押品）
# eth_call: Interaction.locked(slisBNB, userAddr)
# eth_call: Interaction.borrowed(slisBNB, userAddr)
# eth_call: Interaction.availableToBorrow(slisBNB, userAddr)
```

---

### 2.3 全部 Selector 汇总

| 函数（canonical 格式） | 合约 | Selector（cast sig ✅） |
|----------------------|------|----------------------|
| `deposit()` | StakeManager | `0xd0e30db0` |
| `requestWithdraw(uint256)` | StakeManager | `0x745400c9` |
| `claimWithdraw(uint256)` | StakeManager | `0xb13acedd` |
| `deposit(address,address,uint256)` | Interaction | `0x8340f549` |
| `withdraw(address,address,uint256)` | Interaction | `0xd9caed12` |
| `borrow(address,uint256)` | Interaction | `0x4b8a3529` |
| `payback(address,uint256)` | Interaction | `0x35ed8ab8` |
| `paybackFor(address,uint256,address)` | Interaction | `0x3b862af1` |
| `locked(address,address)` | Interaction | `0xdb20266f` |
| `borrowed(address,address)` | Interaction | `0xb0a02abe` |
| `availableToBorrow(address,address)` | Interaction | `0xdc7e91dd` |
| `currentLiquidationPrice(address,address)` | Interaction | `0xfc085c11` |
| `free(address,address)` | Interaction | `0xc5cafb88` |
| `collateralRate(address)` | Interaction | `0x37ffefd4` |
| `borrowApr(address)` | Interaction | `0x9c2b9b63` |
| `convertSnBnbToBnb(uint256)` | StakeManager | `0xa999d3ac` |
| `convertBnbToSnBnb(uint256)` | StakeManager | `0x91c3a07c` |
| `approve(address,uint256)` | ERC-20 (slisBNB / lisUSD) | `0x095ea7b3` |
| `balanceOf(address)` | ERC-20 | `0x70a08231` |
| `allowance(address,address)` | ERC-20 | `0xdd62ed3e` |

---

## 3. 用户场景

### 场景 1：质押 BNB 获取 slisBNB，再开 CDP 借出 lisUSD

**背景**：用户持有 1 BNB，希望质押获得 slisBNB 流动性质押收益，同时借出 lisUSD 用于其他 DeFi 操作。

**操作流程：**

1. **质押 BNB**：调用 StakeManager.deposit()，附带 1 BNB（1e18 wei）
   - 获得约 0.9659 slisBNB
2. **approve slisBNB → Interaction**：`slisBNB.approve(Interaction, 0.5e18)`（存入 0.5 slisBNB 作抵押）
3. **等待 3 秒**
4. **存入抵押品**：`Interaction.deposit(userAddr, slisBNB, 0.5e18)`
5. **查询可借额度**：`Interaction.availableToBorrow(slisBNB, userAddr)` — 约 0.5 × 610 USD × 80% ≈ 244 lisUSD
6. **借出 lisUSD**：`Interaction.borrow(slisBNB, 200e18)`（借 200 lisUSD，保留安全缓冲）
7. **确认**：`Interaction.borrowed(slisBNB, userAddr)` 返回 200e18，`Interaction.locked(slisBNB, userAddr)` 返回 0.5e18

**onchainos 命令序列：**

```bash
# Step 1: 质押 BNB
onchainos wallet contract-call \
  --chain 56 \
  --to 0x1adB950d8bB3dA4bE104211D5AB038628e477fE6 \
  --input-data 0xd0e30db0 \
  --amt 1000000000000000000

# Step 2: approve slisBNB → Interaction
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B \
  --input-data 0x095ea7b3\
000000000000000000000000B68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4\
00000000000000000000000000000000000000000000000006f05b59d3b20000

# Step 3: deposit slisBNB (3s later)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <deposit(userAddr, slisBNB_addr, 0.5e18)_calldata>

# Step 4: borrow 200 lisUSD
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <borrow(slisBNB_addr, 200e18)_calldata>
```

---

### 场景 2：偿还 lisUSD 债务并取回 slisBNB 抵押品

**背景**：用户有一个 slisBNB CDP，已借 200 lisUSD，现在持有足够 lisUSD，希望全额还款并取回抵押品。

**操作流程：**

1. **查询债务**：`Interaction.borrowed(slisBNB, userAddr)` — 返回 200e18 lisUSD（含已计利息）
2. **查询锁定抵押品**：`Interaction.locked(slisBNB, userAddr)` — 返回 0.5e18
3. **approve lisUSD → Interaction**：`lisUSD.approve(Interaction, borrowedAmount)` — 使用 `uint256.max` 或实际金额
4. **等待 3 秒**
5. **还款**：`Interaction.payback(slisBNB, borrowedAmount)` — 用链上查到的实际金额，不用 uint256.max（避免 revert）
6. **取回抵押品**：`Interaction.withdraw(userAddr, slisBNB, 0.5e18)`
7. **确认**：`Interaction.borrowed(slisBNB, userAddr)` = 0，slisBNB 余额恢复

**onchainos 命令序列：**

```bash
# Step 1: approve lisUSD → Interaction
onchainos wallet contract-call \
  --chain 56 \
  --to 0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5 \
  --input-data 0x095ea7b3\
000000000000000000000000B68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4\
ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff

# Step 2: payback（borrowedAmount 从链上查询）(3s later)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <payback(slisBNB_addr, borrowedAmount)_calldata>

# Step 3: withdraw 0.5 slisBNB
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <withdraw(userAddr, slisBNB_addr, 0.5e18)_calldata>
```

---

### 场景 3：追加抵押品，降低清算风险

**背景**：用户的 slisBNB CDP 抵押率接近 125% 清算线（BNB 价格下跌），希望追加 0.3 slisBNB 抵押品以提高安全缓冲。

**操作流程：**

1. **查询当前状态**：`Interaction.locked(slisBNB, userAddr)`、`Interaction.borrowed(slisBNB, userAddr)`
2. **计算清算价格**：`Interaction.currentLiquidationPrice(slisBNB, userAddr)`
3. **approve slisBNB → Interaction**：`slisBNB.approve(Interaction, 0.3e18)`
4. **等待 3 秒**
5. **追加抵押品**：`Interaction.deposit(userAddr, slisBNB, 0.3e18)`
6. **确认**：`Interaction.locked(slisBNB, userAddr)` 增加，清算价格降低

**onchainos 命令序列：**

```bash
# Step 1: approve slisBNB → Interaction
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B \
  --input-data 0x095ea7b3\
000000000000000000000000B68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4\
000000000000000000000000000000000000000000000000429d069189e00000

# Step 2: deposit 追加 0.3 slisBNB (3s later)
onchainos wallet contract-call \
  --chain 56 \
  --to 0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 \
  --input-data <deposit(userAddr, slisBNB_addr, 0.3e18)_calldata>
```

---

### 场景 4：查询 CDP 仓位信息（链下只读）

**背景**：用户想查看当前 CDP 状态：抵押品数量、债务、可借额度、清算价格、年化利率。

**操作流程（纯 eth_call，无需广播）：**

```bash
# 1. 查询锁定抵押品（slisBNB 数量）
# eth_call: Interaction.locked(slisBNB, userAddr)
# Selector: 0xdb20266f + ABI-encode(slisBNB_addr, user_addr)

# 2. 查询已借债务（lisUSD 数量）
# eth_call: Interaction.borrowed(slisBNB, userAddr)
# Selector: 0xb0a02abe

# 3. 查询还可借额度（int256，负值表示超借）
# eth_call: Interaction.availableToBorrow(slisBNB, userAddr)
# Selector: 0xdc7e91dd

# 4. 查询清算触发价格（USD with 18 decimals）
# eth_call: Interaction.currentLiquidationPrice(slisBNB, userAddr)
# Selector: 0xfc085c11

# 5. 查询年化借款利率
# eth_call: Interaction.borrowApr(slisBNB)
# Selector: 0x9c2b9b63  → 返回值 / 1e20 = APR%（当前约 4.35%）
```

---

## 4. 外部 API 依赖

| 依赖 | 类型 | 用途 | 备注 |
|------|------|------|------|
| BSC RPC | JSON-RPC `eth_call` | 查询仓位、合约参数、token 余额 | 使用 `https://bsc-rpc.publicnode.com`（沙箱中如遇 TLS 问题，fallback 到 `https://bsc-dataseed.binance.org` 或 `https://bsc.rpc.blxrbdn.com`） |
| onchainos wallet | CLI | 广播链上交易 | `onchainos wallet contract-call --chain 56 --to <addr> --input-data <hex> [--amt <wei>]` |

**无需任何中心化 REST API。** 所有协议参数均通过链上 `eth_call` 实时读取。

---

## 5. 配置参数

### plugin.yaml（建议配置）

```yaml
name: lista-cdp
chains:
  - chain_id: 56
    rpc_url: "https://bsc-rpc.publicnode.com"
    contracts:
      stake_manager: "0x1adB950d8bB3dA4bE104211D5AB038628e477fE6"
      interaction: "0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4"
      slisbnb: "0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B"
      lisusd: "0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5"
      gemjoin_slisbnb: "0x91e49983598685dd5acac90ceb4061a772f6e5ae"
      hay_join: "0x4C798F81de7736620Cd8e6510158b1fE758e22F7"
      vat: "0x33A34eAB3ee892D40420507B820347b1cA2201c4"
      jug: "0x787BdEaa29A253e40feB35026c3d05C18CbCA7B3"
parameters:
  # CDP 参数（链上实时读取，以下为当前默认值）
  max_ltv: 0.80          # collateralRate = 0.8e18
  min_cr: 1.25           # 对应 125% 最低抵押率
  min_borrow_lisusd: 15  # 最低借款（lisUSD）
  min_bnb_stake: 0.001   # 最低质押 BNB
  borrow_apr: 0.0435     # 约 4.35% 年化（从 borrowApr() 实时读取）
  dry_run: false
```

### 配置参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `rpc_url` | string | `https://bsc-rpc.publicnode.com` | BSC JSON-RPC，沙箱中如有 TLS 问题改用 `bsc-dataseed.binance.org` |
| `interaction` | address | `0xB68443Ee...` | Lista CDP Interaction 合约地址 |
| `stake_manager` | address | `0x1adB950d...` | BNB 质押管理合约 |
| `slisbnb` | address | `0xB0b84D29...` | slisBNB token 地址 |
| `lisusd` | address | `0x0782b6d8...` | lisUSD token 地址 |
| `min_borrow_lisusd` | u64 | 15 | 最低借款门槛（UI 单位，即 15e18 wei），低于此值链上会 revert |
| `dry_run` | bool | false | true 时跳过广播，仅输出 calldata；resolve_wallet 使用零地址占位符 |

### dry_run 处理规则

```rust
if dry_run {
    // 1. 跳过 onchainos wallet contract-call 广播
    // 2. resolve_wallet 返回零地址 0x0000000000000000000000000000000000000000
    //    （wallet 可能无 BSC 余额，不能 resolve）
    // 3. 仍然输出 calldata hex，让用户验证
    // 4. 跳过链上余额/allowance 检查
    return Ok(());
}
```

> **注意**：`resolve_wallet` 必须在 `if dry_run { return }` 之后调用。参考 `kb/common/resolve-wallet.md` Dry-Run Wallet Resolution Ordering。

---

## 6. 已知注意事项 & 风险

### 6.1 BSC RPC 选择

沙箱环境中 `bsc-rpc.publicnode.com` 可能遇到 TLS 问题（见 `kb/onchainos/gotchas.md`）。
**推荐 fallback 顺序：**
1. `https://bsc-rpc.publicnode.com`（首选，稳定）
2. `https://bsc-dataseed.binance.org`（稳定但偶尔连接不稳）
3. `https://bsc.rpc.blxrbdn.com`（备用）

### 6.2 resolve_wallet on BSC

BSC 测试钱包通常没有余额，`wallet balance --chain 56` 返回空 `tokenAssets`。
**必须使用 `wallet addresses` fallback**（见 `kb/common/resolve-wallet.md` Zero-Balance Wallets 章节）。

### 6.3 approve → deposit/payback 的 3 秒延迟

ERC-20 approve 之后必须等待至少 3 秒再提交 deposit/payback，避免 nonce 冲突（见 `kb/protocols/lending.md` ERC-20 Approval for Supply and Repay）。

### 6.4 payback 全额还款

全额还款时使用链上 `Interaction.borrowed(slisBNB, userAddr)` 实际值，**不使用 uint256.max**，避免利息 dust 引起 revert。

### 6.5 BSC L4 测试资金

测试钱包可能无 BNB 或 slisBNB 余额。L4 测试如遇资金不足，标记 BLOCKED 并注明原因（见 `kb/onchainos/gotchas.md` BSC Test Wallet Has No Funds）。

### 6.6 最低借款验证

`borrow` 金额 < 15 lisUSD 会在链上 revert。实现中必须在提交前校验，给出清晰错误信息。

### 6.7 deposit/withdraw 参数顺序

Interaction `deposit` 和 `withdraw` 参数顺序为 `(participant, token, dink)`，与 Gravita 等其他 CDP 协议不同（Gravita 顺序为 `(token, amount, hints)`）。注意区分。

### 6.8 Interaction 是 Proxy 合约

`0xB68443Ee...` 是 TransparentUpgradeableProxy，实现合约在 `0x7d482de9...`。所有函数调用仍发送到 proxy 地址。

---

## 7. 参考资料

| 资料 | URL |
|------|-----|
| Lista DAO 官方文档 | https://docs.bsc.lista.org/ |
| CDP Mechanics 文档 | https://docs.bsc.lista.org/for-developer/collateral-debt-position/mechanics |
| CDP Smart Contract 文档 | https://docs.bsc.lista.org/for-developer/collateral-debt-position/smart-contract |
| slisBNB Smart Contract 文档 | https://docs.bsc.lista.org/for-developer/liquid-staking-slisbnb/smart-contract |
| Lista DAO GitHub | https://github.com/lista-dao/lista-dao-contracts |
| Interaction 合约 (BSCScan) | https://bscscan.com/address/0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4 |
| StakeManager (BSCScan) | https://bscscan.com/address/0x1adB950d8bB3dA4bE104211D5AB038628e477fE6 |
| slisBNB Token (BSCScan) | https://bscscan.com/token/0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B |
| lisUSD Token (BSCScan) | https://bscscan.com/token/0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5 |
| kb/protocols/gravita-protocol.md | Gravita CDP 参考实现 |
| kb/protocols/lending.md | ERC-20 approve 延迟、repay 注意事项 |
| kb/common/resolve-wallet.md | resolve_wallet 模式（特别是零余额 BSC 场景） |
| kb/onchainos/gotchas.md | BSC RPC TLS 问题，测试钱包无资金 |

# Kamino Lend — Plugin Store 接入 PRD

> 通过 onchainos CLI 接入 Kamino Lend，使 AI Agent 能完成 Solana 上的借贷操作

---

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `kamino-lend` |
| dapp_name | Kamino Lend |
| dapp_repo | https://github.com/Kamino-Finance/klend-sdk |
| dapp_alias | kamino, klend, kamino lending |
| one_liner | Solana's leading lending protocol — supply, borrow, repay via onchainos |
| category | defi-protocol |
| tags | lending, borrowing, solana, defi, yield |
| target_chains | solana (chain ID 501) |
| target_protocols | Kamino Lend |

---

## 1. Background

### 这个 DApp 是什么

Kamino Lend（KLend）是部署在 Solana 主网上的开源点对池借贷协议，是 Kamino Finance 一体化 DeFi 产品套件的核心组件（另包含 Liquidity 和 Leverage）。协议支持：

- **Supply/Deposit**：存入资产赚取利息收益（cToken 机制）
- **Borrow**：以抵押品借出资产
- **Repay**：偿还借款
- **Withdraw**：取回存款（销毁 cToken 赎回底层资产）
- **Liquidation**：清算不健康仓位
- **Flash Loans**：单笔交易内无需抵押的闪电贷
- **Leverage/Multiply**：通过闪电贷 + KSwap 实现杠杆仓位

截至 2025 年，Kamino Lend TVL 排名 Solana 借贷协议前列（DeFiLlama Rank ~20），是 Solana 生态基础金融设施之一。

**Solana 程序地址（Mainnet）：** `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD`

**主市场地址（Main Market）：** `7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF`

**JLP 市场地址：** `DxXdAyU3kCjnyggvHmY5nAwg5cRbbmdyX3npfDMjjMek`

### 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 TypeScript SDK？ | Yes — `@kamino-finance/klend-sdk`，官方维护，GitHub: Kamino-Finance/klend-sdk |
| 有 Rust CPI SDK？ | Yes — `kamino-lending` crate（用于链上 CPI 调用） |
| SDK 支持哪些技术栈？ | TypeScript（主要），Rust（CPI），REST API（任意语言） |
| 有 REST API？ | Yes — `https://api.kamino.finance`，含链下查询 + 链上交易构建端点 |
| 有无签名交易构建 API？ | Yes — `/ktx/klend/*` 系列端点返回 base64 编码的未签名交易 |
| 有官方 Skill？ | 有社区 Skill（lobehub.com/skills/sendaifun-skills-kamino），非官方 |
| 开源社区有类似 Skill？ | 有，但覆盖度不完整，需重新开发 |
| 支持哪些链？ | Solana Mainnet（Chain ID 501）；Devnet/Staging 也有部署 |
| 是否需要 onchainos 广播？ | Yes — deposit/borrow/repay/withdraw 均需链上签名和广播 |
| 账户创建费用？ | 首次创建 Obligation 账户需要 ~0.0315 SOL，退出后返还 |

### 接入路径判定

**路径：REST API + onchainos CLI**

Kamino Lend 提供 `/ktx/klend/*` 系列交易构建端点，返回 `base64` 编码的未签名 Solana VersionedTransaction（v0 格式）。

接入流程：
1. 链下查询（市场数据、用户仓位、价格）：直接调用 `api.kamino.finance` REST API
2. 链上写操作（deposit/borrow/repay/withdraw）：
   - 调用 `POST https://api.kamino.finance/ktx/klend/{action}` 获取 `unsignedTransactionBase64`
   - 将 base64 传给 `onchainos wallet contract-call --chain 501 --unsigned-tx <base64>`

**无需直接使用 TypeScript SDK**，REST API 即可覆盖所有核心操作。TypeScript SDK 适用于需要更精细控制的高级场景（如 Multiply 仓位）。

---

## 2. DApp 核心能力 & 接口映射

### 需要接入的操作

| # | 操作 | 说明 | 链上/链下 |
|---|------|------|-----------|
| 1 | markets | 查看所有借贷市场及储备金指标 | 链下 |
| 2 | reserves-metrics | 查看储备金详情（APY、TVL、利用率等） | 链下 |
| 3 | user-obligations | 查看用户仓位（存款、借款、健康因子） | 链下 |
| 4 | deposit | 存款（提供流动性，赚取利息） | 链上 |
| 5 | withdraw | 取款（赎回存款） | 链上 |
| 6 | borrow | 借款（抵押品换流动性） | 链上 |
| 7 | repay | 还款（偿还借贷债务） | 链上 |

---

### 链下查询（API 直接调用）

#### 2.1 查看市场列表

```
GET https://api.kamino.finance/v2/kamino-market
```

无参数，返回所有市场的基础信息。

#### 2.2 查看储备金指标

```
GET https://api.kamino.finance/kamino-market/{pubkey}/reserves/metrics
```

参数：
- `pubkey`：市场地址，如主市场 `7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF`

返回：各储备金的 APY、TVL、借款利率、利用率等实时指标。

#### 2.3 查看单个储备金历史 APY

```
GET https://api.kamino.finance/kamino-market/{marketPubkey}/reserves/{reservePubkey}/borrow-and-staking-apys/history
```

#### 2.4 查看用户仓位（Obligations）

```
GET https://api.kamino.finance/kamino-market/{marketPubkey}/users/{userPubkey}/obligations
```

参数：
- `marketPubkey`：市场地址
- `userPubkey`：用户钱包地址

返回：用户所有 Obligation 账户，包含：
- 各储备金的存款金额（deposits）
- 各储备金的借款金额（borrows）
- 健康因子（health factor）= 抵押价值 / 清算阈值价值
- LTV（loan-to-value ratio）
- 借款上限（borrowLimit）

#### 2.5 查看市场历史 TVL

```
GET https://api.kamino.finance/kamino-market/{pubkey}/metrics/history
```

#### 2.6 查看 Oracle 价格

```
GET https://api.kamino.finance/oracles/prices
```

返回 Scope Oracle 价格数据，用于计算仓位价值。

#### 2.7 查看 Slot 时长

```
GET https://api.kamino.finance/slots/duration
```

返回 Solana 近期 slot 的中位时长，用于估算交易时效。

---

### 链上写操作（必须走 onchainos CLI）

所有链上写操作统一流程：

```
步骤1：调用 POST https://api.kamino.finance/ktx/klend/{action}
        请求体包含 wallet、market、reserve、amount
步骤2：从响应中提取 base64 编码的未签名交易
步骤3：执行 onchainos wallet contract-call --chain 501 --unsigned-tx <base64>
```

**重要：Solana blockhash 约 60 秒过期，获取 serializedData 后必须立即调用 onchainos 广播。**

---

#### 2.8 存款（Deposit）

**API 端点：**
```
POST https://api.kamino.finance/ktx/klend/deposit
```

**请求体（JSON）：**
```json
{
  "wallet": "<用户钱包公钥>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<储备金公钥，如 USDC reserve>",
  "amount": "<存款金额，以最小单位表示，如 USDC 的 1000000 = 1 USDC>"
}
```

**响应（包含）：**
```json
{
  "transaction": "<base64 编码的未签名 VersionedTransaction>"
}
```

**onchainos 调用：**
```bash
onchainos wallet contract-call \
  --chain 501 \
  --unsigned-tx <base64_transaction>
```

**SDK 等价方法：**
```typescript
const depositAction = await KaminoAction.buildDepositTxns(
  market,
  new BN(amount),          // 金额，最小单位
  reserveMint,             // 资产 mint 地址
  wallet,
  new VanillaObligation(PROGRAM_ID)
);
// 指令结构：computeBudgetIxs + setupIxs + lendingIxs + cleanupIxs
```

---

#### 2.9 取款（Withdraw）

**API 端点：**
```
POST https://api.kamino.finance/ktx/klend/withdraw
```

**请求体（JSON）：**
```json
{
  "wallet": "<用户钱包公钥>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<储备金公钥>",
  "amount": "<取款金额，最小单位。传 -1 或 max 表示全部取出>"
}
```

**响应：**
```json
{
  "transaction": "<base64 编码的未签名 VersionedTransaction>"
}
```

**onchainos 调用：**
```bash
onchainos wallet contract-call \
  --chain 501 \
  --unsigned-tx <base64_transaction>
```

**SDK 等价方法：**
```typescript
// 取款（赎回流动性，不销毁 cToken）
const withdrawAction = await KaminoAction.buildWithdrawTxns(...)

// 赎回 cToken（销毁 cToken 换回底层资产）
const redeemAction = await KaminoAction.buildRedeemReserveCollateralTxns(
  market,
  new BN(cTokenAmount),
  reserveMint,
  wallet,
  new VanillaObligation(PROGRAM_ID)
);
```

---

#### 2.10 借款（Borrow）

**API 端点：**
```
POST https://api.kamino.finance/ktx/klend/borrow
```

**请求体（JSON）：**
```json
{
  "wallet": "<用户钱包公钥>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<借款资产储备金公钥>",
  "amount": "<借款金额，最小单位>"
}
```

**前提条件：** 用户必须已在市场中存入足够抵押品，否则交易失败。

**响应：**
```json
{
  "transaction": "<base64 编码的未签名 VersionedTransaction>"
}
```

**onchainos 调用：**
```bash
onchainos wallet contract-call \
  --chain 501 \
  --unsigned-tx <base64_transaction>
```

**SDK 等价方法：**
```typescript
const borrowAction = await KaminoAction.buildBorrowTxns(
  market,
  new BN(1_000_000),    // 1 USDC
  usdcReserve.getLiquidityMint(),
  wallet,
  new VanillaObligation(PROGRAM_ID)
);
// 前置指令包含 RefreshReserve + RefreshObligation
```

---

#### 2.11 还款（Repay）

**API 端点：**
```
POST https://api.kamino.finance/ktx/klend/repay
```

**请求体（JSON）：**
```json
{
  "wallet": "<用户钱包公钥>",
  "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
  "reserve": "<还款资产储备金公钥>",
  "amount": "<还款金额，最小单位。传 max 表示全部还清>"
}
```

**响应：**
```json
{
  "transaction": "<base64 编码的未签名 VersionedTransaction>"
}
```

**onchainos 调用：**
```bash
onchainos wallet contract-call \
  --chain 501 \
  --unsigned-tx <base64_transaction>
```

**SDK 等价方法：**
```typescript
const repayAction = await KaminoAction.buildRepayTxns(
  market,
  repayAmount,          // BN 或 "max"
  tokenSymbol,          // 如 "USDC"
  wallet,
  new VanillaObligation(PROGRAM_ID)
);
```

---

### 补充说明：Instructions-only 端点

以上 4 个端点各有对应的 instructions-only 版本（用于更精细的交易组合）：

```
POST https://api.kamino.finance/ktx/klend/deposit-instructions
POST https://api.kamino.finance/ktx/klend/withdraw-instructions
POST https://api.kamino.finance/ktx/klend/borrow-instructions
POST https://api.kamino.finance/ktx/klend/repay-instructions
```

这些端点返回 `instructions` 数组 + `lookupTables`（地址查找表），适用于需要将多个操作打包到单一交易的场景。Plugin 开发时**优先使用非 instructions 版本**（直接返回完整交易）。

---

## 3. 用户场景

### 场景 1：查看 Kamino Lend 主市场储备金信息

**用户意图：** "查看 Kamino Lend 主市场各资产的供款和借款 APY"

**执行流程：**
1. 调用 `GET https://api.kamino.finance/kamino-market/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF/reserves/metrics`
2. 解析响应，提取各储备金的：
   - 资产符号（SOL, USDC, USDT, JLP 等）
   - 供款 APY（supply APY）
   - 借款 APY（borrow APY）
   - 总存款（total deposits）
   - 总借款（total borrows）
   - 利用率（utilization rate）
3. 以表格形式展示给用户

**工具调用：** 仅需 HTTP GET，无需 onchainos

**示例响应解析：**
```
SOL:  Supply APY 3.2%  |  Borrow APY 5.8%  |  Utilization 55%
USDC: Supply APY 8.1%  |  Borrow APY 10.3% |  Utilization 79%
USDT: Supply APY 7.9%  |  Borrow APY 9.8%  |  Utilization 76%
```

---

### 场景 2：存款到 Kamino Lend（供款赚息）

**用户意图：** "我想存入 100 USDC 到 Kamino Lend"

**执行流程：**
1. 确认用户钱包地址和 USDC 余额
2. 查询 USDC 储备金地址：`D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59`（主市场 USDC reserve）
3. 调用交易构建 API：
   ```bash
   curl -X POST https://api.kamino.finance/ktx/klend/deposit \
     -H "Content-Type: application/json" \
     -d '{
       "wallet": "<USER_WALLET>",
       "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
       "reserve": "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59",
       "amount": "100000000"
     }'
   ```
4. 从响应提取 `transaction`（base64）
5. 立即执行 onchainos 广播（blockhash ~60s 过期）：
   ```bash
   onchainos wallet contract-call \
     --chain 501 \
     --unsigned-tx <base64_transaction>
   ```
6. 等待交易确认，返回 tx hash

**预期结果：** 用户存入 100 USDC，获得等值 cUSDC（collateral token），开始赚取利息

**注意事项：**
- 首次存款可能需要创建 Obligation 账户，需要约 0.0315 SOL
- 存款后 cUSDC 作为抵押品，可用于借款

---

### 场景 3：借款（以 SOL 抵押借出 USDC）

**用户意图：** "我有 1 SOL 作为抵押，想借出 50 USDC"

**执行流程：**
1. 查询用户当前仓位：
   ```
   GET https://api.kamino.finance/kamino-market/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF/users/<USER_WALLET>/obligations
   ```
2. 检查健康因子（health factor）：
   - 如果健康因子 > 1.2，继续
   - 如果 < 1.1，警告用户风险，建议少借
   - 如果 < 1.0，拒绝操作（已触发清算线）
3. 计算借款后预估健康因子：depositValue × liquidationThreshold / (currentBorrow + newBorrow)
4. 构建借款交易：
   ```bash
   curl -X POST https://api.kamino.finance/ktx/klend/borrow \
     -H "Content-Type: application/json" \
     -d '{
       "wallet": "<USER_WALLET>",
       "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
       "reserve": "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59",
       "amount": "50000000"
     }'
   ```
5. 广播交易：
   ```bash
   onchainos wallet contract-call --chain 501 --unsigned-tx <base64_transaction>
   ```

**风险提示：** Agent 必须在执行前告知用户清算风险，建议保持健康因子 > 1.5

---

### 场景 4：查看用户仓位详情

**用户意图：** "查看我在 Kamino Lend 的当前仓位"

**执行流程：**
1. 调用用户仓位 API：
   ```
   GET https://api.kamino.finance/kamino-market/7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF/users/<USER_WALLET>/obligations
   ```
2. 解析并展示：
   - **存款（Deposits）**：资产名称、数量、USD 价值、cToken 数量
   - **借款（Borrows）**：资产名称、借款量、应计利息、USD 价值
   - **健康因子（Health Factor）**：当前值，以及清算触发阈值（通常为 1.0）
   - **净资产价值（Net Account Value）**：存款价值 - 借款价值
   - **LTV**：当前贷款价值比

**示例输出：**
```
Kamino Lend 仓位（主市场）
━━━━━━━━━━━━━━━━━━━━━━
存款：
  ├─ SOL：2.5 SOL（≈ $500）
  └─ USDC：200 USDC（≈ $200）

借款：
  └─ USDC：100 USDC（≈ $100）

健康因子：2.45（安全）
净资产：≈ $600
LTV：14.3%（清算线：80%）
```

---

### 场景 5：还款

**用户意图：** "我想还清 Kamino Lend 上的 USDC 借款"

**执行流程：**
1. 查询当前借款金额：通过 obligations API 获取待还 USDC 数量（含利息）
2. 构建还款交易（使用 `max` 参数全额还清）：
   ```bash
   curl -X POST https://api.kamino.finance/ktx/klend/repay \
     -H "Content-Type: application/json" \
     -d '{
       "wallet": "<USER_WALLET>",
       "market": "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF",
       "reserve": "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59",
       "amount": "max"
     }'
   ```
3. 广播交易：
   ```bash
   onchainos wallet contract-call --chain 501 --unsigned-tx <base64_transaction>
   ```
4. 还款成功后，可选择取回抵押品（withdraw 操作）

---

## 4. 外部 API 依赖

**Base URL：** `https://api.kamino.finance`

**认证：** 公开端点无需 API Key，高频调用有限速，超额需联系 Kamino 团队获取授权

### 数据查询端点

| 端点 | 方法 | 说明 |
|------|------|------|
| `/v2/kamino-market` | GET | 获取所有借贷市场列表 |
| `/kamino-market/{pubkey}/reserves/metrics` | GET | 获取市场所有储备金实时指标 |
| `/kamino-market/{marketPubkey}/users/{userPubkey}/obligations` | GET | 获取用户在指定市场的所有仓位 |
| `/kamino-market/{pubkey}/metrics/history` | GET | 获取市场历史 TVL 和 Obligation 统计 |
| `/kamino-market/{marketPubkey}/reserves/{reservePubkey}/borrow-and-staking-apys/history` | GET | 获取储备金历史 APY |
| `/kamino-market/{marketPubkey}/reserves/{reservePubkey}/metrics/history` | GET | 获取储备金历史指标 |
| `/kamino-market/reserves/account-data` | GET | 获取储备金账户 base64 原始数据 |
| `/oracles/prices` | GET | 获取 Scope Oracle 价格数据 |
| `/slots/duration` | GET | 获取 Solana slot 中位时长 |
| `/v2/kamino-market/{marketPubkey}/users/{userPubkey}/transactions` | GET | 获取用户在市场的交易历史 |

### 交易构建端点（返回未签名交易）

| 端点 | 方法 | 说明 |
|------|------|------|
| `/ktx/klend/deposit` | POST | 构建存款交易，返回 base64 未签名交易 |
| `/ktx/klend/withdraw` | POST | 构建取款交易，返回 base64 未签名交易 |
| `/ktx/klend/borrow` | POST | 构建借款交易，返回 base64 未签名交易 |
| `/ktx/klend/repay` | POST | 构建还款交易，返回 base64 未签名交易 |
| `/ktx/klend/deposit-instructions` | POST | 构建存款指令集（instructions + lookupTables） |
| `/ktx/klend/withdraw-instructions` | POST | 构建取款指令集 |
| `/ktx/klend/borrow-instructions` | POST | 构建借款指令集 |
| `/ktx/klend/repay-instructions` | POST | 构建还款指令集 |

### 所有 `/ktx/klend/*` 端点统一请求体格式

```json
{
  "wallet": "string",   // 用户钱包 Solana 公钥（base58）
  "market": "string",   // 市场地址（base58），默认主市场
  "reserve": "string",  // 储备金地址（base58），指定操作的资产
  "amount": "string"    // 金额，最小单位整数字符串，或 "max" 表示全部
}
```

### 常用储备金地址（主市场）

| 资产 | 储备金地址（Reserve Pubkey） | Mint 地址 |
|------|------------------------------|----------|
| USDC | `D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59` | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| SOL  | `d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q` | `So11111111111111111111111111111111111111112` |
| JLP  | `DdTmCCjv7zHRD1hJv3E8bpnSEQBzdKkzB1j9ApXX5QoP` | `27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4` |
| jupSOL | `d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q` | `jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v` |
| PYUSD | `2gc9Dm1eB6UgVYFBUN9bWks6Kes9PbWSaPaa9DqyvEiN` | `2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo` |

---

## 5. 配置参数

| Parameter | Default | Description |
|-----------|---------|-------------|
| `rpc_url` | `https://api.mainnet-beta.solana.com` | Solana RPC URL，建议使用 Helius 或 Triton 等高性能 RPC |
| `dry_run` | `true` | 模拟模式，不发真实交易（onchainos --dry-run 标志） |
| `main_market` | `7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF` | Kamino Lend 主市场地址 |
| `program_id` | `KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD` | Kamino Lend 程序 ID（Mainnet） |
| `api_base_url` | `https://api.kamino.finance` | Kamino 公共 API 基础 URL |
| `chain_id` | `501` | Solana 链 ID（onchainos 约定） |
| `health_factor_warning` | `1.5` | 健康因子告警阈值，低于此值时提示用户风险 |
| `health_factor_min` | `1.1` | 健康因子最低允许值，低于此值拒绝借款操作 |

---

## 6. 技术备注

### Solana 交易时效

- Solana blockhash 有效期约 **60 秒**（约 150 个 slot）
- 调用 `/ktx/klend/*` 获得 `base64` 交易后，**必须在 60 秒内**传给 onchainos 广播
- 不得缓存或复用同一笔未签名交易

### cToken 机制

- 存款时获得 cToken（如 cUSDC），代表在储备金池中的份额
- cToken 价值随时间因利息积累而增加（兑换率持续上涨）
- 取款时销毁 cToken，获得等值 + 利息的底层资产

### onchainos Solana 调用方式

Solana 没有 EVM 的 `calldata` 概念，整笔交易已由协议 API 序列化：

```bash
# 正确：使用 --unsigned-tx 传入完整序列化交易
onchainos wallet contract-call \
  --chain 501 \
  --unsigned-tx <base64_serialized_versioned_transaction>

# 错误：Solana 不使用 --to + --input-data 模式
```

### 首次使用 Obligation 账户

- 用户首次在市场创建仓位时，需要支付约 **0.0315 SOL** 的账户租金
- 全部退出仓位时，该 SOL 会退还给用户
- Agent 在用户第一次操作前应提示此费用

### 多市场支持

Kamino Lend 有多个独立市场：
- **主市场（Main）**：`7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF` — 支持 SOL, USDC, USDT, JLP 等
- **JLP 市场**：`DxXdAyU3kCjnyggvHmY5nAwg5cRbbmdyX3npfDMjjMek` — 专为 JLP 持有者优化
- 其他专项市场

Plugin 默认使用主市场，但应支持用户指定市场地址。

# Jito — Plugin Store 接入 PRD

> 通过 onchainos CLI 接入 Jito，使 AI Agent 能完成 JitoSOL 液态质押及 Jito Restaking 核心操作

---

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `jito` |
| dapp_name | Jito |
| dapp_repo | https://github.com/jito-foundation |
| dapp_alias | JitoSOL, Jito Liquid Staking, Jito Restaking |
| one_liner | Solana 上最大的液态质押协议，JitoSOL 持有者额外分享 MEV 奖励；同时提供 Restaking Vault 功能 |
| category | defi-protocol |
| tags | liquid-staking, restaking, MEV, solana, jitoSOL, staking |
| target_chains | solana (chain 501) |
| target_protocols | Jito Stake Pool (SPL), Jito Vault (Restaking) |

---

## 1. Background

### 这个 DApp 是什么

Jito 是 Solana 上最大的液态质押协议（TVL ~$8.9 亿 SOL，约合 8,831,609 JitoSOL 流通）。用户质押 SOL 获得 JitoSOL（LST），除标准质押收益外额外获得验证节点通过 MEV bundles 赚取的小费分成。当前 APY 约 5.62%。

Jito 还推出了 Restaking 协议，允许用户将 JitoSOL 等 SPL 代币存入 NCN（Node Consensus Network）Vault，获得 VRT（Vault Receipt Token）并赚取额外的 NCN 激励收益。

### 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 是 — `jito-vault-sdk`（Rust crate），位于 https://github.com/jito-foundation/restaking |
| SDK 支持哪些技术栈？ | Rust SDK（vault + restaking）；JS: `@jito-foundation/restaking-sdk`、`@jito-foundation/vault-sdk`；stake-pool JS: `@solana/spl-stake-pool` |
| 有 REST API？ | 部分有 — Kobe API `https://kobe.mainnet.jito.network/api/v1/` 提供 MEV 奖励、验证节点列表等只读数据；**不提供**构建 stake/unstake 序列化交易的端点 |
| onchainos defi 是否支持 Jito？ | **是** — `onchainos defi search --token sol --platform jito --chain solana` 返回 investmentId=22414；`onchainos defi invest` 直接返回可用 serializedData（base58） |
| 开源社区有类似 Skill？ | 无，需新建 |
| 支持哪些链？ | Solana mainnet only（chain 501） |
| 是否需要 onchainos 广播？ | 是 — 所有链上写操作通过 `onchainos wallet contract-call` 或 `onchainos swap execute` 执行 |

### 接入路径判定

**路径 A（已确认）：`onchainos defi invest` — Stake（SOL → JitoSOL）**

`onchainos defi invest` 已支持 Jito（investmentId=22414），直接返回 SPL Stake Pool 的 serializedData（**base58 编码，无需转换**），再传给 `wallet contract-call` 签名广播。

**路径 A（已确认）：`onchainos swap execute` — Unstake（JitoSOL → SOL，即时）**

通过 DEX（Jupiter）即时兑换，无等待期。命令：`onchainos swap execute --chain solana --from <JitoSOL_MINT> --to <SOL_MINT> --readable-amount <amount> --wallet <WALLET>`

**路径 C（Restaking）：直接构造 Vault 指令**

Jito Restaking 没有专用 REST API，Vault 操作（MintTo / EnqueueWithdrawal）需通过 Vault SDK 构造指令后传给 `wallet contract-call`。

---

## 2. DApp 核心能力 & 接口映射

### 需要接入的操作

| # | 操作 | 说明 | 链上/链下 |
|---|------|------|-----------|
| 1 | info | 获取 JitoSOL 池子信息：APY、TVL、JitoSOL 总供应量、SOL/JitoSOL 兑换率 | 链下 |
| 2 | stake | 质押 SOL → 获得 JitoSOL | 链上 (Solana, 路径A) |
| 3 | unstake | 即时兑换 JitoSOL → SOL（DEX swap，无等待期） | 链上 (Solana, swap execute) |
| 4 | positions | 查看用户的 JitoSOL 余额及当前 SOL 价值 | 链下 |
| 5 | restake-vaults | 列出 Jito Restaking Vault 列表（链上读取） | 链下 |
| 6 | restake-deposit | 存入 JitoSOL 到 Vault → 获得 VRT tokens（MintTo 指令） | 链上 (Solana, 路径C) |
| 7 | restake-withdraw | 从 Vault 发起提款（EnqueueWithdrawal 指令） | 链上 (Solana, 路径C) |

---

### 链下查询（API / 链上账户解析）

#### 操作 1：`info` — 池子基本信息

**多数据源合并**

| 字段 | 数据源 | 调用方式 | 关键响应字段 |
|------|--------|---------|------------|
| APY | onchainos defi detail | `onchainos defi detail --investment-id 22414` | `.data.baseRate`（如 `"0.05620"` = 5.62%） |
| SOL 价格 | onchainos defi detail | 同上 | `.data.aboutToken[0].price`（USD） |
| APR 描述 | onchainos defi detail | 同上 | `.data.apyDetailInfo.title`（如 `"APR 5.62%"`） |
| 当前 epoch MEV | Kobe API | `GET https://kobe.mainnet.jito.network/api/v1/mev_rewards` | `epoch`, `total_network_mev_lamports`, `mev_reward_per_lamport` |
| JitoSOL 总供应量 | Solana RPC | `getTokenSupply(J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn)` | `.result.value.uiAmount`（约 8,831,609） |
| SOL/JitoSOL 兑换率 | Solana RPC | `getAccountInfo(Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb)` | borsh 解析 `total_lamports / pool_token_supply` |

**Kobe MEV API 完整示例：**
```
GET https://kobe.mainnet.jito.network/api/v1/mev_rewards
```
响应（实测）：
```json
{
  "epoch": 950,
  "total_network_mev_lamports": 2437228348718,
  "jito_stake_weight_lamports": 411949689338669055,
  "mev_reward_per_lamport": 5.916325249888278e-6
}
```

**Solana RPC getTokenSupply 示例：**
```
POST https://api.mainnet-beta.solana.com
Content-Type: application/json
{"jsonrpc":"2.0","id":1,"method":"getTokenSupply",
 "params":["J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"]}
```
响应关键字段：`.result.value.uiAmount`（UI 单位）、`.result.value.amount`（raw）

**Solana RPC getAccountInfo（Stake Pool 兑换率）：**
```
POST https://api.mainnet-beta.solana.com
Content-Type: application/json
{"jsonrpc":"2.0","id":1,"method":"getAccountInfo",
 "params":["Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb",{"encoding":"base64"}]}
```
返回 base64 数据，需 borsh 反序列化 SPL Stake Pool 账户结构（`@solana/spl-stake-pool` 的 `StakePoolLayout.decode()`）。兑换率 = `total_lamports / pool_token_supply`。

---

#### 操作 4：`positions` — 用户 JitoSOL 余额

**首选：onchainos wallet balance**
```
onchainos wallet balance --chain 501 --token-address J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn
```
返回：JitoSOL 数量 + USD 价值

**备用：Solana RPC**
```
POST https://api.mainnet-beta.solana.com
{"jsonrpc":"2.0","id":1,"method":"getTokenAccountsByOwner",
 "params":["<USER_WALLET>",
   {"mint":"J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"},
   {"encoding":"jsonParsed"}]}
```
关键字段：`.result.value[0].account.data.parsed.info.tokenAmount.uiAmount`

---

#### 操作 5：`restake-vaults` — Restaking Vault 列表

**Solana RPC getProgramAccounts（Vault Program）**
```
POST https://api.mainnet-beta.solana.com
{"jsonrpc":"2.0","id":1,"method":"getProgramAccounts",
 "params":["Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8",
   {"encoding":"base64","dataSlice":{"offset":0,"length":0},"withContext":true}]}
```
返回约 2058 个账户（实测）；需 `jito-vault-sdk` 的 `Vault::try_from_slice` 反序列化，按 `supported_mint` 过滤出接受 JitoSOL 的 Vault。

备用：引导用户访问 https://www.jito.network/restaking/vaults/ 查看官方 Vault 列表。

---

### 链上写操作（必须走 onchainos CLI）

#### 操作 2：`stake` — SOL → JitoSOL

**接入路径：A（onchainos defi invest → wallet contract-call）**

**Step 1：获取序列化交易**
```
onchainos defi invest \
  --investment-id 22414 \
  --address <USER_WALLET> \
  --token SOL \
  --amount <LAMPORTS> \
  --chain 501
```

- `--amount`：**raw lamports，整数**（9 位小数）
  - 0.01 SOL = `10000000`
  - 0.1 SOL = `100000000`
  - 1 SOL = `1000000000`
- 返回 JSON：`.data.dataList[0].serializedData`（**base58 编码，直接使用**）
- 返回 `.data.dataList[0].to` = `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`

**Step 2：立即签名并广播（60 秒内完成）**
```
onchainos wallet contract-call \
  --chain 501 \
  --to SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy \
  --unsigned-tx <serializedData> \
  --force
```

- `--to` = SPL Stake Pool Program ID（**不是**用户地址，**不是** Stake Pool 账户地址）
- `--unsigned-tx` 接受 base58；`onchainos defi invest` 返回的 `serializedData` **已经是 base58，无需转换**
- Solana blockhash 60 秒过期，Step 1 与 Step 2 必须连续执行
- `--force` 必须加

---

#### 操作 3：`unstake` — JitoSOL → SOL（DEX 即时兑换）

**接入路径：onchainos swap execute（Jupiter DEX）**

```
onchainos swap execute \
  --chain solana \
  --from J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn \
  --to So11111111111111111111111111111111111111112 \
  --readable-amount <JITOSOL_AMOUNT> \
  --wallet <USER_WALLET>
```

- `--from` = JitoSOL Mint
- `--to` = Wrapped SOL Mint（`So11111111111111111111111111111111111111112`，注意：末尾是 `2`）
- `--readable-amount` = **UI 单位，小数**（如 `0.5`、`1.23`）
- 内部完成 quote→sign→broadcast 全流程，无需手动获取 serializedData
- **不加 `--force`**

传统 SPL Stake Pool WithdrawSol 路径（需等待 epoch，约 2-3 天）本 Plugin v1 不实现。

---

#### 操作 6：`restake-deposit` — JitoSOL → Vault（获得 VRT）

**接入路径：C（构造 Vault MintTo 指令 → wallet contract-call）**

先检查：
```
onchainos defi search --token jitosol --platform jito --chain solana
```
若有匹配产品，优先使用 `onchainos defi invest`；否则通过 Jito Vault SDK 构造 `MintTo` 指令。

```
onchainos wallet contract-call \
  --chain 501 \
  --to Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 \
  --unsigned-tx <BASE58_TX> \
  --force
```

- `--to` = Vault Program ID：`Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8`
- amount：**raw minimal units，JitoSOL 9 位小数**
- 执行成功后，用户收到 VRT token

---

#### 操作 7：`restake-withdraw` — 发起 Vault 提款（EnqueueWithdrawal）

**接入路径：C（构造 Vault EnqueueWithdrawal 指令 → wallet contract-call）**

```
onchainos wallet contract-call \
  --chain 501 \
  --to Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 \
  --unsigned-tx <BASE58_TX> \
  --force
```

- `EnqueueWithdrawal` 仅发起申请，创建 WithdrawalTicket
- 冷却期后需调用 `BurnWithdrawalTicket` 完成提款（v1 不实现，提示用户通过 https://www.jito.network/restaking/ 手动操作）
- amount：**raw VRT token 单位**

---

### 已确认地址汇总

| 名称 | 地址 | 验证方式 |
|------|------|---------|
| JitoSOL Mint | `J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn` | onchainos token info 验证 ✅ |
| SPL Stake Pool Program | `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy` | onchainos defi invest 返回 to 字段 ✅ |
| Jito Stake Pool Account | `Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb` | Solana RPC getAccountInfo 验证（owner=SPL Stake Pool Program）✅ |
| Stake Deposit Interceptor Program | `5TAiuAh3YGDbwjEruC1ZpXTJWdNDS7Ur7VeqNNiHMmGV` | GitHub jito-stake-unstake-reference ✅ |
| Jito Restaking Program | `RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q` | 官方文档 + WebSearch ✅ |
| Jito Vault Program | `Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8` | 官方文档 + RPC getProgramAccounts（2058 账户）✅ |
| Wrapped SOL Mint | `So11111111111111111111111111111111111111112` | 标准地址 ✅ |
| onchainos Jito investmentId | `22414` | onchainos defi search 返回 ✅ |
| onchainos Jito analysisPlatformId | `115713` | onchainos defi detail 返回 ✅ |
| onchainos Jito aggregateInvestmentId | `63348` | onchainos defi detail 返回 ✅ |

---

## 3. 用户场景

### 场景 1：查看 JitoSOL 质押信息

**用户说：**「Jito 现在的质押 APY 是多少？JitoSOL 的兑换率是多少？TVL 是多少？」

**Agent 动作序列：**

1. **查询 APY 和 SOL 价格：**
   ```
   onchainos defi detail --investment-id 22414
   ```
   提取 `.data.baseRate` = `"0.05620"` → APY 5.62%；`.data.aboutToken[0].price` → SOL USD 价格

2. **查询 MEV 奖励数据：**
   ```
   GET https://kobe.mainnet.jito.network/api/v1/mev_rewards
   ```
   提取 `epoch`（950）、`mev_reward_per_lamport`

3. **查询 JitoSOL 总供应量：**
   ```
   POST https://api.mainnet-beta.solana.com
   {"jsonrpc":"2.0","id":1,"method":"getTokenSupply",
    "params":["J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"]}
   ```
   提取 `.result.value.uiAmount` ≈ 8,831,609 JitoSOL

4. **返回给用户：**
   > 当前 Jito APY：5.62%（含 MEV 奖励分成）
   > JitoSOL 总供应量：约 8,831,609 JitoSOL
   > 当前 epoch：950，MEV 奖励率：5.92 × 10⁻⁶ lamports/lamport
   > SOL 价格：$XX.XX USD

---

### 场景 2：质押 SOL 获得 JitoSOL

**用户说：**「帮我质押 0.01 SOL 到 Jito」

**Agent 动作序列：**

1. **检查 SOL 余额：**
   ```
   onchainos wallet balance --chain 501
   ```
   确认余额 ≥ 0.01 SOL + gas（约 0.000005 SOL）

2. **告知预期收益（可选）：**
   ```
   onchainos defi detail --investment-id 22414
   ```
   根据 APY 5.62% 计算年化收益

3. **获取序列化交易（0.01 SOL = 10000000 lamports）：**
   ```
   onchainos defi invest \
     --investment-id 22414 \
     --address <USER_WALLET> \
     --token SOL \
     --amount 10000000 \
     --chain 501
   ```
   提取 `.data.dataList[0].serializedData`（base58）

4. **立即签名并广播（60 秒内）：**
   ```
   onchainos wallet contract-call \
     --chain 501 \
     --to SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy \
     --unsigned-tx <serializedData> \
     --force
   ```

5. **返回给用户：**
   > 质押成功！txHash: xxxx
   > 质押 0.01 SOL，预计获得约 0.00976 JitoSOL

---

### 场景 3：即时兑换 JitoSOL 为 SOL

**用户说：**「帮我把 0.5 JitoSOL 换成 SOL」

**Agent 动作序列：**

1. **检查 JitoSOL 余额：**
   ```
   onchainos wallet balance --chain 501 --token-address J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn
   ```
   确认余额 ≥ 0.5 JitoSOL

2. **获取 DEX 报价（可选）：**
   ```
   onchainos swap quote \
     --chain solana \
     --from J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn \
     --to So11111111111111111111111111111111111111112 \
     --readable-amount 0.5
   ```

3. **执行 DEX swap：**
   ```
   onchainos swap execute \
     --chain solana \
     --from J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn \
     --to So11111111111111111111111111111111111111112 \
     --readable-amount 0.5 \
     --wallet <USER_WALLET>
   ```

4. **返回给用户：**
   > 兑换成功！txHash: xxxx
   > 已将 0.5 JitoSOL 兑换为约 0.512 SOL（含 DEX 费用）

---

### 场景 4：查看用户 JitoSOL 持仓

**用户说：**「我现在有多少 JitoSOL？价值多少 SOL？」

**Agent 动作序列：**

1. **查询 JitoSOL 余额：**
   ```
   onchainos wallet balance --chain 501 --token-address J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn
   ```
   返回 JitoSOL 数量和 USD 价值

2. **（可选）查询 JitoSOL 当前 SOL 价值：**
   ```
   onchainos defi detail --investment-id 22414
   ```
   取 SOL 价格，结合 JitoSOL 数量和兑换率计算总 SOL 价值

3. **返回给用户：**
   > 您持有 X.XXX JitoSOL
   > 当前约等于 X.XXX SOL（兑换率 ~1.02X SOL/JitoSOL）
   > USD 价值：约 $XXX.XX

---

### 场景 5：列出 Restaking Vault 并存入 JitoSOL

**用户说：**「Jito Restaking 有哪些 Vault 接受 JitoSOL？帮我存 1 JitoSOL」

**Agent 动作序列：**

1. **枚举 Vault 程序账户：**
   ```
   POST https://api.mainnet-beta.solana.com
   {"jsonrpc":"2.0","id":1,"method":"getProgramAccounts",
    "params":["Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8",
      {"encoding":"base64","dataSlice":{"offset":0,"length":0},"withContext":true}]}
   ```
   获取 2058+ 个账户，解析过滤接受 JitoSOL 的 Vault

2. **展示 Vault 列表，请用户选择目标 Vault**

3. **检查 JitoSOL 余额是否 ≥ 1：**
   ```
   onchainos wallet balance --chain 501 --token-address J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn
   ```

4. **构造 Vault MintTo 序列化交易（amount = 1 JitoSOL = 1000000000 raw）：**
   通过 `jito-vault-sdk` 构造 `MintTo` 指令，序列化为 base58

5. **立即签名并广播（60 秒内）：**
   ```
   onchainos wallet contract-call \
     --chain 501 \
     --to Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8 \
     --unsigned-tx <BASE58_TX> \
     --force
   ```

6. **返回给用户：**
   > Restaking 存入成功！txHash: xxxx
   > 已存入 1 JitoSOL 到 Vault，获得 X.XXX VRT tokens

---

## 4. 外部 API 依赖

| API | Endpoint | 用途 | 需要 API Key？ | 实测可用？ |
|-----|----------|------|---------------|-----------|
| Kobe — MEV 奖励 | `GET https://kobe.mainnet.jito.network/api/v1/mev_rewards` | 当前 epoch MEV 数据 | 否 | **是** ✅ |
| Kobe — 验证节点列表 | `GET https://kobe.mainnet.jito.network/api/v1/validators?limit=N&offset=N` | 验证节点信息 | 否 | **是** ✅ |
| onchainos defi detail | `onchainos defi detail --investment-id 22414` | APY、SOL 价格、产品元数据 | onchainos 鉴权 | **是** ✅ |
| onchainos defi invest | `onchainos defi invest --investment-id 22414 ...` | 构建 stake 序列化交易（base58） | onchainos 鉴权 | **是** ✅（实测返回 serializedData） |
| onchainos swap execute | `onchainos swap execute --chain solana ...` | JitoSOL → SOL DEX swap | onchainos 鉴权 | **是** ✅ |
| Solana RPC — getTokenSupply | `POST https://api.mainnet-beta.solana.com` | JitoSOL 总供应量 | 否 | **是** ✅（实测：8831609.97 JitoSOL） |
| Solana RPC — getAccountInfo | `POST https://api.mainnet-beta.solana.com` | Stake Pool 账户数据（兑换率）、Vault 账户 | 否 | **是** ✅ |
| Solana RPC — getProgramAccounts | `POST https://api.mainnet-beta.solana.com` | 枚举 Vault 程序下所有账户 | 否 | **是** ✅（实测 2058 个账户） |
| Kobe pool/stats | `https://kobe.mainnet.jito.network/api/v1/pool/stats` | 池子汇总 | — | **否** ❌（无响应） |
| Kobe — stake deposit tx | 不存在 | — | — | **否** ❌（无此端点） |

---

## 5. 配置参数

| Parameter | Default | Description |
|-----------|---------|-------------|
| default_chain | solana | 默认操作链（chain 501） |
| max_slippage | 1.0 | DEX 兑换最大滑点 (%)，传给 swap execute 转为 `0.01` |
| dry_run | true | 模拟模式，不发真实交易 |
| unstake_method | instant | `instant`=DEX swap（即时）；`delayed`=SPL Stake Pool WithdrawSol（需等待 epoch，v1 不实现） |
| stake_investment_id | 22414 | onchainos defi 产品 ID（Jito SOL staking） |
| analysis_platform_id | 115713 | onchainos defi 平台 ID，用于 position-detail 查询 |
| kobe_base_url | https://kobe.mainnet.jito.network/api/v1 | Jito Kobe API base URL |
| solana_rpc_url | https://api.mainnet-beta.solana.com | Solana RPC 端点 |

---

## 6. Agent 执行指南

### 核心约束（必须遵守）

1. **Solana-only**：所有操作仅在 chain 501（Solana mainnet）执行
2. **onchainos 广播**：链上写操作统一通过 `onchainos wallet contract-call` 或 `onchainos swap execute` 执行，**禁止直接调用 Solana sendTransaction**
3. **blockhash 超时**：获取 serializedData 后必须在 **60 秒内**调用 contract-call，禁止中间有交互或等待
4. **必须加 `--force`**：`wallet contract-call` **必须带 `--force`** 标志
5. **amount 格式**：
   - `onchainos defi invest --amount`：**raw lamports，整数**（9 位小数）；0.01 SOL = `10000000`
   - `onchainos swap execute --readable-amount`：**UI 单位，小数**；0.5 JitoSOL = `0.5`
6. **serializedData 无需转换**：`onchainos defi invest` 返回的 `serializedData` 已是 base58，直接传入 `--unsigned-tx`

### Phase 1：需求分析（Researcher Agent）— 已完成

调研结论（全部验证完毕）：
- ✅ JitoSOL Mint：`J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn`（onchainos token info 验证）
- ✅ SPL Stake Pool Program：`SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`（defi invest 返回 to 字段）
- ✅ Jito Stake Pool Account：`Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`（Solana RPC 验证）
- ✅ `onchainos defi invest --investment-id 22414` 可用，返回 base58 serializedData
- ✅ `onchainos swap execute` 可用（JitoSOL → SOL via Jupiter）
- ✅ Kobe API `/api/v1/mev_rewards` 可用（实测）
- ✅ Vault Program ID：`Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8`（RPC getProgramAccounts 2058 账户）
- ✅ Restaking Program ID：`RestkWeAVL8fRGgzhfeoqFhsqKRchg6aa1XrcH96z4Q`
- ✅ JitoSOL 总供应量：8,831,609.97 JitoSOL（Solana RPC 实测）
- ✅ 当前 APY：5.62%；当前 epoch：950

### Phase 2：代码实现（Developer Agent）

1. 读取 Plugin Store 开发文档：https://github.com/okx/plugin-store-community/blob/main/PLUGIN_DEVELOPMENT_GUIDE_ZH.md
2. 读取 onchainos skills：https://github.com/okx/onchainos-skills/tree/main/skills
3. 创建 Rust 工程：`jito` plugin
4. 实现 CLI 子命令：`info`、`stake`、`unstake`、`positions`、`restake-vaults`、`restake-deposit`、`restake-withdraw`
5. `stake` 实现要点：
   - 调用 `onchainos defi invest --investment-id 22414 --amount <LAMPORTS> ...`
   - 提取 `.data.dataList[0].serializedData`（已是 base58）
   - 立即调用 `onchainos wallet contract-call --chain 501 --to SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy --unsigned-tx <serializedData> --force`
6. `unstake` 实现要点：
   - 调用 `onchainos swap execute --chain solana --from J1toso1... --to So111...112 --readable-amount <amount> --wallet <wallet>`
7. Cargo.toml 依赖：`reqwest`、`serde_json`；Restaking 账户解析加 `borsh`
8. **无需** `bs58` 或 `base64` 依赖（defi invest 已返回 base58）
9. 执行 `plugin-store lint` 验证

### Phase 3：测试（Tester Agent）

1. 测试顺序：`info` → `positions` → `stake` → `unstake` → `restake-vaults`
2. `stake` 测试金额：0.001 SOL（= `1000000` lamports）
3. `unstake` 测试：最小可交换单位（如 `0.001` JitoSOL）
4. `restake-deposit` / `restake-withdraw` 需真实 JitoSOL 余额
5. 链上测试在 Solana mainnet（chain 501）

### Phase 4：提交 PR

提交到 okx/plugin-store-community

---

## 7. Open Questions

- [x] **Jito 是否有 REST API 返回序列化的 stake 交易？** — **已解决**。无此端点。Kobe API 仅提供只读数据；stake 通过 `onchainos defi invest --investment-id 22414` 获取序列化交易（路径 A），已实测可用。

- [x] **`onchainos defi` 是否已支持 Jito 质押？** — **已解决**。`onchainos defi search --token sol --platform jito --chain solana` 返回 investmentId=22414，TVL=$893M，APY=5.62%，确认已支持。`onchainos defi invest` 实测返回有效 serializedData（base58）。

- [x] **Jito Restaking Vault 是否有公开 REST API？** — **已解决**。无专用 REST API。通过 Solana RPC `getProgramAccounts(Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8)` 枚举（实测 2058 个账户），用 `jito-vault-sdk` 反序列化过滤。备用：引导用户访问 https://www.jito.network/restaking/vaults/

- [x] **Jito Stake Pool Account 地址？** — **已解决**。`Jito4APyf642JPZPx3hGc6WWJ8zPKtRbRs4P815Awbb`（通过 Solana RPC getAccountInfo 验证：owner = `SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy`，space = 611 bytes）。

- [x] **serializedData 是 base64 还是 base58？** — **已解决**。`onchainos defi invest` 返回的 `serializedData` 是 **base58 编码**（887 字符，不含 +/= 特殊字符），可直接传入 `wallet contract-call --unsigned-tx`，**无需任何转换**。

- [x] **restake-withdraw 等待时间？** — **已解决**。等待时间由各 Vault 自行配置（通常 1+ Solana epoch ≈ 2-3 天）。v1 仅实现 `EnqueueWithdrawal`；`BurnWithdrawalTicket` 超出 Plugin 范围，提示用户通过 https://www.jito.network/restaking/ 手动完成。

- [x] **unstake 用哪个 onchainos 命令？** — **已解决**。使用 `onchainos swap execute`（`onchainos dex execute` 不存在）。命令：`onchainos swap execute --chain solana --from J1toso1... --to So111...112 --readable-amount <amount> --wallet <wallet>`。

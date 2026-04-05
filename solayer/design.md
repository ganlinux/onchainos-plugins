# Solayer — Plugin Store 接入 PRD

> 通过 onchainos CLI 接入 Solayer，使 AI Agent 能完成 Solana 上的原生 SOL 再质押（Restaking）操作，获得流动性再质押代币 sSOL

---

## 0. Plugin Meta

| Field | Value |
|-------|-------|
| plugin_name | `solayer` |
| dapp_name | Solayer |
| dapp_repo | https://github.com/solayer-labs/solayer-cli |
| dapp_alias | solayer restaking, sSOL, solayer ssol |
| one_liner | Solana 上的硬件加速原生 Restaking 协议，用户质押 SOL 获得 sSOL 流动性再质押代币 |
| category | defi-protocol |
| tags | restaking, liquid-staking, sSOL, Solana |
| target_chains | solana (chain ID 501) |
| target_protocols | Solayer Restaking Program, Solayer Stake Pool |

---

## 1. Background

### 这个 DApp 是什么

Solayer 是部署在 Solana 主网的硬件加速原生 Restaking 协议。用户将原生 SOL 质押到 Solayer，经过两步流程：

1. **Staking**：SOL 质押进 Solayer Stake Pool，铸造中间 LST（Stake Pool Mint `sSo1wxKKr6zW2hqf5hZrp2CawLibcwi1pMBqk5bg2G4`）
2. **Restaking**：Stake Pool Mint 再质押进 Restaking Program，铸造 sSOL（`sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh`）

sSOL 是流动性再质押代币（LRT），持有者累积 Solayer 积分，并可参与 DeFi 应用（流动性池、抵押借贷）或委托给 AVS 赚取额外收益。

退出时，用户须将 sSOL 经过 5 步 unrestake 流程换回 SOL，最终进入已停用的 stake account，等待 epoch 结束后提取原生 SOL。

### 接入可行性调研

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | No — 官方只提供 TypeScript CLI/SDK（https://github.com/solayer-labs/solayer-cli） |
| SDK 支持哪些技术栈？ | TypeScript（@coral-xyz/anchor，@solana/spl-token，@solana/spl-stake-pool） |
| 有 REST API？ | Yes — Partner Restake API（https://app.solayer.org/api/partner/restake/ssol），GET 请求，无 API Key |
| 有官方 Skill？ | No |
| 开源社区有类似 Skill？ | No |
| 支持哪些链？ | Solana mainnet only |
| 是否需要 onchainos 广播？ | Yes — restake 和 unrestake 均需链上写操作 |

### 接入路径判定

```
有 REST API？ Yes
→ restake：调用 Partner API 获取 unsigned base64 tx → 转换为 base58 → 通过 onchainos 广播
→ unrestake：构造多指令 Solana tx（Anchor Program 调用）→ 通过 onchainos 广播
→ 查询操作：直接调用 Solana JSON-RPC（getTokenAccountsByOwner、getAccountInfo 等）
```

**接入路径：API + 链上直接构造（Rust 调用 REST API，链上操作走 onchainos）**

---

## 2. DApp 核心能力 & 接口映射

### 关键合约地址（Solana Mainnet）

| 名称 | 地址 |
|------|------|
| Restaking Program ID | `sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn` |
| sSOL Mint (rstMint/LRT) | `sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh` |
| Stake Pool Mint (lstMint/中间 LST) | `sSo1wxKKr6zW2hqf5hZrp2CawLibcwi1pMBqk5bg2G4` |
| Stake Pool Program | `po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2` |
| Pool Address | `3sk58CzpitB9jsnVzZWwqeCn2zcXVherhALBh88Uw9GQ` |
| Solayer Admin Signer | `so1MFdbL7gd8mraypNEEQeroQYqTKtS7pZCN4H46BPa` |
| Stake Pool Validator List | `nk5E1Gc2rCuU2MDTRqdcQdiMfV9KnZ6JHykA1cTJQ56` |
| Stake Pool Withdraw Authority | `H5rmot8ejBUWzMPt6E44h27xj5obbSz3jVuK4AsJpHmv` |
| Stake Pool Validator Stake | `CpWqBteUJodiTcGYWsxq4WTaBPoZJyKkBbkWwAMXSyTK` |
| Stake Pool Manager Fee Account | `ARs3HTD79nsaUdDKqfGhgbNMVJkXVdRs2EpHAm4LNEcq` |

### 需要接入的操作

| # | 操作 | 说明 | 链上/链下 |
|---|------|------|-----------|
| 1 | restake | 质押 SOL 获得 sSOL | 链上 (Solana) |
| 2 | unrestake | 解质押 sSOL 换回 SOL（stake account，需等待 epoch） | 链上 (Solana) |
| 3 | get-balance | 查询用户 sSOL 余额及原生 SOL 余额 | 链下 |
| 4 | get-positions | 查询用户 sSOL 仓位及当前换算 SOL 价值 | 链下 |

---

### 链下查询（Solana JSON-RPC 直接调用）

#### 操作 3：get-balance

**目标**：查询用户钱包中的 SOL 余额和 sSOL 代币余额。

**步骤 1：查询原生 SOL 余额**

```
onchainos wallet balance --chain 501
```

**步骤 2：查询 sSOL 代币余额（Solana JSON-RPC）**

```
POST https://mainnet-rpc.solayer.org
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getTokenAccountsByOwner",
  "params": [
    "<WALLET_PUBKEY>",
    { "mint": "sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh" },
    { "encoding": "jsonParsed" }
  ]
}
```

返回 `value[0].account.data.parsed.info.tokenAmount.uiAmount` 即为 sSOL 数量（UI 单位，9 位精度）。

---

#### 操作 4：get-positions

**目标**：查询用户 sSOL 持仓及当前 SOL 价值（基于 sSOL/SOL 兑换率）。

**步骤 1：查询用户 sSOL 余额**（同 get-balance 步骤 2）

**步骤 2：查询 Stake Pool 兑换率**

```
POST https://mainnet-rpc.solayer.org
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getAccountInfo",
  "params": [
    "po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2",
    { "encoding": "jsonParsed" }
  ]
}
```

兑换率计算：`exchange_rate = total_lamports / pool_token_supply`（即 1 sSOL 对应的 SOL 数量，约 1.0148 SOL）

**步骤 3：计算仓位价值**

```
position_value_sol = ssol_balance * exchange_rate
```

---

### 链上写操作（必须走 onchainos CLI）

> 硬性要求：所有链上交易（签名、广播）必须通过 onchainos 执行。
> Solana onchainos 命令不加 `--output json`。

#### 操作 1：restake（SOL → sSOL）

**流程**：调用 Partner Restake API 获取 unsigned base64 transaction → 转换为 base58 → 通过 onchainos 广播签名。

**Step 1：调用 Partner Restake API**

```
GET https://app.solayer.org/api/partner/restake/ssol
  ?staker=<WALLET_PUBKEY>
  &amount=<AMOUNT_IN_SOL>
  &referrerkey=<REFERRER_PUBKEY>
```

**关键参数说明**：
- `amount`：**UI 单位（SOL）**，例如质押 0.5 SOL 传入 `"0.5"`。注意：不是 lamports，不是 `500000000`。源码注释明确：`amount - the amount of native SOL (in SOL) that the user will stake`
- `staker`：用户钱包公钥
- `referrerkey`：partner 钱包地址（可选，用于追踪推荐量）

**API 响应**：

```json
{
  "transaction": "<BASE64_ENCODED_SERIALIZED_TRANSACTION>",
  "message": "restaking 0.5 SOL for 0.4927 sSOL"
}
```

**Step 2：base64 → base58 转换**

API 返回的是 base64 编码的序列化 VersionedTransaction，onchainos 接受 base58 格式的 unsigned tx。代码中必须执行转换：

```rust
// 伪代码示意
let tx_bytes = base64::decode(api_response.transaction)?;
let tx_base58 = bs58::encode(tx_bytes).into_string();
```

**Step 3：onchainos 广播**

```bash
onchainos wallet contract-call \
  --chain 501 \
  --to sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn \
  --unsigned-tx <BASE58_TX> \
  --force
```

---

#### 操作 2：unrestake（sSOL → SOL）

**流程**：构造包含 5 个指令的 Solana 事务，通过 onchainos 广播。

**5 步指令序列**：

| 步骤 | 指令类型 | 描述 |
|------|---------|------|
| 1 | `restakeProgram.methods.unrestake(amount_u64)` | 调用 Restaking Program unrestake 方法：销毁 sSOL，释放 Stake Pool Mint（LST）到用户 lstAta |
| 2 | `createApproveInstruction(lstAta, feePayer, feePayer, amount_u64)` | Approve 指令：授权从 lstAta 转出 LST，供后续 withdrawStake 使用 |
| 3 | `SystemProgram.createAccount(...)` | 创建新 stake account（空间 StakeProgram.space，租金 2282880 lamports） |
| 4 | `StakePoolInstruction.withdrawStake(...)` | 从 Stake Pool 提取 stake 到新创建的 stake account |
| 5 | `StakeProgram.deactivate(stakeAccount, feePayer)` | 停用 stake account，等待 epoch 结束后可取出原生 SOL |

**关键账户**：

```
// unrestake 指令所需账户
signer:                  <USER_WALLET>
lstMint:                 sSo1wxKKr6zW2hqf5hZrp2CawLibcwi1pMBqk5bg2G4
rstMint:                 sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh
solayerSigner:           so1MFdbL7gd8mraypNEEQeroQYqTKtS7pZCN4H46BPa
pool:                    3sk58CzpitB9jsnVzZWwqeCn2zcXVherhALBh88Uw9GQ
vault (lstVault):        getAssociatedTokenAddress(STAKE_POOL_MINT, POOL_ADDRESS, allowOffCurve=true)
lstAta:                  getAssociatedTokenAddress(STAKE_POOL_MINT, USER_WALLET)
rstAta:                  getAssociatedTokenAddress(SSOL_MINT, USER_WALLET, allowOffCurve=true)

// withdrawStake 指令所需账户
stakePool:               po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2
validatorList:           nk5E1Gc2rCuU2MDTRqdcQdiMfV9KnZ6JHykA1cTJQ56
withdrawAuthority:       H5rmot8ejBUWzMPt6E44h27xj5obbSz3jVuK4AsJpHmv
validatorStake:          CpWqBteUJodiTcGYWsxq4WTaBPoZJyKkBbkWwAMXSyTK
managerFeeAccount:       ARs3HTD79nsaUdDKqfGhgbNMVJkXVdRs2EpHAm4LNEcq
```

**amount 参数**：`amount` 在链上合约中为 `u64`，单位为 lamports（9位精度），即 1 SOL = `1_000_000_000`。例如解质押 1 sSOL 传入 `1000000000`。

**构造完整 tx 后通过 onchainos 广播**：

```bash
onchainos wallet contract-call \
  --chain 501 \
  --to sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn \
  --unsigned-tx <BASE58_TX> \
  --force
```

注意：tx 中包含 ComputeBudget 指令（units: 500000, microLamports: 200000），以及新生成的 stakeAccount keypair 作为额外签名者。

---

## 3. 用户场景

### 场景 1：质押 SOL 获得 sSOL

**用户说**："帮我把 1 SOL 质押到 Solayer，获得 sSOL"

**Agent 动作序列**：

1. **[链下 — 查余额]** 确认用户 SOL 余额足够：
   ```
   onchainos wallet balance --chain 501
   ```
   验证余额 > 1 SOL + gas fees（约 0.01 SOL）。

2. **[链下 — API 调用]** 调用 Partner Restake API 获取 unsigned transaction：
   ```
   GET https://app.solayer.org/api/partner/restake/ssol
     ?staker=<USER_WALLET>&amount=1&referrerkey=<REFERRER>
   ```
   解析响应，提取 `transaction`（base64 格式），以及 `message` 中的预期 sSOL 数量（如 "restaking 1 SOL for 0.9854 sSOL"）。

3. **[链下 — 编码转换]** 将 base64 编码的 tx 转换为 base58：
   ```rust
   let tx_base58 = bs58::encode(base64::decode(response.transaction)?).into_string();
   ```

4. **[链上 — onchainos]** 广播并签名交易：
   ```
   onchainos wallet contract-call \
     --chain 501 \
     --to sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn \
     --unsigned-tx <BASE58_TX> \
     --force
   ```

5. **[链下 — 确认]** 交易确认后，查询 sSOL 余额验证到账：
   ```
   POST https://mainnet-rpc.solayer.org → getTokenAccountsByOwner
   ```

**Agent 回复**："已成功将 1 SOL 质押到 Solayer，获得约 0.9854 sSOL。交易哈希：`<TX_HASH>`。sSOL 可在 Solayer 生态中获取积分奖励，也可委托给 AVS 赚取额外收益。"

---

### 场景 2：查询当前 sSOL 仓位

**用户说**："查看我在 Solayer 的质押仓位"

**Agent 动作序列**：

1. **[链下 — 查询 sSOL 余额]** 通过 Solana RPC 查询用户 sSOL 余额：
   ```
   POST https://mainnet-rpc.solayer.org
   {
     "method": "getTokenAccountsByOwner",
     "params": ["<USER_WALLET>", {"mint": "sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh"}, {"encoding": "jsonParsed"}]
   }
   ```
   提取 `uiAmount`，例如 `2.4537` sSOL。

2. **[链下 — 查询兑换率]** 获取 Stake Pool 账户数据计算 sSOL/SOL 兑换率：
   ```
   POST https://mainnet-rpc.solayer.org
   {
     "method": "getAccountInfo",
     "params": ["po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2", {"encoding": "jsonParsed"}]
   }
   ```
   解析 `total_lamports` 和 `pool_token_supply`，计算 `exchange_rate = total_lamports / pool_token_supply`（例如 1 sSOL ≈ 1.0148 SOL）。

3. **[链下 — 计算]** 计算仓位 SOL 价值：
   ```
   position_sol = 2.4537 * 1.0148 ≈ 2.4900 SOL
   ```

**Agent 回复**："您当前在 Solayer 的质押仓位：
- sSOL 余额：2.4537 sSOL
- 当前兑换率：1 sSOL ≈ 1.0148 SOL
- 仓位价值：约 2.4900 SOL
- 收益（相对本金）：+1.48%"

---

### 场景 3：解质押 sSOL 取回 SOL

**用户说**："我想把 1 sSOL 解质押，取回 SOL"

**Agent 动作序列**：

1. **[链下 — 查余额]** 确认用户 sSOL 余额充足（≥ 1 sSOL）：
   ```
   POST https://mainnet-rpc.solayer.org → getTokenAccountsByOwner(sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh)
   ```

2. **[链下 — 说明等待期]** 告知用户：unrestake 后 SOL 会进入已停用的 stake account，需等待当前 epoch 结束（通常 2-3 天）才能提取原生 SOL。

3. **[链下 — 构造 tx]** 构造包含 5 个指令的 Solana 事务：
   - 指令 1：调用 Restaking Program `unrestake(1_000_000_000)` 方法
   - 指令 2：`createApproveInstruction` 授权 lstAta
   - 指令 3：`SystemProgram.createAccount` 创建新 stake account
   - 指令 4：`StakePoolInstruction.withdrawStake` 从 Stake Pool 提取
   - 指令 5：`StakeProgram.deactivate` 停用 stake account

4. **[链上 — onchainos]** 序列化 tx 并转为 base58，通过 onchainos 广播：
   ```
   onchainos wallet contract-call \
     --chain 501 \
     --to sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn \
     --unsigned-tx <BASE58_TX> \
     --force
   ```

5. **[链下 — 确认结果]** 解析交易结果，告知用户 stake account 地址及预计可提取时间。

**Agent 回复**："已成功发起 1 sSOL 解质押请求。SOL 已存入 stake account `<STAKE_ACCOUNT_PUBKEY>`，将在当前 epoch 结束后（约 2-3 天）可提取为原生 SOL。届时您需要运行提款指令将 SOL 转入钱包。"

---

### 场景 4：查询 SOL 和 sSOL 双余额

**用户说**："查一下我现在钱包里有多少 SOL 和 sSOL"

**Agent 动作序列**：

1. **[链下 — 查 SOL 余额]**：
   ```
   onchainos wallet balance --chain 501
   ```

2. **[链下 — 查 sSOL 余额]**：
   ```
   POST https://mainnet-rpc.solayer.org → getTokenAccountsByOwner(sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh)
   ```

**Agent 回复**："您的当前余额：
- SOL：3.2100 SOL
- sSOL：2.4537 sSOL（约合 2.4900 SOL）"

---

## 4. 外部 API 依赖

| API | Base URL | 用途 | 需要 API Key？ |
|-----|----------|------|--------------|
| Partner Restake API | `https://app.solayer.org` | GET `/api/partner/restake/ssol` — 获取 restake unsigned tx，参数：staker, amount(SOL), referrerkey | No |
| Solayer RPC | `https://mainnet-rpc.solayer.org` | Solana JSON-RPC，用于查询账户余额（getTokenAccountsByOwner）、Stake Pool 信息（getAccountInfo） | No |
| Solana Mainnet RPC (备用) | `https://api.mainnet-beta.solana.com` | 备用 Solana RPC，用于查询账户数据、获取最新 blockhash | No |

---

## 5. 配置参数

| 参数名 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `dry_run` | bool | `false` | 为 true 时仅模拟执行，不广播链上交易（打印将要执行的 onchainos 命令） |
| `default_rpc` | string | `"https://mainnet-rpc.solayer.org"` | 默认使用的 Solana JSON-RPC 节点地址 |
| `referrer_key` | string | `""` | Partner 钱包地址，用于 restake API 追踪推荐量（可选） |
| `compute_unit_limit` | u32 | `500000` | unrestake 交易的 ComputeBudget 单元上限 |
| `compute_unit_price` | u64 | `200000` | unrestake 交易的优先费（microLamports） |

---

## 6. 技术注意事项

### amount 单位对照表

| 操作 | amount 单位 | 示例 |
|------|------------|------|
| restake API (`amount` 参数) | **SOL（UI 单位）** | 质押 0.5 SOL → `amount=0.5` |
| unrestake 合约调用（`u64` 参数） | **lamports（链上原子单位）** | 解质押 1 sSOL → `1000000000` |

### base64 → base58 转换（必须）

Partner API 返回 base64 编码的 VersionedTransaction，onchainos 接受 base58 格式。插件代码中必须完成转换：

```rust
// Rust 示意（使用 base64 + bs58 crate）
let tx_bytes = base64::engine::general_purpose::STANDARD.decode(&api_response.transaction)?;
let tx_base58 = bs58::encode(&tx_bytes).into_string();
// 然后传给 onchainos --unsigned-tx
```

### unrestake 需要额外 keypair 签名

unrestake 流程中需生成一个新的 stakeAccount Keypair（`web3.Keypair.generate()`），该 keypair 必须与用户一起作为签名者。需在构造 tx 时预签或在 onchainos 执行时处理。

### unrestake 后的 SOL 提取

unrestake 完成后，SOL 在已停用的 stake account 中，用户需等待 epoch 结束后调用 `StakeProgram.withdraw` 将 SOL 提取到钱包。这是第二步操作，超出本次接入范围，但应在 UI 中明确告知用户。

### Stake Pool 兑换率来源

通过 `spl-stake-pool` 协议解析 Stake Pool 账户（`po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2`）：

```
exchange_rate = pool.total_lamports / pool.pool_token_supply
```

例：total_lamports = 10,148,000,000,000，pool_token_supply = 10,000,000,000,000 → 1 sSOL = 1.0148 SOL

---

## 7. IDL 参考

- Restaking Program IDL: https://github.com/solayer-labs/solayer-cli/blob/main/restaking/utils/restaking_program.json
- 关键方法：`restake(amount: u64)`，`unrestake(amount: u64)`
- 参考实现：
  - Restake: https://github.com/solayer-labs/solayer-cli/blob/main/restaking/actions/partner_restake_ssol.ts
  - Unrestake: https://github.com/solayer-labs/solayer-cli/blob/main/restaking/actions/unrestake_ssol.ts

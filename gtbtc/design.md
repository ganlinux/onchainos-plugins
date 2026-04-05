# GTBTC Plugin — Design Document

## §0 Plugin Meta

| 字段 | 值 |
|------|-----|
| plugin_name | gtbtc |
| dapp_name | Gate Wrapped BTC (GTBTC) |
| dapp_url | https://www.gate.com/proof-of-reserve/gtbtc |
| dapp_staking_url | https://www.gate.com/staking/BTC |
| category | BTC Wrapped Token / Yield-Bearing BTC |
| rank | #146 (P1) |
| supported_chains | Ethereum (1), BNB Smart Chain (56), Base (8453), Solana |
| plugin_version | 0.1.0 |
| author | TBD |

### 协议简介

Gate Wrapped BTC (GTBTC) 是由 Gate Web3 发行的链上 BTC 收益凭证。用户在 Gate 平台质押 BTC 后按 1:1 比例铸造 GTBTC，收益以净资产增长形式自动累积（类似 rETH/stETH 的 rebasing 机制），GTBTC 随时间相对 BTC 升值。GTBTC 作为标准 ERC-20/SPL 代币，可在链上进行转账、DEX 交易、抵押借贷等 DeFi 操作。

**关键架构特点：**
- GTBTC 是纯粹的 **ERC-20（EVM）/ SPL（Solana）代币**，无独立 DeFi 合约（不是 Uniswap/Curve 池，不是借贷协议）
- 铸造（BTC → GTBTC）和赎回（GTBTC → BTC）**仅通过 Gate 中心化平台**完成，无链上 mint/redeem 合约
- 链上操作 = **ERC-20 标准操作**（transfer、approve）+ DEX swap（通过第三方 DEX）
- 价格/余额查询通过 Gate API v4 或 DeFiLlama

---

## §1 接入可行性调研

### 调研表

| 检查项 | 结果 |
|--------|------|
| 有 Rust SDK？ | 无 GTBTC 专属 Rust SDK。Gate.io 官方仅提供 Python/Go/JS SDK（gateapi-python, gateapi-go, gateapi-js）|
| SDK 支持哪些技术栈？ | Python, Go, JavaScript/TypeScript, C# (NuGet). 无 Rust |
| 有 REST API？ | **有** — Gate API v4: `https://api.gateio.ws/api/v4`。Earn/Staking 端点 + Spot 端点可查询 GTBTC 价格、余额、APR |
| 有官方 Skill？ | 无 Gate 官方 plugin-store skill |
| 开源社区有类似 Skill？ | 无已知类似 Skill。Solv SolvBTC (PR #181) 是同类 BTC 收益凭证，可作为参考架构 |
| 支持哪些链？ | Ethereum (1), BNB Smart Chain (56), Base (8453), Solana, Abstract (ABS) |
| 是否需要 onchainos 广播？ | **Yes** — EVM 链上的 transfer/approve 操作需要 onchainos 广播。Solana 上的 SPL token transfer 同样需要 onchainos |

### 接入路径

```
有参考 skills? → Solv SolvBTC (PR #181) 架构相似可参考
有 Rust SDK? → No
有其他语言 SDK? → Yes (gateapi-go, gateapi-python)
有 REST API? → Yes (Gate API v4)
```

**结论：接入路径 = "API"（Rust 调用 Gate API v4 REST）**

GTBTC 本质上是 ERC-20/SPL 代币，链上写操作 = 标准 ERC-20 transfer/approve，走 onchainos。链下查询走 Gate API v4 + DeFiLlama。

---

## §2 接口映射

### 2a. 操作列表

| 操作 | 类型 | 描述 |
|------|------|------|
| get-balance | 链下查询 | 查询用户 GTBTC 余额（EVM/Solana 地址） |
| get-price | 链下查询 | 查询 GTBTC/BTC 当前汇率及 USD 价格 |
| get-apr | 链下查询 | 查询当前 BTC 质押 APR |
| transfer | 链上写 | 将 GTBTC 从用户地址转给另一地址（EVM/Solana） |
| approve | 链上写 | 授权第三方合约（如 DEX）使用用户的 GTBTC（EVM 专用） |

> **关于 mint/redeem：**
> GTBTC 的铸造（BTC → GTBTC）和赎回（GTBTC → BTC）**仅通过 Gate 中心化平台**完成，无链上合约接口。官方明确表示"GTBTC 的 swap 和 redemption 目前只能通过 Gate 进行"。因此 **mint/redeem 不纳入链上操作范围**，如有需要可通过 Gate API v4 账户接口（需用户 API Key）实现，但超出本 plugin 初始版本范围。

---

### 2b. 链下查询接口

**Base URL:** `https://api.gateio.ws/api/v4`

| 操作 | 端点 | 方法 | 参数 | 响应关键字段 |
|------|------|------|------|------------|
| get-price | `/spot/tickers` | GET | `currency_pair=GTBTC_USDT` | `last`（最新价 USD）, `change_percentage` |
| get-apr | `/earn/uni/currencies/GTBTC` | GET | — | `min_lend_rate`, `max_lend_rate` |
| get-earn-lend-records | `/earn/uni/lend_records` | GET | `currency=GTBTC`, `page`, `limit` | `amount`, `type`, `create_time` |
| get-tvl | DeFiLlama: `https://api.llama.fi/protocol/gtbtc` | GET | — | `tvl`, `currentChainTvls` |

> **注意：** `get-balance` 通过 EVM RPC `eth_call` (balanceOf) 或 Solana RPC `getTokenAccountsByOwner` 查询链上余额，不依赖 Gate API。

**DeFiLlama 端点：**
```
GET https://api.llama.fi/protocol/gtbtc
响应: { tvl: [...], currentChainTvls: { Ethereum: x, BSC: y, ... } }
```

---

### 2c. 链上写操作（EVM）

> GTBTC 是 EIP-1967 透明代理合约，实现为标准 ERC-20（FiatToken 实现，Solidity 0.8.20）。

#### GTBTC Token 合约地址

| 链 | Chain ID | 合约地址（Token） |
|----|----------|-----------------|
| Ethereum | 1 | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` |
| BNB Smart Chain | 56 | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` |
| Base | 8453 | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` |

> 注意：三条 EVM 链合约地址相同（均为跨链代理部署）。

#### EVM 链上写操作映射表

| 操作 | 合约地址（来源） | 函数签名（canonical ABI 格式） | Selector（cast sig 验证 ✅） | ABI 参数顺序 |
|------|---------------|-------------------------------|---------------------------|------------|
| transfer | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3`（GTBTC Token，多链相同） | `transfer(address,uint256)` | `0xa9059cbb` ✅ | 1: `to` (address), 2: `amount` (uint256, decimals=8) |
| approve | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3`（GTBTC Token，多链相同） | `approve(address,uint256)` | `0x095ea7b3` ✅ | 1: `spender` (address), 2: `amount` (uint256, decimals=8) |

**关键：GTBTC decimals = 8**（与 BTC 精度一致，非标准 18）。
例：转账 0.001 GTBTC → `amount = 100_000`（8位精度）

**onchainos 命令映射（EVM）：**
```
# transfer
onchainos evm tx --chain <chain_id> \
  --to 0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3 \
  --data "transfer(address,uint256)" \
  --args "<recipient_address>" "<amount_in_8dec>"

# approve
onchainos evm tx --chain <chain_id> \
  --to 0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3 \
  --data "approve(address,uint256)" \
  --args "<spender_address>" "<amount_in_8dec>"
```

---

### 2d. 链上写操作（Solana）

| 操作 | Program ID | 操作方式 | amount 单位 | tx 编码 |
|------|-----------|---------|-----------|--------|
| transfer（SPL） | Token Program: `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | SPL Token `transfer` 指令（或 `transferChecked`） | lamports/satoshi（decimals=8）| base64 |

**Solana GTBTC Mint 地址：** `gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So`

**onchainos 命令映射（Solana）：**
```
# SPL token transfer
onchainos solana tx \
  --program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA \
  --instruction spl-transfer \
  --mint gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So \
  --from <source_token_account> \
  --to <dest_token_account> \
  --amount <amount_in_8dec>
```

---

## §3 用户场景

### 场景 1：查询 GTBTC 持仓与当前收益率

**用户意图：** "我持有多少 GTBTC？现在的年化收益是多少？"

**动作序列：**
1. `get-balance` — 调用 EVM RPC `eth_call(balanceOf(userAddress))` 到 `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3`
2. `get-price` — `GET /spot/tickers?currency_pair=GTBTC_USDT` 获取当前 USD 价格
3. `get-apr` — `GET /earn/uni/currencies/GTBTC` 获取当前年化利率范围
4. 计算持仓 USD 价值 = balance × price，展示给用户

**返回示例：**
```
持有 GTBTC: 0.05 GTBTC
当前价格: $67,090 / GTBTC（相当于 $3,354.50 USD）
BTC 质押 APR: 3.03%（参考，实际按净值增长）
```

---

### 场景 2：将 GTBTC 转账给另一地址（EVM）

**用户意图：** "把我的 0.01 GTBTC 转给 0xAbCd...1234"

**动作序列：**
1. `get-balance` — 确认余额充足（0.01 GTBTC = 1_000_000 原子单位，decimals=8）
2. `transfer` — onchainos 广播 EVM tx，调用 `transfer(address,uint256)` to GTBTC token 合约
   - `to`: 0xAbCd...1234
   - `amount`: 1000000（0.01 GTBTC × 10^8）
3. 等待 tx 确认，返回 tx hash

**onchainos 调用：**
```
onchainos evm tx --chain 1 \
  --to 0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3 \
  --data "transfer(address,uint256)" \
  --args "0xAbCd...1234" "1000000"
```

---

### 场景 3：授权 DEX 使用 GTBTC（EVM approve 场景）

**用户意图：** "我想在 Uniswap 上用 GTBTC 换 USDC，先授权"

**动作序列：**
1. 用户确认 DEX 路由器地址（如 Uniswap V3 Router: `0xE592427A0AEce92De3Edee1F18E0157C05861564`）
2. `approve` — onchainos 广播 EVM tx，调用 `approve(address,uint256)` to GTBTC token 合约
   - `spender`: 0xE592427A0AEce92De3Edee1F18E0157C05861564
   - `amount`: `uint256 max` 或具体数量
3. 返回 approve tx hash，提示用户继续在 DEX 完成交换

**onchainos 调用：**
```
onchainos evm tx --chain 1 \
  --to 0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3 \
  --data "approve(address,uint256)" \
  --args "0xE592427A0AEce92De3Edee1F18E0157C05861564" \
         "115792089237316195423570985008687907853269984665640564039457584007913129639935"
```

---

### 场景 4：查询 TVL 和持仓份额

**用户意图：** "GTBTC 的总锁仓量是多少？我持仓占多少比例？"

**动作序列：**
1. `GET https://api.llama.fi/protocol/gtbtc` — 获取 TVL 和各链分布
2. `get-balance` — 获取用户余额
3. 计算用户份额 = user_balance / total_supply × 100%

---

## §4 外部 API 依赖

| API | 用途 | 认证 | 限流 |
|-----|------|------|------|
| Gate API v4: `https://api.gateio.ws/api/v4` | GTBTC 价格（`/spot/tickers`）、APR（`/earn/uni/currencies/GTBTC`）| 公开（只读）无需 Key | 300 req/min |
| DeFiLlama: `https://api.llama.fi` | TVL、`/protocol/gtbtc` | 无需认证 | 宽松 |
| EVM JSON-RPC（各链） | balanceOf 余额查询 | 视节点而定（可用公共 RPC） | 视 RPC 而定 |
| Solana RPC | SPL token 余额（`getTokenAccountsByOwner`）| 无需认证（公共节点）| 视 RPC 而定 |

---

## §5 配置参数

| 参数 | 类型 | 必填 | 默认值 | 描述 |
|------|------|------|--------|------|
| `chain` | string | Yes | `ethereum` | 目标链：`ethereum`、`bsc`、`base`、`solana` |
| `evm_rpc_url` | string | No | 公共 RPC | EVM 节点 RPC URL |
| `solana_rpc_url` | string | No | `https://api.mainnet-beta.solana.com` | Solana RPC URL |
| `dry_run` | bool | No | `false` | 为 true 时仅模拟，不广播链上交易 |
| `slippage_bps` | u32 | No | 50 | DEX swap 时的滑点（basis points，仅 swap 场景使用）|

---

## §6 合约地址汇总

| 链 | 资产 | 地址 |
|----|------|------|
| Ethereum (1) | GTBTC Token | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` |
| BNB Smart Chain (56) | GTBTC Token | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` |
| Base (8453) | GTBTC Token | `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3` |
| Abstract | GTBTC Token | `0x0035b877c3ab50cffa9ed52ba05282c7045f78dd` |
| Solana | GTBTC SPL Mint | `gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So` |

---

## §7 实现注意事项

### Decimals = 8（关键！）

GTBTC 采用 8 位精度（与 BTC 一致，**不是** ERC-20 标准的 18 位）。

```
0.001 GTBTC = 100_000 atomic units (not 1_000_000_000_000_000)
```

错误的精度会导致转账金额偏差 10^10 倍，Developer Agent 必须单独处理。

### 无链上 Mint/Redeem

与 Solv SolvBTC 不同，GTBTC **没有**链上的 mint 或 redeem 合约。所有铸造和赎回均通过 Gate 交易所完成，不在本 plugin 初始版本范围内。如果用户询问如何"购买 GTBTC"，应引导至 `https://www.gate.com/staking/BTC`。

### 多链同地址

Ethereum、BNB Chain、Base 三条链使用**相同的合约地址** `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3`，但这是不同链上的独立部署，调用时必须指定正确的 `chain_id`。

### 参考实现

Solv SolvBTC plugin（PR #181）是同类 BTC 收益凭证，可参考其：
- 余额查询 → balanceOf 调用方式
- 价格查询 → DeFiLlama 集成方式
- L4 测试说明（BTC-pegged 协议需要小额 GTBTC 进行链上测试）

---

## §8 调研来源

- CoinGecko GTBTC 页面: https://www.coingecko.com/en/coins/gate-wrapped-btc
- Gate.com GTBTC Proof of Reserve: https://www.gate.com/proof-of-reserve/gtbtc
- Gate.com BTC Staking: https://www.gate.com/staking/BTC
- DeFiLlama GTBTC: https://defillama.com/protocol/gtbtc
- Gate API v4 文档: https://www.gate.com/docs/developers/apiv4/en/
- Gate EarnUniApi (GitHub): https://github.com/gateio/gateapi-python/blob/master/docs/EarnUniApi.md
- Etherscan GTBTC: https://etherscan.io/token/0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3

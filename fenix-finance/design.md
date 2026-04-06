# Fenix Finance V3 — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field               | Value                                                            |
|---------------------|------------------------------------------------------------------|
| plugin_name         | `fenix-finance`                                                  |
| dapp_name           | Fenix Finance V3                                                 |
| record_id           | `recvfIWnsWrSLr`                                                 |
| category            | dex                                                              |
| tags                | dex, amm, concentrated-liquidity, algebra, blast, ve33          |
| target_chains       | blast (81457)                                                    |
| target_protocols    | Algebra Integral V1 (Fenix fork), Fenix V2 AMM                  |
| priority            | P1                                                               |
| onchainos_broadcast | 是                                                               |

---

## 1. Background

### 1.1 DApp 介绍

Fenix Finance 是部署在 Blast（chain ID 81457）上的 MetaDEX，融合了 Uniswap AMM 架构、Curve ve 托管代币经济与 Convex 流动性市场三者的设计精华。

**三种池类型**：
- **Concentrated Liquidity AMM (CLAMM)**：基于 Algebra Integral V1 的集中流动性引擎，gas 效率比 Uniswap V3 提升约 20%，动态手续费，支持 Uniswap V4 风格的 Plugin & Hooks。
- **Volatile AMM (vAMM)**：恒积公式 (x×y=k)，适用于高波动性代币对。
- **Stableswap AMM**：Solidly 曲线 (x³y+y³x=k)，低滑点，适用于稳定币对。

**核心特点**：
- Algebra Integral V1 fork，**无 fee tier 参数**——每对代币只有一个池，手续费由协议动态调整。
- ve(3,3) 投票经济：veFNX 持有人对流动性激励进行投票。
- Blast 原生收益：合约内 ETH/USDB 自动产生 Blast native yield。
- 原生 ETH 通过 WETH 包装后交互（WETH = `0x4300000000000000000000000000000000000004`）。

### 1.2 接入可行性调研

| 维度                 | 结论                                                                                           |
|----------------------|------------------------------------------------------------------------------------------------|
| 合约开源             | 是 — [Satsyxbt/Fenix-dex-v3](https://github.com/Satsyxbt/Fenix-dex-v3)，Blastscan 已验证     |
| ABI 可获取           | 是 — Algebra Integral V1 标准接口 + Blastscan 源码验证                                         |
| 协议类型             | **Algebra Integral V1 fork**（非 Uniswap V3）— 无 fee tier，ExactInputSingleParams 含 deployer 字段 |
| 链下查询             | Goldsky 子图 (GraphQL) + 合约 view 函数 (eth_call)                                             |
| 链上操作             | EVM `wallet contract-call` via onchainos                                                       |
| ERC-20 Approve       | 需要（代币→SwapRouter，代币→NFPM），需先查 allowance                                            |
| 原生 ETH 处理        | 需要包装为 WETH（`0x4300000000000000000000000000000000000004`）后通过 SwapRouter 交互           |
| SDK                  | 无官方 Rust SDK，直接构造 ABI calldata                                                          |
| 子图 API Key         | 无需 Key（Goldsky 公开端点）                                                                    |
| 审计状态             | 已审计，Hats Finance bug bounty 在运行                                                          |

### 1.3 与 Uniswap V3 的关键差异

| 维度                    | Uniswap V3              | Fenix Finance (Algebra Integral V1)                    |
|-------------------------|-------------------------|--------------------------------------------------------|
| fee tier                | 有（500/3000/10000）     | **无** — 每对代币一个池，费率动态                       |
| ExactInputSingleParams  | 7 fields (含 fee)        | **7 fields，含 `deployer`，无 fee** (详见 §2.1)        |
| Factory.getPool         | `getPool(addr,addr,fee)` | **`poolByPair(addr,addr)`** selector `0xd9a641e1`      |
| QuoterV2.quoteExactInputSingle | 含 fee 参数       | **`QuoteExactInputSingleParams` 含 `deployer`**        |
| 路径编码（multihop）     | token-fee-token          | **token-deployer-token**（base pools deployer = 0x0）  |

### 1.4 接入路径

```
用户意图
  └→ Plugin (Rust)
       ├→ [链下] Goldsky 子图 GraphQL   (pools, tokens, price, TVL)
       ├→ [链下] QuoterV2.quoteExactInputSingle (eth_call, no gas)
       ├→ [链下] Factory.poolByPair     (validate pool exists)
       ├→ [链下] NFPM.positions(tokenId) (position details)
       ├→ [链下] NFPM.balanceOf(wallet) + tokenOfOwnerByIndex (enumerate positions)
       ├→ [链下] ERC-20 allowance check (eth_call)
       ├→ [链上] ERC-20 approve         (onchainos wallet contract-call)
       └→ [链上] SwapRouter / NFPM      (onchainos wallet contract-call)
```

---

## 2. DApp 核心能力 & 接口映射

### 2.0 合约地址（Blast Mainnet, Chain ID 81457）

| 合约                           | 地址                                         | 说明                                       |
|-------------------------------|----------------------------------------------|--------------------------------------------|
| **SwapRouter**                | `0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00` | Algebra Integral V1 SwapRouter             |
| **QuoterV2**                  | `0x94Ca5B835186A37A99776780BF976fAB81D84ED8` | 链下报价（eth_call）                        |
| **AlgebraFactory**            | `0x7a44CD060afC1B6F4c80A2B9b37f4473E74E25Df` | 查询池地址 `poolByPair`                     |
| **NonfungiblePositionManager (NFPM)** | `0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd` | LP 仓位 NFT（FNX-POS）            |
| **WETH**                      | `0x4300000000000000000000000000000000000004` | Blast 上的 Wrapped ETH                     |
| **USDB**                      | `0x4300000000000000000000000000000000000003` | Blast 原生稳定币                            |
| **BLAST token**               | `0xb1a5700fa2358173fe465e6ea4ff52e36e88e2ad` | Blast 生态 token                           |
| **FNX token**                 | `0x52f847356b38720B55ee18Cb3e094ca11C85A192` | Fenix 治理 token                           |
| **veFNX**                     | `0xC900C984a3581EfA4Fb56cAF6eF19721aAFbB4f9` | ve 锁仓 NFT                                |
| **Pair Factory (V2 AMM)**     | `0xa19C51D91891D3DF7C13Ed22a2f89d328A82950f` | vAMM/sAMM V2 池工厂                         |
| **RouterV2**                  | `0xbD571125856975DBfC2E9b6d1DE496D614D7BAEE` | V2 AMM 路由                                |

---

### 操作 1：Swap（代币兑换）

**类型**：链下报价 + 链上执行（两步）

**协议**：Algebra Integral V1 SwapRouter，`exactInputSingle`

> **重要**：与 Uniswap V3 不同，ExactInputSingleParams 含 `deployer` 字段（base pools 传 `address(0)`），且**无 fee tier 参数**。

#### Step A：获取报价（链下，eth_call）

```
函数签名: quoteExactInputSingle((address,address,uint256,uint160))
Selector: 0x5e5e6e0f
合约: QuoterV2 = 0x94Ca5B835186A37A99776780BF976fAB81D84ED8
可见性: view（不消耗 gas）
```

**QuoteExactInputSingleParams 结构**：

```solidity
struct QuoteExactInputSingleParams {
    address tokenIn;       // 输入 token 地址
    address tokenOut;      // 输出 token 地址
    uint256 amountIn;      // 精确输入量（最小单位）
    uint160 limitSqrtPrice; // 价格限制（0 = 不限制）
}
```

**返回值**：
```
(uint256 amountOut, uint256 amountIn, uint160 sqrtPriceX96After,
 uint32 initializedTicksCrossed, uint256 gasEstimate, uint16 fee)
```

**Rust 伪代码**：
```rust
// selector 0x5e5e6e0f + ABI encode (tokenIn, tokenOut, amountIn, 0)
let data = encode_call_4param(
    "0x5e5e6e0f",
    token_in, token_out, amount_in, 0u160
);
let result = eth_call(QUOTER_V2, &data, rpc_url).await?;
let amount_out = decode_u256(&result[..32])?;
```

#### Step B：验证池存在（链下）

```
函数签名: poolByPair(address,address)
Selector: 0xd9a641e1
合约: AlgebraFactory = 0x7a44CD060afC1B6F4c80A2B9b37f4473E74E25Df
```

返回 `address(0)` 时表示池不存在，应报错退出。

#### Step C：执行 Swap（链上）

**Step C1：检查并执行 ERC-20 Approve**

```
函数签名: allowance(address,address)
Selector: 0xdd62ed3e
参数: (owner=用户地址, spender=SwapRouter)

函数签名: approve(address,uint256)
Selector: 0x095ea7b3
合约: tokenIn ERC-20 地址
参数: (spender=SwapRouter=0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00, amount=uint256::MAX)
```

若 allowance < amountIn，执行 approve；执行后等待 3 秒再发 swap。

**Step C2：exactInputSingle**

```
函数签名: exactInputSingle((address,address,address,uint256,uint256,uint256,uint160))
Selector: 0xbc651188
合约: SwapRouter = 0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00
payable: 否（ERC-20 对）
```

**ExactInputSingleParams 结构（Algebra Integral V1）**：

```solidity
struct ExactInputSingleParams {
    address tokenIn;           // 输入 token
    address tokenOut;          // 输出 token
    address recipient;         // 接收输出的地址（用户钱包地址，不能为 0x0）
    uint256 deadline;          // Unix 时间戳截止时间（block.timestamp + 300）
    uint256 amountIn;          // 精确输入量
    uint256 amountOutMinimum;  // 最小输出量（来自 quoteExactInputSingle * (1 - slippage)）
    uint160 limitSqrtPrice;    // 0 = 不限制价格
}
```

**注意**：`deployer` 字段不在 ExactInputSingleParams 中（Integral V1 与 V2+ 差异），base pools 自动使用 address(0)。

**onchainos 命令**：
```bash
onchainos wallet contract-call \
  --chain-id 81457 \
  --address 0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00 \
  --input-data <ABI_ENCODED_CALLDATA>
```

**calldata 构造（Rust）**：
```rust
// recipient = get_wallet_address(81457)?;  // 真实钱包地址，不能为 0x0
let params = ExactInputSingleParams {
    token_in:            token_in_addr,
    token_out:           token_out_addr,
    recipient:           wallet_addr,
    deadline:            block_timestamp + 300,
    amount_in:           amount_in_wei,
    amount_out_minimum:  amount_out * (10000 - slippage_bps) / 10000,
    limit_sqrt_price:    U160::ZERO,
};
let calldata = abi_encode("0xbc651188", params.as_tuple());
```

#### Multihop Swap（exactInput）

路径格式：`abi.encodePacked(tokenIn, address(0), tokenMid, address(0), tokenOut)`
（deployer 为 `address(0)` 代表 base pool）

```
函数签名: exactInput((bytes,address,uint256,uint256,uint256))
Selector: 0xc04b8d59
```

**ExactInputParams**：
```solidity
struct ExactInputParams {
    bytes  path;              // encodePacked(tokenIn, deployer0, tokenMid, deployer1, tokenOut)
    address recipient;
    uint256 deadline;
    uint256 amountIn;
    uint256 amountOutMinimum;
}
```

---

### 操作 2：Add Liquidity（添加集中流动性仓位）

**类型**：链上操作（多步：approve × 2 → mint）

**合约**：NonfungiblePositionManager (NFPM) = `0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd`

#### Step A：approve token0 & token1 → NFPM

与 swap approve 相同逻辑，spender = NFPM 地址。两次 approve 之间等待 5 秒。

#### Step B：mint 新仓位

```
函数签名: mint((address,address,int24,int24,uint256,uint256,uint256,uint256,address,uint256))
Selector: 0x9cc1a283
合约: NFPM = 0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd
payable: 是
```

**MintParams 结构**：

```solidity
struct MintParams {
    address token0;          // 较小地址的 token（按地址排序）
    address token1;          // 较大地址的 token
    int24   tickLower;       // 价格区间下界（Tick）
    int24   tickUpper;       // 价格区间上界（Tick）
    uint256 amount0Desired;  // 期望添加的 token0 数量
    uint256 amount1Desired;  // 期望添加的 token1 数量
    uint256 amount0Min;      // token0 最小数量（滑点保护）
    uint256 amount1Min;      // token1 最小数量（滑点保护）
    address recipient;       // 接收 LP NFT 的地址（用户钱包）
    uint256 deadline;        // Unix 时间戳
}
```

**返回值**：`(uint256 tokenId, uint128 liquidity, uint256 amount0, uint256 amount1)`

**Tick 计算参考**：
- 全区间流动性（无限范围）：tickLower = -887220，tickUpper = 887220
- 对应当前价格 ±X% 的 tick：需根据 `sqrtPriceX96` 从子图或链上读取当前价格后换算

**多步调用时序**：
```
approve(token0, NFPM, MAX) → sleep(5s) →
approve(token1, NFPM, MAX) → sleep(5s) →
mint(params)
```

---

### 操作 3：Remove Liquidity（减少/移除集中流动性仓位）

**类型**：链上操作（两步：decreaseLiquidity → collect）

**合约**：NFPM = `0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd`

#### Step A：decreaseLiquidity

```
函数签名: decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))
Selector: 0x0c49ccbe
```

**DecreaseLiquidityParams**：
```solidity
struct DecreaseLiquidityParams {
    uint256 tokenId;      // LP NFT token ID
    uint128 liquidity;    // 要减少的流动性（从 positions(tokenId) 获取）
    uint256 amount0Min;   // token0 最小返还量
    uint256 amount1Min;   // token1 最小返还量
    uint256 deadline;
}
```

#### Step B：collect（提取代币 + 累积手续费）

等待 5 秒后执行 collect：

```
函数签名: collect((uint256,address,uint128,uint128))
Selector: 0xfc6f7865
```

**CollectParams**：
```solidity
struct CollectParams {
    uint256 tokenId;
    address recipient;     // 接收地址（用户钱包）
    uint128 amount0Max;    // uint128::MAX 收取全部
    uint128 amount1Max;    // uint128::MAX 收取全部
}
```

**完整时序**：
```
decreaseLiquidity(params) → sleep(5s) → collect(params)
```

---

### 操作 4：查询仓位（Positions）

**类型**：链下查询（eth_call + 子图）

#### 4A：枚举用户所有 LP NFT

```
函数签名: balanceOf(address)
Selector: 0x70a08231
合约: NFPM

函数签名: tokenOfOwnerByIndex(address,uint256)
Selector: 0x2f745c59
合约: NFPM
```

**流程**：
1. `balanceOf(walletAddr)` → `count`
2. 对 i in 0..count：`tokenOfOwnerByIndex(walletAddr, i)` → `tokenId`
3. `positions(tokenId)` → 仓位详情

#### 4B：查询仓位详情

```
函数签名: positions(uint256)
Selector: 0x99fbab88
合约: NFPM
```

**返回值**（ABI 解码顺序）：
```
(uint96 nonce, address operator, address token0, address token1,
 int24 tickLower, int24 tickUpper, uint128 liquidity,
 uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128,
 uint128 tokensOwed0, uint128 tokensOwed1)
```

> **Tick 解码注意**：tickLower/tickUpper 为 ABI `int24` 字段（返回 32 字节），取最后 6 个 hex 字符（3 字节），用 `i32` 解析。参见 [kb/protocols/dex.md#tick-decoding]。

#### 4C：子图查询（推荐用于 UI）

```graphql
# V3 Pools Subgraph
# Endpoint: https://api.goldsky.com/api/public/project_clxadvm41bujy01ui2qalezdn/subgraphs/fenix-v3-dex/latest/gn

query GetUserPositions($owner: String!) {
  positions(where: { owner: $owner, liquidity_gt: "0" }) {
    id
    owner
    pool {
      id
      token0 { id symbol decimals }
      token1 { id symbol decimals }
      sqrtPrice
      tick
      feeTier
    }
    tickLower { tickIdx }
    tickUpper { tickIdx }
    liquidity
    collectedFeesToken0
    collectedFeesToken1
  }
}
```

---

### 操作 5：查询价格与池信息

**类型**：链下查询（子图 GraphQL）

```graphql
# 查询指定 token pair 的池信息
query GetPool($token0: String!, $token1: String!) {
  pools(
    where: {
      token0_: { id: $token0 }
      token1_: { id: $token1 }
    }
  ) {
    id
    token0Price
    token1Price
    sqrtPrice
    liquidity
    volumeUSD
    feeTier
    token0 { id symbol decimals }
    token1 { id symbol decimals }
  }
}
```

```graphql
# 查询代币的 USD 价格
query GetTokenPrice($tokenId: String!) {
  token(id: $tokenId) {
    id
    symbol
    derivedETH
    tokenDayData(first: 1, orderBy: date, orderDirection: desc) {
      priceUSD
    }
  }
}
```

---

## 3. 用户场景

### 场景 1：ETH → USDB Swap（单跳）

**用户意图**："帮我在 Fenix Finance 用 0.1 ETH 换 USDB"

**执行流程**：
1. 解析 token 地址：ETH → WETH = `0x4300000000000000000000000000000000000004`，USDB = `0x4300000000000000000000000000000000000003`
2. 从子图或 QuoterV2 确认 WETH/USDB 池存在（`poolByPair` 返回非零地址）
3. 调用 `quoteExactInputSingle(WETH, USDB, 0.1 ETH, 0)` → 获取预估 amountOut
4. 检查 allowance(walletAddr, SwapRouter)：
   - 若不足，执行 `approve(SwapRouter, uint256::MAX)` → 等待 3 秒
5. 构造 ExactInputSingleParams：
   ```
   tokenIn = WETH, tokenOut = USDB, recipient = walletAddr,
   deadline = now + 300, amountIn = 0.1e18,
   amountOutMinimum = amountOut * 9950 / 10000 (0.5% slippage),
   limitSqrtPrice = 0
   ```
6. 执行：
   ```bash
   onchainos wallet contract-call \
     --chain-id 81457 \
     --address 0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00 \
     --input-data <calldata with selector 0xbc651188>
   ```
7. 输出：txHash，预估收到 X USDB，实际 amountOut

**注意**：若用户输入原生 ETH，需先调用 WETH.deposit() 进行包装，或使用支持 ETH 的路径（Fenix 的 SwapRouter 继承 PeripheryPayments，可自动处理 ETH→WETH）。

---

### 场景 2：添加 WETH/USDB 集中流动性仓位

**用户意图**："帮我在 Fenix Finance 的 WETH/USDB 池里添加 0.05 WETH 和 100 USDB 的流动性，价格区间 ±20%"

**执行流程**：
1. 从子图查询当前池价格（`sqrtPrice`, `tick`）
2. 根据当前 tick 计算 tickLower/tickUpper（±20% 约 ±1825 ticks）
3. 检查并执行 approve(WETH, NFPM, MAX) → 等待 5 秒
4. 检查并执行 approve(USDB, NFPM, MAX) → 等待 5 秒
5. 构造 MintParams：
   ```
   token0 = USDB (小地址), token1 = WETH (大地址),
   tickLower = currentTick - 1825, tickUpper = currentTick + 1825,
   amount0Desired = 100e18 (USDB), amount1Desired = 0.05e18 (WETH),
   amount0Min = 95e18, amount1Min = 0.0475e18 (5% slippage),
   recipient = walletAddr, deadline = now + 300
   ```
6. 执行：
   ```bash
   onchainos wallet contract-call \
     --chain-id 81457 \
     --address 0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd \
     --input-data <calldata with selector 0x9cc1a283>
   ```
7. 解析返回的 tokenId，输出：LP NFT token ID，实际消耗 token0/token1 数量

---

### 场景 3：移除流动性并收取手续费

**用户意图**："帮我移除我在 Fenix Finance 上 tokenId=1234 的全部流动性，并提取手续费"

**执行流程**：
1. 调用 `positions(1234)` 获取当前 liquidity、token0、token1、tickLower、tickUpper
2. 若 liquidity = 0，提示"此仓位已无流动性，可直接执行 collect 收取手续费"
3. 调用 `decreaseLiquidity(1234, liquidity, 0, 0, deadline)` → 等待 5 秒
4. 调用 `collect(1234, walletAddr, uint128::MAX, uint128::MAX)`
5. 输出：收到的 token0 数量、token1 数量（含未提取的手续费）

---

### 场景 4：查询用户 LP 仓位列表

**用户意图**："显示我在 Fenix Finance 上的所有流动性仓位"

**执行流程**：
1. 优先调用子图 `GetUserPositions` 查询（一次返回所有仓位）
2. 备选：链上 `balanceOf(walletAddr)` → 逐个 `tokenOfOwnerByIndex` → `positions(tokenId)`
3. 对每个仓位，通过子图或 `token().symbol()` 解析 token0/token1 符号
4. 展示：tokenId，池对，价格区间，流动性，未收取手续费（tokensOwed0/1）

---

### 场景 5：查询 token 兑换汇率

**用户意图**："现在 Fenix Finance 上 1 WETH 能换多少 USDB？"

**执行流程**：
1. 调用 `quoteExactInputSingle(WETH, USDB, 1e18, 0)` → `amountOut`
2. 格式化：1 WETH ≈ {amountOut / 1e18} USDB（当前价格）
3. 附加：从子图查询 24h 交易量、手续费率

---

## 4. 外部 API 依赖

| API                        | 用途                               | Base URL / 端点                                                                                       | 认证        |
|----------------------------|------------------------------------|-------------------------------------------------------------------------------------------------------|-------------|
| Goldsky V3 Subgraph        | 池信息、token 价格、用户仓位       | `https://api.goldsky.com/api/public/project_clxadvm41bujy01ui2qalezdn/subgraphs/fenix-v3-dex/latest/gn` | 无需 Key   |
| Goldsky V2 Subgraph        | V2 AMM 池信息                      | `https://api.goldsky.com/api/public/project_clxadvm41bujy01ui2qalezdn/subgraphs/fenix-v2-subgraph/latest/gn` | 无需 Key |
| Blast RPC                  | eth_call (quote, positions)        | 公开 RPC 或 onchainos 内置                                                                             | 视提供商    |
| Blast RPC                  | 链上写入                           | via onchainos wallet contract-call                                                                    | onchainos   |

**Goldsky 速率限制**：公开端点默认 50 req / 10s，生产环境如需更高配额联系 support@goldsky.com。

**Goldsky GraphQL 示例请求**：
```bash
curl -X POST \
  https://api.goldsky.com/api/public/project_clxadvm41bujy01ui2qalezdn/subgraphs/fenix-v3-dex/latest/gn \
  -H "Content-Type: application/json" \
  -d '{"query": "{ pools(first: 5, orderBy: volumeUSD, orderDirection: desc) { id token0 { symbol } token1 { symbol } volumeUSD } }"}'
```

---

## 5. 配置参数

| 参数名                  | 类型    | 说明                                       | 示例值 / 默认值                                            |
|-------------------------|---------|--------------------------------------------|-----------------------------------------------------------|
| `chain_id`              | u64     | 目标链 ID                                   | `81457` (Blast)                                           |
| `token_in`              | String  | 输入 token（符号或地址）                    | `"WETH"` / `"0x4300...0004"`                              |
| `token_out`             | String  | 输出 token（符号或地址）                    | `"USDB"` / `"0x4300...0003"`                              |
| `amount_in`             | String  | 输入数量（人类可读）                        | `"0.1"`                                                   |
| `slippage_bps`          | u32     | 最大滑点（基点，100 = 1%）                  | `50` (0.5%)                                               |
| `deadline_secs`         | u64     | 交易截止时间（秒，从当前块起算）             | `300`                                                     |
| `token_id`              | u64     | LP NFT token ID（仓位操作时使用）           | 由用户提供或从查询获取                                     |
| `tick_lower`            | i32     | LP 价格区间下界 tick                        | `-887220`（全区间）                                       |
| `tick_upper`            | i32     | LP 价格区间上界 tick                        | `887220`（全区间）                                        |
| `amount0_desired`       | String  | 添加 token0 的期望数量                      | `"100"` (USDB)                                            |
| `amount1_desired`       | String  | 添加 token1 的期望数量                      | `"0.05"` (WETH)                                           |
| `swap_router`           | Address | SwapRouter 地址（可覆盖默认值）              | `0x2df37Cb897fdffc6B4b03d8252d85BE7C6dA9d00`             |
| `nfpm_address`          | Address | NFPM 地址（可覆盖默认值）                   | `0x8881b3Fb762d1D50e6172f621F107E24299AA1Cd`             |
| `quoter_v2_address`     | Address | QuoterV2 地址（可覆盖默认值）               | `0x94Ca5B835186A37A99776780BF976fAB81D84ED8`             |
| `factory_address`       | Address | AlgebraFactory 地址（可覆盖默认值）         | `0x7a44CD060afC1B6F4c80A2B9b37f4473E74E25Df`             |
| `subgraph_v3_url`       | String  | Goldsky V3 子图 URL                         | 见 §4                                                     |

**已知 token 地址映射（Blast, chain 81457）**：

| Symbol | Address                                      | Decimals |
|--------|----------------------------------------------|----------|
| WETH   | `0x4300000000000000000000000000000000000004` | 18       |
| USDB   | `0x4300000000000000000000000000000000000003` | 18       |
| BLAST  | `0xb1a5700fa2358173fe465e6ea4ff52e36e88e2ad` | 18       |
| FNX    | `0x52f847356b38720B55ee18Cb3e094ca11C85A192` | 18       |

---

## 6. 关键函数 Selector 汇总

| 函数签名                                                                                               | Selector     | 合约           | 验证命令                                                                                                                         |
|--------------------------------------------------------------------------------------------------------|--------------|----------------|----------------------------------------------------------------------------------------------------------------------------------|
| `exactInputSingle((address,address,address,uint256,uint256,uint256,uint160))`                          | `0xbc651188` | SwapRouter     | `cast sig "exactInputSingle((address,address,address,uint256,uint256,uint256,uint160))"`                                        |
| `exactInput((bytes,address,uint256,uint256,uint256))`                                                  | `0xc04b8d59` | SwapRouter     | `cast sig "exactInput((bytes,address,uint256,uint256,uint256))"`                                                                |
| `exactOutputSingle((address,address,address,uint256,uint256,uint256,uint160))`                         | `0x61d4d5b3` | SwapRouter     | `cast sig "exactOutputSingle((address,address,address,uint256,uint256,uint256,uint160))"`                                       |
| `quoteExactInputSingle((address,address,uint256,uint160))`                                             | `0x5e5e6e0f` | QuoterV2       | `cast sig "quoteExactInputSingle((address,address,uint256,uint160))"`                                                           |
| `quoteExactInput(bytes,uint256)`                                                                       | `0xcdca1753` | QuoterV2       | `cast sig "quoteExactInput(bytes,uint256)"`                                                                                     |
| `poolByPair(address,address)`                                                                          | `0xd9a641e1` | AlgebraFactory | `cast sig "poolByPair(address,address)"`                                                                                        |
| `mint((address,address,int24,int24,uint256,uint256,uint256,uint256,address,uint256))`                  | `0x9cc1a283` | NFPM           | `cast sig "mint((address,address,int24,int24,uint256,uint256,uint256,uint256,address,uint256))"`                                |
| `increaseLiquidity((uint256,uint256,uint256,uint256,uint256,uint256))`                                 | `0x219f5d17` | NFPM           | `cast sig "increaseLiquidity((uint256,uint256,uint256,uint256,uint256,uint256))"`                                               |
| `decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))`                                         | `0x0c49ccbe` | NFPM           | `cast sig "decreaseLiquidity((uint256,uint128,uint256,uint256,uint256))"`                                                       |
| `collect((uint256,address,uint128,uint128))`                                                           | `0xfc6f7865` | NFPM           | `cast sig "collect((uint256,address,uint128,uint128))"`                                                                         |
| `positions(uint256)`                                                                                   | `0x99fbab88` | NFPM           | `cast sig "positions(uint256)"`                                                                                                 |
| `burn(uint256)`                                                                                        | `0x42966c68` | NFPM           | `cast sig "burn(uint256)"`                                                                                                      |
| `balanceOf(address)`                                                                                   | `0x70a08231` | NFPM / ERC-20  | `cast sig "balanceOf(address)"`                                                                                                 |
| `tokenOfOwnerByIndex(address,uint256)`                                                                 | `0x2f745c59` | NFPM           | `cast sig "tokenOfOwnerByIndex(address,uint256)"`                                                                               |
| `allowance(address,address)`                                                                           | `0xdd62ed3e` | ERC-20         | `cast sig "allowance(address,address)"`                                                                                         |
| `approve(address,uint256)`                                                                             | `0x095ea7b3` | ERC-20         | `cast sig "approve(address,uint256)"`                                                                                           |

---

## 7. 注意事项 & 已知陷阱

1. **Algebra vs Uniswap V3**：Fenix 使用 Algebra Integral V1，**没有 fee tier** 参数。Factory 用 `poolByPair(address,address)` 而非 `getPool(address,address,uint24)`。选错接口会导致静默失败。

2. **recipient 不能为 0x0**：`exactInputSingle` 的 `recipient` 必须是真实钱包地址（`get_wallet_address(81457)`），传 `address(0)` 会导致 revert（TF 错误）。dry-run 模式用零地址占位时需注意此风险。

3. **approve 后需等待**：approve → swap 等待 **3 秒**；approve → mint（多步 LP）等待 **5 秒**；decreaseLiquidity → collect 等待 **5 秒**，避免 nonce 冲突。

4. **allowance 检查**：每次 approve 前先 `allowance(owner, spender)` 检查，若已足额跳过，防止 "replacement transaction underpriced" 错误。

5. **tick 解码溢出**：`positions(tokenId)` 返回的 tickLower/tickUpper 为 ABI int24（32 字节），正确解法：取末 6 个 hex 字符（3 字节）→ `u32::from_str_radix(last6, 16) as i32`。

6. **WETH 包装**：Blast 原生 ETH 需先包装为 WETH（`0x4300...0004`）才能通过 SwapRouter 交互。若用户传 "ETH"，需先执行 `WETH.deposit{value: amount}()` 或通过 multicall/refundETH 路径。

7. **token0/token1 排序**：NFPM.mint 要求 token0 < token1（地址数值比较）。若用户输入顺序相反，需在构造 MintParams 前交换 token0/token1 以及对应 amountDesired。

8. **QuoterV2 零流动性虚假报价**：QuoterV2 在池内无流动性时仍可能返回非零报价。调用 quoteExactInputSingle 前，先用 `poolByPair` 确认池存在；可进一步调用池合约 `liquidity()` 确认有效流动性。

9. **函数 Selector 计算**：必须使用 Ethereum Keccak-256（`cast keccak` 或 `cast sig`），**不能用 Python hashlib.sha3_256**（NIST SHA3，结果不同）。

10. **Blast native yield**：合约持有的 ETH/USDB 会产生 Blast 原生收益，不影响 swap/LP 操作的 ABI，但需注意余额查询时可能包含 yield 部分。

# Stargate V2 — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field            | Value                                                     |
|------------------|-----------------------------------------------------------|
| plugin_name      | `stargate-v2`                                             |
| dapp_name        | Stargate V2                                               |
| category         | defi-protocol                                             |
| tags             | bridge, cross-chain, layerzero, omnichain                 |
| target_chains    | ethereum,arbitrum,optimism,polygon,bsc,avalanche,base,linea,mantle,scroll |
| target_protocols | Stargate V2, LayerZero V2                                 |
| priority         | P1                                                        |
| sdk              | LayerZero SDK TS (TypeScript only → 接入路径: API/合约直调) |
| onchainos_broadcast | 是                                                   |

---

## 1. Background

### 1.1 DApp 介绍

Stargate V2 是基于 LayerZero V2 构建的跨链流动性传输协议，支持在 30+ EVM 链之间无缝转移原生资产（USDC、USDT、ETH 等）。与 V1 相比，V2 引入了两种传输模式：

- **Taxi 模式**：立即发送，每笔交易独立打包，费用较高但即时到达。
- **Bus 模式**：批量聚合（2–10 笔），摊薄 LayerZero 消息费，费用更低但需等待凑团。

V2 合约实现了 LayerZero OFT（Omnichain Fungible Token）标准的超集接口 `IStargate`，所有链上操作通过唯一入口函数 `sendToken()` 执行。

### 1.2 接入可行性调研

| 维度           | 结论                                                                            |
|----------------|---------------------------------------------------------------------------------|
| 合约开源       | 是 — [stargate-protocol/stargate-v2](https://github.com/stargate-protocol/stargate-v2) |
| ABI 可获取     | 是 — IStargate.sol / IOFT.sol 完整公开                                          |
| SDK 语言       | TypeScript (LayerZero SDK TS)，无 Rust SDK                                      |
| 链下查询       | LayerZero Scan API (REST) + 合约 view 函数                                      |
| 链上操作       | EVM `sendToken()` via onchainos wallet contract-call                            |
| ERC-20 Approve | 需要（非 native token 池），via contract-call                                   |
| 是否需要 API Key | LayerZero Scan API 无需 Key（公开端点）                                       |
| 开源社区类似实现 | 无                                                                            |

### 1.3 接入路径

由于 SDK 仅提供 TypeScript 实现，Plugin 后端为 Rust，**接入路径选择：直接调用链上合约 + LayerZero Scan REST API**。

```
用户意图
  └→ Plugin (Rust)
       ├→ [链下] quoteOFT / quoteSend  (eth_call via RPC)
       ├→ [链下] ERC-20 allowance 检查 (eth_call via RPC)
       ├→ [链上] ERC-20 approve        (onchainos wallet contract-call)
       ├→ [链上] sendToken             (onchainos wallet contract-call)
       └→ [链下] LayerZero Scan API    (GET /messages/tx/{txHash})
```

---

## 2. DApp 核心能力 & 接口映射

### 2.1 支持的链与 Endpoint ID (EID) 映射

| Chain           | Chain ID | LayerZero EID |
|-----------------|----------|---------------|
| Ethereum        | 1        | 30101         |
| BNB Chain       | 56       | 30102         |
| Avalanche C     | 43114    | 30106         |
| Polygon         | 137      | 30109         |
| Arbitrum One    | 42161    | 30110         |
| OP Mainnet      | 10       | 30111         |
| Mantle          | 5000     | 30181         |
| Base            | 8453     | 30184         |
| Linea           | 59144    | 30183         |
| Scroll          | 534352   | 30214         |
| Metis           | 1088     | 30151         |
| Kava            | 2222     | 30177         |

### 2.2 Asset ID 映射

| Asset  | Asset ID | 说明                        |
|--------|----------|-----------------------------|
| USDC   | 1        | 含 USDC.e（Scroll）          |
| USDT   | 2        | 含 m.USDT（Metis）           |
| ETH    | 13       | Native ETH / WETH           |
| METIS  | 17       | Metis 链原生 token           |
| mETH   | 22       | Mantle ETH                  |

### 2.3 主要合约地址（Pool 型，常用链）

| Chain      | Pool         | 合约地址                                     |
|------------|--------------|----------------------------------------------|
| Ethereum   | Native (ETH) | `0x77b2043768d28E9C9aB44E1aBfC95944bcE57931` |
| Ethereum   | USDC         | `0xc026395860Db2d07ee33e05fE50ed7bD583189C7` |
| Ethereum   | USDT         | `0x933597a323Eb81cAe705C5bC29985172fd5A3973` |
| Arbitrum   | Native (ETH) | `0xA45B5130f36CDcA45667738e2a258AB09f4A5f7F` |
| Arbitrum   | USDC         | `0xe8CDF27AcD73a434D661C84887215F7598e7d0d3` |
| Arbitrum   | USDT         | `0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0` |
| OP Mainnet | Native (ETH) | `0xe8CDF27AcD73a434D661C84887215F7598e7d0d3` |
| OP Mainnet | USDC         | `0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0` |
| Base       | Native (ETH) | `0xdc181Bd607330aeeBEF6ea62e03e5e1Fb4B6F7C7` |
| Base       | USDC         | `0x27a16dc786820B16E5c9028b75B99F6f604b5d26` |
| Polygon    | USDC         | `0x9Aa02D4Fae7F58b8E8f34c66E756cC734DAc7fe4` |
| Polygon    | USDT         | `0xd47b03ee6d86Cf251ee7860FB2ACf9f91B9fD4d7` |
| BNB Chain  | USDC         | `0x962Bd449E630b0d928f308Ce63f1A21F02576057` |
| BNB Chain  | USDT         | `0x138EB30f73BC423c6455C53df6D89CB01d9eBc63` |
| Avalanche  | USDC         | `0x5634c4a5FEd09819E3c46D86A965Dd9447d86e47` |
| Avalanche  | USDT         | `0x12dC9256Acc9895B076f6638D628382881e62CeE` |

> Pool 合约地址即为 `sendToken` 的调用目标。Native 池（token() == address(0)）无需 ERC-20 approve。

---

### 操作 1：查询跨链报价与费用

**类型**：链下查询（eth_call，两步）

**目的**：在执行 `sendToken` 前，获取预计到账金额和所需 LayerZero 消息费（native token 费）。

#### Step A：`quoteOFT` — 查询滑点与实际到账

```
函数签名: quoteOFT((uint32,bytes32,uint256,uint256,bytes,bytes,bytes))
Selector: 0x0d35b415  [验证: cast sig "quoteOFT((uint32,bytes32,uint256,uint256,bytes,bytes,bytes))"]
可见性: view
合约: Stargate Pool/OFT 地址（对应链对应 token）
```

**SendParam 结构（ABI tuple 编码顺序）**：

```
(
  uint32  dstEid,       // 目标链 EID，如 30110 = Arbitrum
  bytes32 to,           // 接收地址，左补零：abi.encode(address) → bytes32
  uint256 amountLD,     // 发送金额（本链精度，USDC=6位，ETH=18位）
  uint256 minAmountLD,  // 最小接收金额（首次可设为 amountLD，后用 receipt 覆盖）
  bytes   extraOptions, // 通常为 0x（空）
  bytes   composeMsg,   // 通常为 0x（空，无 compose）
  bytes   oftCmd        // 0x = Taxi; 0x00 = Bus
)
```

**返回值**：
```
(OFTLimit memory, OFTFeeDetail[] memory, OFTReceipt memory receipt)
receipt.amountSentLD    // 实际扣款
receipt.amountReceivedLD // 目标链实际到账（含协议费）
```

**RPC 调用（Rust 伪代码）**：
```rust
// 编码 calldata
let calldata = encode_call("quoteOFT", &[send_param_tuple]);
let result = rpc_eth_call(pool_address, calldata, chain_id).await?;
let receipt = decode_oft_receipt(result)?;
// 用 receipt.amountReceivedLD 作为 minAmountLD
```

#### Step B：`quoteSend` — 查询 LayerZero 消息费

```
函数签名: quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)
Selector: 0x3b6f743b  [验证: cast sig "quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)"]
可见性: view
合约: 同 Stargate Pool/OFT 地址
```

**参数**：
- `_sendParam`: 与 quoteOFT 相同的 SendParam（minAmountLD 已更新为 receipt.amountReceivedLD）
- `_payInLzToken`: `false`（用 native token 付费）

**返回值**：
```
MessagingFee {
  uint256 nativeFee,   // 需随 tx 附带的 ETH 金额
  uint256 lzTokenFee   // 通常为 0
}
```

---

### 操作 2：ERC-20 Approve（非 native token 池）

**类型**：链上操作（onchainos wallet contract-call）

**触发条件**：当目标池 token 为 ERC-20（USDC、USDT 等），且当前 allowance < amountLD 时执行。

#### 先查询 allowance（链下）

```
函数签名: allowance(address,address)
Selector: 0xdd62ed3e  [验证: cast sig "allowance(address,address)"]
合约: ERC-20 token 地址（从 pool 合约读 token() 获得）
参数: (owner=用户地址, spender=pool合约地址)
```

**获取 token 地址**：
```
函数签名: token()
Selector: 0xfc0c546a  [验证: cast sig "token()"]
合约: Stargate Pool 地址
返回: address（若为 0x0 则为 native ETH，无需 approve）
```

#### 执行 approve

```
函数签名: approve(address,uint256)
Selector: 0x095ea7b3  [验证: cast sig "approve(address,uint256)"]
合约: ERC-20 token 地址（如 USDC）
参数:
  spender: Stargate Pool 合约地址
  amount:  amountLD（或 uint256::MAX 无限授权）
```

**onchainos 命令**：
```bash
onchainos wallet contract-call \
  --chain-id <CHAIN_ID> \
  --address <ERC20_TOKEN_ADDRESS> \
  --input-data "0x095ea7b3\
<spender_address_padded_32bytes>\
<amount_uint256_hex>"
```

**calldata 构造示例（approve USDC to Stargate USDC Pool on Ethereum）**：
```
function selector: 0x095ea7b3
spender (Stargate USDC Pool Ethereum):
  0x000000000000000000000000c026395860db2d07ee33e05fe50ed7bd583189c7
amount (100 USDC = 100_000_000, hex):
  0x0000000000000000000000000000000000000000000000000000000005f5e100
```

---

### 操作 3：跨链转账 sendToken（核心操作）

**类型**：链上操作（onchainos wallet contract-call）

**前提**：已完成 quoteOFT → quoteSend → approve（如需）

```
函数签名: sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)
Selector: 0xcbef2aa9
  [验证: cast sig "sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)"]
合约: Stargate Pool/OFT 地址（来源链对应 token 的 pool）
payable: 是
```

**完整 ABI 参数（按顺序）**：

```solidity
sendToken(
  SendParam calldata _sendParam,   // tuple(uint32,bytes32,uint256,uint256,bytes,bytes,bytes)
  MessagingFee calldata _fee,      // tuple(uint256,uint256)  [nativeFee, lzTokenFee]
  address _refundAddress           // 超额 native fee 退回地址
) external payable
```

**msg.value 计算**：
- ERC-20 池：`msg.value = messagingFee.nativeFee`
- Native ETH 池：`msg.value = messagingFee.nativeFee + amountLD`

**onchainos 命令**：
```bash
onchainos wallet contract-call \
  --chain-id <SRC_CHAIN_ID> \
  --address <STARGATE_POOL_ADDRESS> \
  --value <MSG_VALUE_WEI> \
  --input-data <ABI_ENCODED_CALLDATA>
```

**calldata 构造（Rust，taxi 模式转 USDC）**：
```rust
// sendParam tuple
let send_param = (
    dst_eid as u32,                            // e.g. 30110 for Arbitrum
    address_to_bytes32(receiver),              // receiver padded to 32 bytes
    amount_ld,                                 // e.g. 100_000_000 (100 USDC)
    min_amount_ld,                             // from quoteOFT receipt
    Bytes::new(),                              // extraOptions = 0x
    Bytes::new(),                              // composeMsg = 0x
    Bytes::new(),                              // oftCmd = 0x (taxi)
);
// messagingFee tuple
let fee = (native_fee, 0u256);
// refund address = caller
let calldata = abi_encode_call(
    "sendToken",
    &[send_param.into(), fee.into(), refund_addr.into()]
);
```

**Bus 模式**：将 `oftCmd` 设为 `Bytes::from([0u8])` (单字节 0x00)，费用更低但需等待批次聚合。

---

### 操作 4：查询跨链交易状态

**类型**：链下查询（LayerZero Scan REST API）

**Base URL**：`https://scan.layerzero-api.com/v1`

**端点**：
```
GET /messages/tx/{txHash}
```

**参数**：
- `txHash`：来源链的交易哈希（`sendToken` 返回后从链上日志获取）

**请求示例**：
```bash
curl "https://scan.layerzero-api.com/v1/messages/tx/0xabc123..."
```

**响应关键字段**：
```json
{
  "messages": [{
    "pathway": {
      "srcEid": 30101,
      "dstEid": 30110,
      "sender": { "address": "0x..." },
      "receiver": { "address": "0x..." }
    },
    "source": {
      "tx": { "txHash": "0xabc...", "blockNumber": 12345678 },
      "status": "DELIVERED"
    },
    "destination": {
      "tx": { "txHash": "0xdef..." },
      "status": "DELIVERED"
    },
    "status": "DELIVERED"
  }]
}
```

**状态枚举**：
| Status           | 含义                                  |
|------------------|---------------------------------------|
| INFLIGHT         | 等待来源链确认                        |
| CONFIRMING       | 目标链已提交，等待最终确认            |
| DELIVERED        | 跨链消息已成功在目标链执行            |
| FAILED           | 交易失败                              |
| PAYLOAD_STORED   | 到达目标链但 gas 不足或 revert，可重试 |
| BLOCKED          | 前序 nonce 未清，暂停                 |

**轮询建议**：每 10 秒轮询一次，超时阈值 15 分钟（taxi）/ 60 分钟（bus）。

---

### 操作 5：查询指定地址的跨链历史记录

**类型**：链下查询（LayerZero Scan REST API）

**端点**：
```
GET /messages/wallet/{srcAddress}
```

**参数**：
- `srcAddress`：用户钱包地址
- `limit`：返回数量（可选，默认 20）
- `start` / `end`：Unix 时间戳过滤（可选）
- `nextToken`：分页游标（可选）

**请求示例**：
```bash
curl "https://scan.layerzero-api.com/v1/messages/wallet/0xUserAddress?limit=20"
```

---

## 3. 用户场景

### 场景 1：ETH 从 Ethereum 跨链到 Arbitrum（Taxi 模式，立即到账）

**用户意图**："帮我把 0.1 ETH 从以太坊主网桥接到 Arbitrum"

**执行流程**：
1. 确定 src pool：Ethereum Native Pool `0x77b2043768d28E9C9aB44E1aBfC95944bcE57931`（EID 30101）
2. 确定 dst EID：Arbitrum = 30110
3. 调用 `token()` 确认为 native（返回 0x0），无需 approve
4. 构造 SendParam：`oftCmd = 0x`（taxi），`amountLD = 0.1 ETH = 100000000000000000`
5. 调用 `quoteOFT` 获取 `minAmountLD`
6. 调用 `quoteSend` 获取 `nativeFee`
7. `msg.value = nativeFee + 0.1 ETH`
8. 执行：
   ```bash
   onchainos wallet contract-call \
     --chain-id 1 \
     --address 0x77b2043768d28E9C9aB44E1aBfC95944bcE57931 \
     --value <nativeFee + 100000000000000000> \
     --input-data <sendToken calldata>
   ```
9. 获取 txHash，轮询 LayerZero Scan API 直到 status = DELIVERED

**预期结果**：~1–3 分钟内 Arbitrum 收到 ETH（扣除少量协议费）。

---

### 场景 2：USDC 从 Arbitrum 跨链到 Polygon（Bus 模式，低费用）

**用户意图**："便宜一些，把 500 USDC 从 Arbitrum 转到 Polygon"

**执行流程**：
1. src pool：Arbitrum USDC `0xe8CDF27AcD73a434D661C84887215F7598e7d0d3`（EID 30110）
2. dst EID：Polygon = 30109
3. 调用 `token()` 返回 USDC 合约地址，需要 approve
4. 检查 `allowance(user, stargatePool)` → 若不足则先 approve：
   ```bash
   onchainos wallet contract-call \
     --chain-id 42161 \
     --address <USDC_ADDRESS_ARBITRUM> \
     --input-data 0x095ea7b3<pool_addr_padded><500000000_padded>
   ```
5. 构造 SendParam：`oftCmd = 0x00`（bus 模式），`amountLD = 500_000_000`
6. `quoteOFT` → `quoteSend`（bus 费用约为 taxi 的 1/3）
7. 执行 sendToken（`msg.value = nativeFee` 仅消息费）
8. 轮询状态，bus 模式需等待 2–10 笔凑团后发送

**预期结果**：节省约 60–70% LayerZero 消息费，~5–20 分钟到账。

---

### 场景 3：USDT 从 BNB Chain 跨链到 Avalanche

**用户意图**："将 1000 USDT 从 BSC 转到 Avalanche"

**执行流程**：
1. src pool：BNB USDT `0x138EB30f73BC423c6455C53df6D89CB01d9eBc63`（EID 30102）
2. dst EID：Avalanche = 30106
3. 调用 `token()` 获取 USDT 地址，检查并执行 approve（若 allowance 不足）
4. Taxi 模式（`oftCmd = 0x`），构造 SendParam：`amountLD = 1_000_000_000`（USDT 6位）
5. 两步 quote → 获取 minAmountLD 和 nativeFee（BNB 计价）
6. sendToken：
   ```bash
   onchainos wallet contract-call \
     --chain-id 56 \
     --address 0x138EB30f73BC423c6455C53df6D89CB01d9eBc63 \
     --value <nativeFee_in_BNB_wei> \
     --input-data <calldata>
   ```
7. 查询状态：`GET /messages/tx/{txHash}`

**预期结果**：约 2–5 分钟在 Avalanche 收到 USDT（扣除协议费，约 0.06% 左右）。

---

### 场景 4：查询跨链进度

**用户意图**："查一下我刚才那笔桥接交易状态"

**执行流程**：
1. 用户提供 txHash 或从对话上下文获取
2. 调用 LayerZero Scan API：
   ```bash
   GET https://scan.layerzero-api.com/v1/messages/tx/0x...
   ```
3. 解析 `status` 字段并展示：
   - INFLIGHT → "交易已发出，等待来源链确认中..."
   - CONFIRMING → "目标链已收到，等待最终确认..."
   - DELIVERED → "跨链成功！目标链 txHash: 0x..."
   - PAYLOAD_STORED → "到达目标链但执行失败，需要重试"

---

## 4. 外部 API 依赖

| API                  | 用途             | Base URL                              | 认证     |
|----------------------|------------------|---------------------------------------|----------|
| LayerZero Scan API   | 查询消息/交易状态 | `https://scan.layerzero-api.com/v1`   | 无需 Key |
| EVM RPC (各链)       | eth_call (quote) | 各链 RPC 端点                         | 视提供商 |
| EVM RPC (各链)       | eth_sendRawTx    | 经 onchainos wallet contract-call     | onchainos |

**LayerZero Scan API 核心端点**：

| Method | Path                         | 说明                     |
|--------|------------------------------|--------------------------|
| GET    | `/messages/tx/{txHash}`      | 按来源 txHash 查消息     |
| GET    | `/messages/wallet/{address}` | 按用户地址查历史记录     |
| GET    | `/messages/guid/{guid}`      | 按 LayerZero GUID 查消息 |
| GET    | `/messages/latest`           | 查最新消息列表           |

**Swagger 文档**：`https://scan.layerzero-api.com/v1/swagger`

---

## 5. 配置参数

| 参数名                    | 类型    | 说明                                           | 示例值                          |
|---------------------------|---------|------------------------------------------------|---------------------------------|
| `src_chain_id`            | u64     | 来源链 EVM Chain ID                            | 1 (Ethereum)                    |
| `dst_chain_id`            | u64     | 目标链 EVM Chain ID                            | 42161 (Arbitrum)                |
| `dst_eid`                 | u32     | 目标链 LayerZero Endpoint ID                   | 30110                           |
| `token_symbol`            | String  | 跨链 token 类型                                | "USDC" / "USDT" / "ETH"        |
| `amount`                  | String  | 金额（人类可读，如 "100.5"）                    | "100.5"                         |
| `receiver`                | Address | 目标链接收地址（默认为 caller）                 | "0xAbcd..."                     |
| `mode`                    | String  | 传输模式："taxi"（快）/ "bus"（便宜）           | "taxi"                          |
| `slippage_bps`            | u32     | 最大滑点（基点，1bps=0.01%），用于计算 minAmountLD | 50 (0.5%)                    |
| `poll_interval_secs`      | u64     | 状态轮询间隔（秒）                              | 10                              |
| `poll_timeout_secs`       | u64     | 轮询超时（秒）                                  | 900 (taxi) / 3600 (bus)        |
| `layerzero_scan_base_url` | String  | LayerZero Scan API 地址                        | `https://scan.layerzero-api.com/v1` |

---

## 6. 关键函数 Selector 汇总

| 函数                                                                                         | Selector     | 验证命令                                                                                             |
|----------------------------------------------------------------------------------------------|--------------|------------------------------------------------------------------------------------------------------|
| `sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)`    | `0xcbef2aa9` | `cast sig "sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)"` |
| `quoteOFT((uint32,bytes32,uint256,uint256,bytes,bytes,bytes))`                               | `0x0d35b415` | `cast sig "quoteOFT((uint32,bytes32,uint256,uint256,bytes,bytes,bytes))"`                            |
| `quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)`                         | `0x3b6f743b` | `cast sig "quoteSend((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),bool)"`                      |
| `token()`                                                                                    | `0xfc0c546a` | `cast sig "token()"`                                                                                 |
| `allowance(address,address)`                                                                 | `0xdd62ed3e` | `cast sig "allowance(address,address)"`                                                              |
| `approve(address,uint256)`                                                                   | `0x095ea7b3` | `cast sig "approve(address,uint256)"`                                                                |

---

## 7. 注意事项

1. **地址格式转换**：`address` → `bytes32` 需左补零：`bytes32(uint256(uint160(addr)))`
2. **Native ETH 池识别**：调用 `token()` 返回 `address(0)` 时为原生 ETH 池，无需 approve，但 `msg.value` 须加上 `amountLD`
3. **Bus 模式 ticketId**：`sendToken` 返回 `Ticket.ticketId`，可通过 `busQueues[dstEid].nextTicketId` 判断是否已发车
4. **quoteSend 须紧随 sendToken 调用**：fee 有效期约 1 个区块，生产环境建议 quote 与 send 在同一交易或同一区块
5. **精度**：USDC/USDT 为 6 位小数，ETH 为 18 位小数，amountLD 单位为 token 最小单位
6. **滑点保护**：`minAmountLD` 建议设为 `receipt.amountReceivedLD * (10000 - slippage_bps) / 10000`
7. **EID vs Chain ID**：LayerZero EID 与 EVM Chain ID 不同，桥接时目标链用 EID，onchainos 调用来源链用 Chain ID

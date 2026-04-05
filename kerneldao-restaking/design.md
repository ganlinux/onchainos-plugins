# KernelDAO Restaking — Plugin Store 接入 PRD

## 0. Plugin Meta

| Field            | Value                                      |
|------------------|--------------------------------------------|
| plugin_name      | `kerneldao-restaking`                      |
| dapp_name        | KernelDAO Restaking                        |
| category         | defi-protocol                              |
| tags             | restaking, btc-defi, bsc, bnb, btcb       |
| target_chains    | bsc                                        |
| target_protocols | KernelDAO                                  |
| priority         | P1                                         |
| onchainos        | 是                                          |

---

## 1. Background

### 1.1 DApp 介绍

KernelDAO 是一个多链 Restaking 协议，聚焦于 BNB Chain 上的 BNB 及 BTC 衍生资产 Restaking。协议于 2024 年 12 月主网上线，TVL 迅速突破 $50M，当前超过 $2B。

核心产品：
- **Kernel**（BNB Chain）：BNB、LST、BTC 衍生品的 Restaking
- **Kelp**（Ethereum）：ETH/LST Restaking，铸造 rsETH
- **Gain**：非托管收益聚合 Vault

本插件专注 **Kernel（BSC）** 部分，支持 BTC 衍生资产（BTCB、SolvBTC、stBTC、pumpBTC、uniBTC 等）和 BNB 衍生资产（WBNB、slisBNB、BNBx、asBNB 等）的 Restaking。

### 1.2 接入可行性调研

| 维度             | 结论                                                                 |
|----------------|----------------------------------------------------------------------|
| 是否有 SDK       | 无                                                                   |
| 是否有 REST API  | 无（合约直接交互）                                                      |
| 合约是否开源     | 是（[GitHub: Kelp-DAO/kernel-smart-contracts-public](https://github.com/Kelp-DAO/kernel-smart-contracts-public)） |
| 合约是否经过审计 | 是（ChainSecurity、SigmaPrime 审计）                                   |
| 主部署链         | BSC Mainnet（主力链），Ethereum/Arbitrum 为 Kelp/Gain，不在本插件范围     |
| 接入路径         | 直接合约交互（encod calldata，通过 onchainos wallet contract-call 广播）  |
| 取款是否有锁定期 | 是，7-14 天解绑期（依链而定）                                            |
| 是否需要 Approve | 是，ERC-20 资产质押前须 approve StakerGateway；原生 BNB 不需要            |

### 1.3 接入路径

无 SDK / 无 REST API → **直接合约交互**

所有链上操作通过 `onchainos wallet contract-call` 执行：
```
onchainos wallet contract-call \
  --address <contract_address> \
  --input-data <hex_calldata>
```

链下查询通过 `eth_call`（onchainos 或标准 RPC）执行。

---

## 2. DApp 核心能力 & 接口映射

### 2.1 核心合约地址（BSC Mainnet）

| 合约名称         | 地址                                         | 说明                              |
|----------------|----------------------------------------------|-----------------------------------|
| StakerGateway  | `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394` | 用户交互入口（ERC1967 Proxy）        |
| AssetRegistry  | `0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB` | 支持资产注册表（ERC1967 Proxy）      |
| KernelConfig   | `0x45d7Bb73253A908E6160aa5FD9DA083F7Bc6faf5` | 协议配置                           |

来源：[KernelDAO GitBook - Smart Contracts](https://kerneldao.gitbook.io/kernel/getting-started/kernel/smart-contracts)

### 2.2 支持资产列表（BSC Mainnet）

| 资产类型 | Token 名称   | Token 地址                                     | Vault 地址                                     |
|--------|-------------|------------------------------------------------|------------------------------------------------|
| BNB    | WBNB        | `0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c`  | `0xF4b14b29f2a2f76BA6BD4dD45e107eb34D6bb7ec`  |
| BNB    | slisBNB     | `0xb0b84d294e0c75a6abe60171b70edeb2efd14a1b`  | `0xBFA21d5c2a6C400e1A01b1f8EAa5224450d1cDC3`  |
| BNB    | BNBx        | `0x1bdd3cf7f79cfb8edbb955f20ad99211551ba275`  | `0xf49A3752713116c38136e3a4DaDe609B9b36691c`  |
| BNB    | asBNB       | `0x77734e70b6e88b4d82fe632a168edf6e700912b6`  | `0xC21A4Ee664F56C6e9d57302e6a060665317cD52d`  |
| BTC    | BTCB        | `0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c`  | `0xFa185f2bbfC32146424C879FC772020E4636d2f1`  |
| BTC    | SolvBTC     | `0x4aae823a6a0b376de6a78e74ecc5b079d38cbcf7`  | `0x13518474faAa73E2399aDD8F014BA21E2E299465`  |
| BTC    | SolvBTC.BBN | `0x1346b618dc92810ec74163e4c27004c921d446a5`  | `0x64ab78576a1ea528e31eb290Df874320448C3B72`  |
| BTC    | uniBTC      | `0x6b2a01a5f79deb4c2f3c0eda7b01df456fbd726a`  | `0x015937Cd0614B205D0Ecea3259A8E280a8d61848`  |
| BTC    | stBTC       | `0xf6718b2701d4a6498ef77d7c152b2137ab28b8a3`  | `0xe6E56756a9f48D3871F52CBE841346AB5047865D`  |
| BTC    | pumpBTC     | `0xf9C4FF105803A77eCB5DAE300871Ad76c2794fa4`  | `0x799592457b9987C9c4949ED391AFea87FBcb6972`  |
| BTC    | mBTC        | `0x9BFA177621119e64CecbEabE184ab9993E2ef727`  | `0x4F49f1d480D48AF660b7f4506bbB785AD5648726`  |

### 2.3 操作列表

#### 操作 1：查询用户质押余额（链下查询）

**用途**：查询用户在某资产 Vault 中的质押余额。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | StakerGateway `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`        |
| 函数签名（canonical）| `balanceOf(address,address)`                                  |
| Selector       | `0xf7888aec` ✅（`cast sig "balanceOf(address,address)"`）          |
| ABI 参数顺序     | `(address asset, address owner)`                                   |
| 返回值           | `uint256` — 用户在该资产 Vault 中的质押量（原始精度）                   |
| 调用方式         | eth_call（只读，无需签名）                                             |

**调用示例（BTCB 余额查询）**：
```bash
# asset = BTCB: 0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c
# owner = 用户地址
onchainos rpc eth_call \
  --to 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --data 0xf7888aec \
        0000000000000000000000007130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c \
        000000000000000000000000<user_address_padded>
```

---

#### 操作 2：查询支持资产列表（链下查询）

**用途**：获取协议当前支持的所有 ERC-20 资产地址列表。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | AssetRegistry `0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB`        |
| 函数签名（canonical）| `getAssets()`                                                 |
| Selector       | `0x67e4ac2c` ✅（`cast sig "getAssets()"`）                         |
| ABI 参数顺序     | 无参数                                                              |
| 返回值           | `address[]` — 所有支持的 token 地址数组                               |
| 调用方式         | eth_call（只读）                                                     |

**调用示例**：
```bash
onchainos rpc eth_call \
  --to 0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB \
  --data 0x67e4ac2c
```

---

#### 操作 3：ERC-20 Approve（链上操作，质押前置步骤）

**用途**：授权 StakerGateway 从用户地址转移 ERC-20 资产（BTCB、SolvBTC 等），是 `stake` 的必要前置步骤。原生 BNB 质押（`stakeNative`）无需此步骤。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | ERC-20 Token 合约（如 BTCB `0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c`） |
| 函数签名（canonical）| `approve(address,uint256)`                               |
| Selector       | `0x095ea7b3` ✅（`cast sig "approve(address,uint256)"`）             |
| ABI 参数顺序     | `(address spender, uint256 amount)`                                |
| spender        | StakerGateway `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`        |
| amount         | 质押数量（uint256，含 token decimals）；可设为 `type(uint256).max` 无限授权 |
| 调用方式         | onchainos wallet contract-call（需用户签名）                          |

**onchainos 命令**：
```bash
# 示例：approve BTCB 0.001 BTC (decimals=18, amount=1000000000000000)
onchainos wallet contract-call \
  --address 0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c \
  --input-data 0x095ea7b3 \
               000000000000000000000000b32df5b33dbcca60437ec17b27842c12bfe83394 \
               00000000000000000000000000000000000000000000000000038d7ea4c68000
```

---

#### 操作 4：质押 ERC-20 资产（Stake，链上操作）

**用途**：将 ERC-20 资产（BTC/BNB 衍生品）质押至 KernelDAO，获取 Kernel Points 奖励。需先完成 Approve（操作 3）。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | StakerGateway `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`        |
| 函数签名（canonical）| `stake(address,uint256,string)`                               |
| Selector       | `0x4df42566` ✅（`cast sig "stake(address,uint256,string)"`）        |
| ABI 参数顺序     | `(address asset, uint256 amount, string referralId)`               |
| asset          | 被质押的 ERC-20 token 地址（如 BTCB）                                 |
| amount         | 质押数量（uint256，含 token decimals）                                 |
| referralId     | 推荐码字符串，无推荐时传空字符串 `""`                                    |
| 调用方式         | onchainos wallet contract-call（需用户签名）                          |

**onchainos 命令**：
```bash
# 示例：stake 0.001 BTCB
# calldata = selector + abi.encode(asset, amount, referralId)
# referralId="" → 空字符串 ABI 编码：offset=0x60, length=0, no data
onchainos wallet contract-call \
  --address 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --input-data <abi_encoded_calldata>
```

> 插件实现时使用 Rust `ethabi` crate 编码 calldata，再传入 `--input-data`。

---

#### 操作 5：质押原生 BNB（StakeNative，链上操作）

**用途**：直接质押原生 BNB（无需 approve），传入 ETH value 即为质押金额。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | StakerGateway `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`        |
| 函数签名（canonical）| `stakeNative(string)`                                         |
| Selector       | `0xc412056b` ✅（`cast sig "stakeNative(string)"`）                  |
| ABI 参数顺序     | `(string referralId)`                                              |
| referralId     | 推荐码字符串，无推荐时传空字符串 `""`                                    |
| msg.value      | 质押的 BNB 数量（wei）                                               |
| 调用方式         | onchainos wallet contract-call --value <bnb_in_wei>               |

**onchainos 命令**：
```bash
onchainos wallet contract-call \
  --address 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --value 1000000000000000 \
  --input-data <abi_encoded_calldata>
```

---

#### 操作 6：取消质押 ERC-20（Unstake，链上操作）

**用途**：发起 ERC-20 资产解除质押请求。解绑期约 7-14 天，到期后可提取。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | StakerGateway `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`        |
| 函数签名（canonical）| `unstake(address,uint256,string)`                             |
| Selector       | `0xf91daa33` ✅（`cast sig "unstake(address,uint256,string)"`）      |
| ABI 参数顺序     | `(address asset, uint256 amount, string referralId)`               |
| asset          | 被取消质押的 ERC-20 token 地址                                        |
| amount         | 取消质押数量（uint256，含 token decimals）                              |
| referralId     | 推荐码字符串，通常传空字符串 `""`                                        |
| 调用方式         | onchainos wallet contract-call（需用户签名）                          |

**onchainos 命令**：
```bash
onchainos wallet contract-call \
  --address 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --input-data <abi_encoded_calldata>
```

---

#### 操作 7：取消质押原生 BNB（UnstakeNative，链上操作）

**用途**：发起原生 BNB 解除质押请求。

| 字段             | 值                                                                 |
|----------------|--------------------------------------------------------------------|
| 合约             | StakerGateway `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`        |
| 函数签名（canonical）| `unstakeNative(uint256,string)`                               |
| Selector       | `0x4693cf07` ✅（`cast sig "unstakeNative(uint256,string)"`）        |
| ABI 参数顺序     | `(uint256 amount, string referralId)`                              |
| amount         | 取消质押的 BNB 数量（wei）                                            |
| referralId     | 推荐码字符串，通常传空字符串 `""`                                        |
| 调用方式         | onchainos wallet contract-call（需用户签名）                          |

---

### 2.4 函数 Selector 汇总

| 函数                           | Canonical 签名                          | Selector     | cast sig 验证 |
|-------------------------------|----------------------------------------|--------------|--------------|
| ERC-20 Approve                | `approve(address,uint256)`             | `0x095ea7b3` | ✅            |
| ERC-20 Allowance              | `allowance(address,address)`           | `0xdd62ed3e` | ✅            |
| StakerGateway.stake           | `stake(address,uint256,string)`        | `0x4df42566` | ✅            |
| StakerGateway.stakeNative     | `stakeNative(string)`                  | `0xc412056b` | ✅            |
| StakerGateway.unstake         | `unstake(address,uint256,string)`      | `0xf91daa33` | ✅            |
| StakerGateway.unstakeNative   | `unstakeNative(uint256,string)`        | `0x4693cf07` | ✅            |
| StakerGateway.balanceOf       | `balanceOf(address,address)`           | `0xf7888aec` | ✅            |
| AssetRegistry.getAssets       | `getAssets()`                          | `0x67e4ac2c` | ✅            |
| AssetRegistry.getVault        | `getVault(address)`                    | `0x0eb9af38` | ✅            |
| AssetRegistry.getVaultBalance | `getVaultBalance(address)`             | `0xd3d7c002` | ✅            |
| AssetRegistry.hasAsset        | `hasAsset(address)`                    | `0xa567fb47` | ✅            |

---

## 3. 用户场景

### 场景 1：用户质押 BTCB 获取 Kernel Points

**背景**：用户持有 BTCB，希望通过 KernelDAO 参与 BTC Restaking，赚取 Kernel Points（可能兑换 KERNEL 代币奖励）。

**操作流程**：
1. **查询支持资产**（链下）：调用 `AssetRegistry.getAssets()` 确认 BTCB 是否在支持列表
2. **查询当前余额**（链下）：调用 `StakerGateway.balanceOf(BTCB, userAddr)` 查看当前质押量
3. **授权**（链上）：调用 BTCB ERC-20 合约 `approve(StakerGateway, amount)`
4. **质押**（链上）：调用 `StakerGateway.stake(BTCB_addr, amount, "")` 完成质押
5. **确认**：再次查询 `balanceOf` 确认质押成功

**onchainos 调用序列**：
```bash
# Step 3: Approve BTCB
onchainos wallet contract-call \
  --address 0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c \
  --input-data <approve_calldata>

# Step 4: Stake BTCB
onchainos wallet contract-call \
  --address 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --input-data <stake_calldata>
```

---

### 场景 2：用户质押原生 BNB

**背景**：用户持有原生 BNB，希望直接质押至 KernelDAO 无需兑换为 LST。

**操作流程**：
1. **确认链**：确保用户在 BSC 网络
2. **质押 BNB**（链上）：调用 `StakerGateway.stakeNative("")`，附带 msg.value = 质押 BNB 数量（wei）
3. **确认**：查询 WBNB vault 余额或监听 tx receipt

**onchainos 调用**：
```bash
onchainos wallet contract-call \
  --address 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --value 1000000000000000000 \
  --input-data <stakeNative_calldata>
```

---

### 场景 3：用户查询质押持仓概览

**背景**：用户希望查看自己在 KernelDAO 中所有资产的质押余额。

**操作流程**：
1. **获取资产列表**（链下）：调用 `AssetRegistry.getAssets()` 获取所有支持资产地址
2. **批量查询余额**（链下）：对每个资产地址调用 `StakerGateway.balanceOf(asset, userAddr)`
3. **过滤非零余额**：返回用户有持仓的资产列表及对应数量
4. **格式化展示**：将余额按 token decimals 格式化后展示给用户

**onchainos 调用**（示例查询 BTCB 和 SolvBTC）：
```bash
# Query BTCB balance
onchainos rpc eth_call --to 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --data <balanceOf_BTCB_calldata>

# Query SolvBTC balance
onchainos rpc eth_call --to 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --data <balanceOf_SolvBTC_calldata>
```

---

### 场景 4：用户发起取消质押 BTCB

**背景**：用户希望从 KernelDAO 取回已质押的 BTCB，接受 7-14 天解绑等待期。

**操作流程**：
1. **查询当前质押余额**（链下）：`StakerGateway.balanceOf(BTCB, userAddr)` 确认可赎回数量
2. **发起 Unstake**（链上）：调用 `StakerGateway.unstake(BTCB_addr, amount, "")`
3. **提示用户**：说明解绑期约 7-14 天，到期后需另行操作提取

**onchainos 调用**：
```bash
onchainos wallet contract-call \
  --address 0xb32dF5B33dBCCA60437EC17b27842c12bFE83394 \
  --input-data <unstake_calldata>
```

---

## 4. 外部 API 依赖

| 依赖类型       | 说明                                                                 |
|-------------|----------------------------------------------------------------------|
| BSC RPC     | 标准 EVM JSON-RPC，用于 eth_call 查询；onchainos 内置                   |
| 合约地址硬编码 | StakerGateway、AssetRegistry 地址在插件中配置，来源于官方文档               |
| 资产列表      | 从链上 `AssetRegistry.getAssets()` 动态获取，或在插件中维护白名单             |
| 无第三方 API  | 不依赖 KernelDAO 中心化 API，完全链上交互                                  |

---

## 5. 配置参数

| 参数名                    | 类型     | 默认值                                         | 说明                          |
|--------------------------|--------|------------------------------------------------|-------------------------------|
| `chain_id`               | u64    | `56`                                           | BSC Mainnet chain ID          |
| `staker_gateway_address` | String | `0xb32dF5B33dBCCA60437EC17b27842c12bFE83394`  | StakerGateway 合约地址         |
| `asset_registry_address` | String | `0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB`  | AssetRegistry 合约地址         |
| `referral_id`            | String | `""`                                           | 默认推荐码（可由插件运营商配置）    |
| `default_slippage`       | f64    | `0.005`                                        | 预留参数（当前合约无滑点控制）      |

---

## 6. 实现注意事项

### 6.1 ABI 编码

- 使用 Rust `ethabi` crate 或 `alloy-sol-types` 编码 calldata
- `string` 类型参数（`referralId`）需按 ABI 动态类型规范编码：offset + length + data（32字节对齐）
- 空字符串 `""` 的 ABI 编码（在 `stake(address,uint256,string)` 中，string 在第 3 个位置）：
  - offset = `0x60`（96 bytes，3个参数 head 区域之后）
  - length = `0x00`
  - 无 data 部分

### 6.2 Approve 前检查

建议在 `stake` 前先 eth_call `allowance(user, StakerGateway)`，若 allowance 不足才发送 approve 交易，避免不必要的 gas 消耗。

### 6.3 原生 BNB vs ERC-20

- 原生 BNB：`stakeNative(string)` / `unstakeNative(uint256,string)`，需设置 `--value`
- ERC-20（WBNB、BTCB 等）：`stake(address,uint256,string)` / `unstake(address,uint256,string)`，需先 `approve`

### 6.4 解绑期处理

unstake 后有 7-14 天解绑期，插件目前仅支持发起 unstake 请求。到期提取（claim/withdraw）的函数待进一步调研（合约可能有独立的 `claim` 函数）。

### 6.5 合约升级

StakerGateway 和 AssetRegistry 均为 ERC1967 Proxy，实现合约可能升级。插件应使用 Proxy 地址（而非 implementation 地址）进行所有调用。

---

## 7. 参考资料

- [KernelDAO 官网](https://kerneldao.com)
- [KernelDAO GitBook 文档](https://kerneldao.gitbook.io/kernel/)
- [Smart Contracts 地址文档](https://kerneldao.gitbook.io/kernel/getting-started/kernel/smart-contracts)
- [GitHub: Kelp-DAO/kernel-smart-contracts-public](https://github.com/Kelp-DAO/kernel-smart-contracts-public)
- [BSCScan: StakerGateway](https://bscscan.com/address/0xb32dF5B33dBCCA60437EC17b27842c12bFE83394)
- [BSCScan: AssetRegistry](https://bscscan.com/address/0xd0B91Fc0a323bbb726faAF8867CdB1cA98c44ABB)

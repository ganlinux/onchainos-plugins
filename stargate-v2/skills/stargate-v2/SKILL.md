---
name: stargate-v2
version: "0.1.0"
binary: stargate-v2
description: "Cross-chain bridge skill for Stargate V2 / LayerZero V2. Supports ETH, USDC, USDT bridging across 10+ EVM chains."
---

# stargate-v2

## description

Cross-chain bridge skill for Stargate V2 (LayerZero V2). Supports bridging ETH, USDC, and USDT across Ethereum, Arbitrum, Optimism, Base, Polygon, BNB Chain, Avalanche, Mantle, Linea, Scroll, Metis, and Kava. Provides on-chain sendToken execution, fee quotes, ERC-20 approve, and LayerZero transaction status tracking.

## commands

### quote

Get a cross-chain bridge quote showing expected fees and received amount before executing.

```
stargate-v2 quote --src-chain <CHAIN> --dst-chain <CHAIN> --token <TOKEN> --amount <AMOUNT> [--mode taxi|bus] [--receiver <ADDR>] [--rpc <URL>]
```

**Parameters:**
- `--src-chain`: Source chain name or ID (e.g. `ethereum`, `arbitrum`, `42161`)
- `--dst-chain`: Destination chain name or ID
- `--token`: Token symbol: `ETH`, `USDC`, or `USDT`
- `--amount`: Human-readable amount (e.g. `100.5`)
- `--mode`: `taxi` (fast, default) or `bus` (cheap, batch)
- `--receiver`: Optional destination address (defaults to sender)
- `--rpc`: Optional custom RPC endpoint for source chain

**Examples:**
```
stargate-v2 quote --src-chain ethereum --dst-chain arbitrum --token ETH --amount 0.1
stargate-v2 quote --src-chain arbitrum --dst-chain polygon --token USDC --amount 500 --mode bus
stargate-v2 quote --src-chain 56 --dst-chain 43114 --token USDT --amount 1000
```

**Output:** Expected received amount, protocol fee, LayerZero messaging fee, and mode details.

---

### send

Execute a cross-chain token transfer via Stargate V2. This command:
1. Calls `quoteOFT` to get expected received amount
2. Calls `quoteSend` to get LayerZero messaging fee
3. If ERC-20 token: checks allowance, executes `approve` if needed
4. Executes `sendToken` on the Stargate pool contract

**ask user to confirm** each on-chain transaction (approve and sendToken) before submitting.

```
stargate-v2 send --src-chain <CHAIN> --dst-chain <CHAIN> --token <TOKEN> --amount <AMOUNT> [--mode taxi|bus] [--receiver <ADDR>] [--slippage-bps <BPS>] [--rpc <URL>] [--dry-run]
```

**Parameters:**
- `--src-chain`: Source chain name or ID
- `--dst-chain`: Destination chain name or ID
- `--token`: Token symbol: `ETH`, `USDC`, or `USDT`
- `--amount`: Human-readable amount (e.g. `100.5`)
- `--mode`: `taxi` (fast, default) or `bus` (cheap)
- `--receiver`: Destination address (defaults to sender wallet)
- `--slippage-bps`: Max slippage in basis points, default `50` (0.5%)
- `--rpc`: Optional custom RPC for source chain
- `--dry-run`: Build and print calldata without submitting transactions

**Examples:**
```
stargate-v2 send --src-chain ethereum --dst-chain arbitrum --token ETH --amount 0.1
stargate-v2 send --src-chain arbitrum --dst-chain polygon --token USDC --amount 500 --mode bus
stargate-v2 send --src-chain 56 --dst-chain 43114 --token USDT --amount 1000 --dry-run
```

**On-chain operations (each requires user confirmation):**

1. **ERC-20 approve** (only for USDC/USDT pools when allowance is insufficient):
   - ask user to confirm the approve transaction before submitting
   - Contract: ERC-20 token address (USDC or USDT)
   - Spender: Stargate pool address
   - Uses `onchainos wallet contract-call --chain <chain_id> --to <token_addr> --input-data <approve_calldata>`

2. **sendToken** (cross-chain bridge execution):
   - ask user to confirm the sendToken transaction before submitting
   - Contract: Stargate pool address on source chain
   - Function: `sendToken((uint32,bytes32,uint256,uint256,bytes,bytes,bytes),(uint256,uint256),address)` selector `0xcbef2aa9`
   - Uses `onchainos wallet contract-call --chain <chain_id> --to <pool_addr> --input-data <calldata> --amt <msg_value_wei>`
   - `msg.value` = LayerZero fee (ERC-20 pools) or LayerZero fee + bridge amount (native ETH pools)

**Output:** Transaction hash; use `stargate-v2 status --tx-hash <hash>` to track delivery.

---

### status

Query LayerZero cross-chain message status using LayerZero Scan API.

```
stargate-v2 status --tx-hash <TX_HASH>
stargate-v2 status --wallet <ADDRESS> [--limit <N>]
stargate-v2 status --guid <GUID>
```

**Parameters:**
- `--tx-hash`: Source chain transaction hash from `sendToken`
- `--wallet`: Wallet address to query message history
- `--guid`: LayerZero message GUID
- `--limit`: Max records for wallet history (default: 10)
- `--scan-url`: Override LayerZero Scan API base URL

**Examples:**
```
stargate-v2 status --tx-hash 0xabc123...
stargate-v2 status --wallet 0xYourAddress --limit 5
```

**Status values:**
- `INFLIGHT`: Waiting for source chain confirmation
- `CONFIRMING`: Received on destination, awaiting finality
- `DELIVERED`: Transfer complete, funds on destination chain
- `FAILED`: Transaction failed
- `PAYLOAD_STORED`: Arrived but execution reverted, can retry
- `BLOCKED`: Blocked by pending nonce

---

### pools

List all supported Stargate V2 pools, assets, and chains.

```
stargate-v2 pools [--chain <CHAIN>] [--token <TOKEN>]
```

**Parameters:**
- `--chain`: Filter by chain name or ID (optional)
- `--token`: Filter by token symbol (optional)

**Examples:**
```
stargate-v2 pools
stargate-v2 pools --chain arbitrum
stargate-v2 pools --token USDC
stargate-v2 pools --chain ethereum --token ETH
```

**Output:** Table of chains, chain IDs, LayerZero EIDs, pool addresses, and token types.

---

## supported-chains

| Chain       | Chain ID | LayerZero EID |
|-------------|----------|---------------|
| Ethereum    | 1        | 30101         |
| BNB Chain   | 56       | 30102         |
| Avalanche   | 43114    | 30106         |
| Polygon     | 137      | 30109         |
| Arbitrum    | 42161    | 30110         |
| OP Mainnet  | 10       | 30111         |
| Mantle      | 5000     | 30181         |
| Base        | 8453     | 30184         |
| Linea       | 59144    | 30183         |
| Scroll      | 534352   | 30214         |

## notes

- Native ETH pools do not require ERC-20 approve. msg.value includes bridge amount plus LayerZero fee.
- ERC-20 pools (USDC, USDT) require approve before sendToken.
- Taxi mode: immediate delivery, ~1-3 minutes, higher fee.
- Bus mode: batch aggregation, ~5-20 minutes, ~60-70% cheaper.
- Always ask user to confirm on-chain transactions before submitting.
- LayerZero fee quote is valid for approximately 1 block; quote and send should be close in time.

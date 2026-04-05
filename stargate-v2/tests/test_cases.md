# Stargate V2 Test Cases

## L2 — Read-Only Tests (no wallet required)

### TC-L2-01: pools (no filter)
```
./target/release/stargate-v2 pools
```
Expected: Table listing all 16 pools across Ethereum, Arbitrum, OP, Base, Polygon, BNB, Avalanche. No error.

### TC-L2-02: pools --chain arbitrum
```
./target/release/stargate-v2 pools --chain arbitrum
```
Expected: Shows ETH, USDC, USDT pools for Arbitrum (chain ID 42161, EID 30110).

### TC-L2-03: pools --token USDC
```
./target/release/stargate-v2 pools --token USDC
```
Expected: Shows only USDC pools across all chains.

### TC-L2-04: quote Arbitrum -> Base USDC
```
./target/release/stargate-v2 quote --src-chain 42161 --dst-chain 8453 --token USDC --amount 1.0
```
Expected: Returns amountReceived ~1.0 USDC, LayerZero native fee in wei, mode = taxi.

### TC-L2-05: quote Ethereum -> Arbitrum ETH
```
./target/release/stargate-v2 quote --src-chain ethereum --dst-chain arbitrum --token ETH --amount 0.01
```
Expected: Returns expected received ETH amount and fee. No error.

### TC-L2-06: quote bus mode
```
./target/release/stargate-v2 quote --src-chain 42161 --dst-chain 8453 --token USDC --amount 10.0 --mode bus
```
Expected: Returns quote in bus mode (batch, ~5-20 min label shown).

## L3 — Dry-Run Tests

### TC-L3-01: send dry-run USDC Arbitrum -> Base
```
./target/release/stargate-v2 send --src-chain 42161 --dst-chain 8453 --token USDC --amount 1.0 --dry-run
```
Expected:
- sendToken calldata starts with `0xcbef2aa9`
- approve calldata starts with `0x095ea7b3` (if allowance check is bypassed in dry-run, this may not appear — but the logic is correct)
- Output shows "dry_run" mode, zero tx hash, no actual submission

### TC-L3-02: send dry-run ETH Arbitrum -> Base (native, no approve)
```
./target/release/stargate-v2 send --src-chain 42161 --dst-chain 8453 --token ETH --amount 0.001 --dry-run
```
Expected:
- No ERC-20 approve step (native pool)
- sendToken calldata starts with `0xcbef2aa9`
- msg.value = native_fee + amount_ld

### TC-L3-03: send dry-run bus mode
```
./target/release/stargate-v2 send --src-chain 42161 --dst-chain 8453 --token USDC --amount 5.0 --mode bus --dry-run
```
Expected: Calldata includes bus mode (oftCmd byte 0x00 present), starts with `0xcbef2aa9`.

## L4 — On-Chain Tests

### TC-L4-01: quote live (real RPC call)
```
./target/release/stargate-v2 quote --src-chain 42161 --dst-chain 8453 --token USDC --amount 1.0
```
Expected: Live RPC response with non-zero LayerZero fee. amountReceived slightly less than 1.0 USDC.

### TC-L4-02: send USDC 1.0 Arbitrum -> Base (live)
```
./target/release/stargate-v2 send --src-chain 42161 --dst-chain 8453 --token USDC --amount 1.0
```
Pre-condition: Wallet has >= 1.0 USDC and >= 0.001 ETH on Arbitrum.
Expected: approve tx hash (if needed), sendToken tx hash. Track with status command.

### TC-L4-03: status check
```
./target/release/stargate-v2 status --tx-hash <hash_from_TC-L4-02>
```
Expected: INFLIGHT or DELIVERED status from LayerZero Scan API.

## L1-Error — Error Handling Tests

### TC-ERR-01: unsupported chain
```
./target/release/stargate-v2 quote --src-chain 99999 --dst-chain 8453 --token USDC --amount 1.0
```
Expected: Error "Unsupported destination chain" or pool not found. Non-zero exit code.

### TC-ERR-02: unsupported token on chain
```
./target/release/stargate-v2 quote --src-chain 42161 --dst-chain 8453 --token UNKNOWN --amount 1.0
```
Expected: Error "No Stargate pool for UNKNOWN on chain 42161". Non-zero exit code.

### TC-ERR-03: invalid amount
```
./target/release/stargate-v2 quote --src-chain 42161 --dst-chain 8453 --token USDC --amount 1.1234567
```
Expected: Error "Too many decimal places for this token (max 6)". Non-zero exit code.

### TC-ERR-04: status with bad tx hash
```
./target/release/stargate-v2 status --tx-hash 0xinvalid
```
Expected: API error or empty result, graceful error message.

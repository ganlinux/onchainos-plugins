# Kamino Lend Plugin — Test Cases

Generated: 2026-04-05
Plugin: `kamino-lend`
Chain: Solana (chain ID 501)

---

## Level 1 — Compile + Lint

| TC-L1-1 | `cargo build` succeeds with 0 errors |
| TC-L1-2 | `cargo clean && plugin-store lint .` reports 0 errors |

---

## Level 2 — Read-Only (No Wallet, No Gas)

### TC-L2-1: markets — List all Kamino markets
```bash
./target/release/kamino-lend markets
```
**Expected**: JSON array of markets, each with a pubkey and name field.

### TC-L2-2: reserves — Main market reserves (default)
```bash
./target/release/kamino-lend reserves
```
**Expected**: JSON array with at least 3 reserves, each containing `symbol`, `supply_apy`, `borrow_apy`.

### TC-L2-3: reserves — Specific market
```bash
./target/release/kamino-lend reserves --market 7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF
```
**Expected**: Same format as TC-L2-2, scoped to main market.

### TC-L2-4: obligations — Query positions for known wallet (off-chain)
```bash
./target/release/kamino-lend obligations --wallet <WALLET>
```
**Expected**: JSON with obligations list (may be empty if wallet has no positions).

---

## Level 3 — Dry-Run / Simulation (No Broadcast)

All commands use `--dry-run` flag. Expected output includes `"dry_run": true`.

### TC-L3-1: deposit dry-run
```bash
./target/release/kamino-lend deposit \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 1000000 \
  --dry-run
```
**Expected**: `"dry_run": true`, `"serialized_tx"` non-empty (valid base64).

### TC-L3-2: borrow dry-run
```bash
./target/release/kamino-lend borrow \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 100000 \
  --dry-run
```
**Expected**: `"dry_run": true`, transaction data returned or health-factor error.

### TC-L3-3: repay dry-run
```bash
./target/release/kamino-lend repay \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 100000 \
  --dry-run
```
**Expected**: `"dry_run": true`, `"serialized_tx"` non-empty.

### TC-L3-4: withdraw dry-run
```bash
./target/release/kamino-lend withdraw \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 1000000 \
  --dry-run
```
**Expected**: `"dry_run": true`, `"serialized_tx"` non-empty.

---

## Level 4 — On-Chain (Real Broadcast)

Requires: SOL balance >= 0.001 SOL, lock acquired.

### TC-L4-1: obligations — Check wallet positions
```bash
./target/release/kamino-lend obligations --wallet <WALLET>
```
**Expected**: Valid JSON; may be empty obligations list.

### TC-L4-2: deposit — Supply 1 USDC (1,000,000 lamports)
```bash
./target/release/kamino-lend deposit \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 1000000
```
**Expected**: JSON with `txHash`, verify on https://solscan.io

### TC-L4-3: withdraw — Withdraw 1 USDC (only after TC-L4-2 succeeds)
```bash
./target/release/kamino-lend withdraw \
  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 \
  --amount 1000000
```
**Expected**: JSON with `txHash`, verify on https://solscan.io

---

## Reserve Reference (Main Market)

| Token | Reserve Pubkey |
|-------|---------------|
| USDC  | D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 |
| SOL   | d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q |
| JLP   | DdTmCCjv7zHRD1hJv3E8bpnSEQBzdKkzB1j9ApXX5QoP |

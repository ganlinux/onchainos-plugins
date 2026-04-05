# GTBTC Test Cases

## L2 — Read Tests (no wallet needed)

| ID | Command | Expected |
|----|---------|----------|
| L2-01 | `./target/release/gtbtc price` | JSON with `ok:true`, fields: `price_usd`, `change_24h`, `high_24h`, `low_24h` |
| L2-02 | `./target/release/gtbtc apr` | JSON with `ok:true`, field: `apr` or rate info |
| L2-03 | `./target/release/gtbtc balance --chain 1 --address 0x0000000000000000000000000000000000000001` | JSON with `ok:true`, fields: `address`, `chain`, `balance_gtbtc`, `decimals:8` |
| L2-04 | `./target/release/gtbtc --chain 56 balance --address 0x0000000000000000000000000000000000000001` | JSON with `ok:true`, chain=bsc |
| L2-05 | `./target/release/gtbtc --chain 8453 balance --address 0x0000000000000000000000000000000000000001` | JSON with `ok:true`, chain=base |

## L3 — Dry-Run Tests (no wallet, no gas)

| ID | Command | Expected |
|----|---------|----------|
| L3-01 | `./target/release/gtbtc --dry-run transfer --to 0x0000000000000000000000000000000000000001 --amount 0.001 --chain 1` | JSON with `dry_run:true`, calldata starts with `0xa9059cbb`, `amount_atomic:"100000"`, `decimals:8` |
| L3-02 | `./target/release/gtbtc --dry-run approve --spender 0x0000000000000000000000000000000000000001 --amount 0.001 --chain 1` | JSON with `dry_run:true`, calldata starts with `0x095ea7b3`, `amount_atomic:"100000"`, `decimals:8` |
| L3-03 | `./target/release/gtbtc --dry-run approve --spender 0x0000000000000000000000000000000000000001 --chain 1` | JSON with `dry_run:true`, `amount:"unlimited"`, calldata with `ffffffff...` |
| L3-04 | `./target/release/gtbtc --dry-run transfer --to 0x0000000000000000000000000000000000000001 --amount 0.001 --chain 56` | BSC chain_id=56 in output |
| L3-05 | `./target/release/gtbtc --dry-run transfer --to 0x0000000000000000000000000000000000000001 --amount 0.001 --chain 8453` | Base chain_id=8453 in output |

## L4 — On-Chain Tests

| ID | Command | Expected |
|----|---------|----------|
| L4-01 | `./target/release/gtbtc --chain 501 balance --address gtBTCGWvSRYYoZpU9UZj6i3eUGUpgksXzzsbHk2K9So` | JSON with Solana balance (likely 0) |
| L4-02 | `./target/release/gtbtc --dry-run transfer --to ... --amount 0.001 --chain 1` (actual on-chain) | BLOCKED — test wallet has no GTBTC |
| L4-03 | `./target/release/gtbtc --chain 1 approve ...` (actual on-chain) | BLOCKED — test wallet has no GTBTC |

## L1-Error — Invalid Input Tests

| ID | Command | Expected |
|----|---------|----------|
| E-01 | `./target/release/gtbtc --chain 999 balance --address 0x0000000000000000000000000000000000000001` | Error or unknown chain |
| E-02 | `./target/release/gtbtc transfer` (missing --to) | clap error: required arg missing |
| E-03 | `./target/release/gtbtc approve` (missing --spender) | clap error: required arg missing |

# Test Cases — treehouse-protocol

Generated: 2026-04-05

## L1 — Compile + Lint

| ID | Test | Expected |
|----|------|----------|
| L1-01 | `cargo build --release` | Exits 0, binary produced |
| L1-02 | `plugin-store lint .` | "passed all checks" |

## L2 — Read Tests (no gas)

### price command

| ID | Command | Expected |
|----|---------|----------|
| L2-01 | `price --chain 1` | JSON with `ok:true`, `data.token:"tETH"`, `data.price` > 0 |
| L2-02 | `price --chain 43114` | JSON with `ok:true`, `data.token:"tAVAX"`, `data.price` > 0 |

### balance command

| ID | Command | Expected |
|----|---------|----------|
| L2-03 | `balance --chain 1` | JSON with `ok:true`, `data.token:"tETH"`, `data.balance` field present |
| L2-04 | `balance --chain 43114` | JSON with `ok:true`, `data.token:"tAVAX"`, `data.balance` field present |

### positions command

| ID | Command | Expected |
|----|---------|----------|
| L2-05 | `positions --chain 1` | JSON with `ok:true`, position data or empty; no crash |
| L2-06 | `positions --chain 43114` | JSON with `ok:true`, position data or empty; no crash |

## L3 — Dry-run Tests (no gas)

### deposit dry-run

| ID | Command | Expected Selector |
|----|---------|------------------|
| L3-01 | `deposit --token ETH --amount 0.001 --chain 1 --dry-run` | calldata starts with `0xf6326fb3` (depositETH) |
| L3-02 | `deposit --token wstETH --amount 0.001 --chain 1 --dry-run` | step2_deposit calldata starts with `0x47e7ef24`; step1_approve starts with `0x095ea7b3` |
| L3-03 | `deposit --token AVAX --amount 0.1 --chain 43114 --dry-run` | calldata starts with `0xa0d065c3` (depositAVAX) |
| L3-04 | `deposit --token sAVAX --amount 0.1 --chain 43114 --dry-run` | step2_deposit calldata starts with `0x47e7ef24`; step1_approve starts with `0x095ea7b3` |

### withdraw dry-run

| ID | Command | Expected |
|----|---------|----------|
| L3-05 | `withdraw --amount 0.001 --chain 1 --dry-run` | step2_exchange calldata starts with `0x3df02124`; step1_approve starts with `0x095ea7b3` |
| L3-06 | `withdraw --amount 0.001 --chain 43114 --dry-run` | Error: tAVAX withdrawal not supported |

## L4 — On-chain Tests

| ID | Command | Expected |
|----|---------|----------|
| L4-01 | `balance --chain 1` (with real wallet) | Valid JSON, no error |
| L4-02 | ETH deposit on chain 1 | BLOCKED if wallet ETH < 0.001 ETH |
| L4-03 | AVAX deposit on chain 43114 | BLOCKED if wallet AVAX < 0.1 AVAX |

## Error Cases

| ID | Command | Expected |
|----|---------|----------|
| E-01 | `deposit --token ETH --amount 0.001 --chain 999` | Error: Unsupported chain_id |
| E-02 | `deposit --token USDC --amount 1 --chain 1 --dry-run` | Error: Unsupported token |
| E-03 | `withdraw --amount 0.001 --chain 43114 --dry-run` | Error: tAVAX withdrawal not supported |

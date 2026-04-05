# Test Cases — swellchain-staking

Generated: 2026-04-05

## CLI Flags Reference

| Command | Amount flag | Address flag | Token flag |
|---------|-------------|--------------|------------|
| stake | `--amt <wei>` | `--from <addr>` | — |
| earn-deposit | `--amt <wei>` | `--from <addr>` | `--token swETH\|rswETH` |
| earn-withdraw | `--amt <wei>` | `--from <addr>` | `--token swETH\|rswETH` |
| request-withdrawal | `--amt <wei>` | `--from <addr>` | — |
| finalize-withdrawal | `--token-id <id>` | `--from <addr>` | — |
| balance | — | `--address <addr>` | — |
| positions | — | `--address <addr>` | — |

**Note**: The pipeline prompt uses `--amount` but the binary uses `--amt`.

---

## L2: Read Tests (no gas)

### TC-L2-01: Balance Query

```bash
./target/release/swellchain-staking balance --address 0xf951E335afb289353dc249e82926178EaC7DEd78
```

**Expected:**
- Exit code 0
- Valid JSON with `ok: true`
- Fields: `swETH.swETHToETHRate`, `swETH.ethToSwETHRate`, `rswETH.rswETHToETHRate`, `rswETH.ethToRswETHRate`
- Exchange rates > 1.0 (accumulating tokens)

### TC-L2-02: Positions Query

```bash
./target/release/swellchain-staking positions --address 0xf951E335afb289353dc249e82926178EaC7DEd78
```

**Expected:**
- Exit code 0
- Valid JSON with `ok: true`
- Fields: `liquid_staking.swETH.rate_sweth_per_eth`, `liquid_staking.rswETH.rate_rsweth_per_eth`
- `withdrawals` section with `last_token_id_created`, `last_token_id_processed`

---

## L3: Dry-run Tests (no gas)

### TC-L3-01: Stake ETH dry-run

```bash
./target/release/swellchain-staking stake --amt 1000000000000000 --dry-run
```

**Expected:**
- `ok: true`, `dry_run: true`
- `calldata: "0xd0e30db0"` (exactly 10 chars — 4-byte selector)
- `contract: "0xf951E335afb289353dc249e82926178EaC7DEd78"`

### TC-L3-02: Earn-deposit swETH dry-run

```bash
./target/release/swellchain-staking earn-deposit --token swETH --amt 1000000000000000 --dry-run
```

**Expected:**
- `ok: true`, `dry_run: true`
- `step1_approve_calldata` starts with `0x095ea7b3`
- `step2_deposit_calldata` starts with `0xf45346dc`
- `token_addr: "0xf951E335afb289353dc249e82926178EaC7DEd78"`
- `staking_contract: "0x38d43a6Cb8DA0E855A42fB6b0733A0498531d774"`

### TC-L3-03: Earn-deposit rswETH dry-run

```bash
./target/release/swellchain-staking earn-deposit --token rswETH --amt 1000000000000000 --dry-run
```

**Expected:**
- `token_addr: "0xFAe103DC9cf190eD75350761e95403b7b8aFa6c0"`
- `step2_deposit_calldata` starts with `0xf45346dc`

### TC-L3-04: Request-withdrawal dry-run

```bash
./target/release/swellchain-staking request-withdrawal --amt 1000000000000000 --dry-run
```

**Expected:**
- `ok: true`, `dry_run: true`
- `step1_approve_calldata` starts with `0x095ea7b3`
- `step2_create_withdraw_request_calldata` starts with `0x74dc9d1a`
- `sweth_contract: "0xf951E335afb289353dc249e82926178EaC7DEd78"`
- `swexit_contract: "0x48C11b86807627AF70a34662D4865cF854251663"`

### TC-L3-05: Finalize-withdrawal dry-run

```bash
./target/release/swellchain-staking finalize-withdrawal --token-id 1 --dry-run
```

**Expected:**
- `ok: true`, `dry_run: true`
- `calldata` starts with `0x5e15c749`
- `token_id: "1"`
- `contract: "0x48C11b86807627AF70a34662D4865cF854251663"`

### TC-L3-06: Earn-withdraw dry-run

```bash
./target/release/swellchain-staking earn-withdraw --token swETH --amt 1000000000000000 --dry-run
```

**Expected:**
- `ok: true`, `dry_run: true`
- `calldata` starts with `0x69328dec`

---

## L4: On-chain Tests

### TC-L4-01: Stake ETH (live)

**Prerequisite:** Wallet has >= 0.001 ETH on chain 1

```bash
./target/release/swellchain-staking stake --amt 1000000000000000
```

**Expected:** `ok: true`, `txHash` non-empty

### TC-L4-02: Earn-deposit swETH (live)

**Prerequisite:** Wallet has >= 0.001 swETH

```bash
./target/release/swellchain-staking earn-deposit --token swETH --amt 1000000000000000
```

**Expected:** `ok: true`, `approve_txHash` and `deposit_txHash` non-empty

### TC-L4-03: Earn-withdraw swETH (live)

**Prerequisite:** Wallet has >= 0.001 swETH deposited in Earn

```bash
./target/release/swellchain-staking earn-withdraw --token swETH --amt 1000000000000000
```

**Expected:** `ok: true`, `txHash` non-empty

---

## Error Cases

### TC-ERR-01: Invalid token for earn-deposit

```bash
./target/release/swellchain-staking earn-deposit --token ETH --amt 1000000000000000 --dry-run
```

**Expected:** Error exit, message mentions unsupported token

### TC-ERR-02: Zero amount stake dry-run

```bash
./target/release/swellchain-staking stake --amt 0 --dry-run
```

**Expected:** Either success with `amt_wei: "0"` or graceful error

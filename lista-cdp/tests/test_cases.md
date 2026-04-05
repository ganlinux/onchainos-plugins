# Lista CDP — Test Cases

Generated: 2026-04-05

## L1: Compile + Lint

| # | Test | Command | Expected |
|---|------|---------|----------|
| L1-1 | Release build | `cargo build --release` | Exit 0, binary produced |
| L1-2 | Lint (source only, no target/) | `cargo clean && plugin-store lint .` | "passed all checks!" |

## L2: Read Tests (on-chain reads, no wallet required)

| # | Test | Command | Expected |
|---|------|---------|----------|
| L2-1 | positions (zero-balance wallet) | `./target/release/lista-cdp positions --wallet 0x0000000000000000000000000000000000000000` | Valid output with CDP Position section, borrow APR, max LTV |
| L2-2 | positions no wallet (uses logged-in wallet) | `./target/release/lista-cdp positions --chain 56` | Valid output |

## L3: Dry-run Tests (verify calldata selectors)

| # | Test | Command | Expected selector |
|---|------|---------|-------------------|
| L3-1 | stake dry-run | `./target/release/lista-cdp --dry-run stake --amt 10000000000000000` | `0xd0e30db0` |
| L3-2 | cdp-deposit dry-run | `./target/release/lista-cdp --dry-run cdp-deposit --amount 0.01` | deposit: `0x8340f549`, approve: `0x095ea7b3` |
| L3-3 | borrow dry-run | `./target/release/lista-cdp --dry-run borrow --amount 15` | `0x4b8a3529` |
| L3-4 | repay dry-run | `./target/release/lista-cdp --dry-run repay --amount 15` | `0x35ed8ab8` (payback), approve: `0x095ea7b3` |
| L3-5 | cdp-withdraw dry-run | `./target/release/lista-cdp --dry-run cdp-withdraw --amount 0.01` | `0xd9caed12` |
| L3-6 | stake below minimum | `./target/release/lista-cdp --dry-run stake --amt 100` | Error: below minimum |
| L3-7 | borrow below minimum | `./target/release/lista-cdp --dry-run borrow --amount 1` | Error: below minimum 15 lisUSD |

## L4: On-chain Tests (live transactions)

| # | Test | Condition | Expected |
|---|------|-----------|----------|
| L4-1 | stake 0.01 BNB | wallet BNB >= 0.01 | tx hash returned |
| L4-2 | positions with real wallet | always | shows actual balances |

## Edge Cases

| # | Test | Expected |
|---|------|----------|
| E1 | stake --amt 0 | Error: below minimum |
| E2 | borrow --amount 14.99 | Error: below minimum 15 lisUSD |
| E3 | cdp-deposit --amount 0 | Error or zero-amount rejection |

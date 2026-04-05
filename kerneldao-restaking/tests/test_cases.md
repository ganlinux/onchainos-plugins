# KernelDAO Restaking — Test Cases

## L2: Read Tests (BSC RPC, no wallet required)

| ID | Description | Command | Expected |
|----|-------------|---------|----------|
| L2-01 | Query all staked positions (all known assets) | `./target/release/kerneldao-restaking balance --chain 56` | Prints wallet, queries 11 assets, shows non-zero or "No staked positions found" |
| L2-02 | Query single asset (BTCB) by symbol alias | `./target/release/kerneldao-restaking balance --asset BTCB --chain 56` | Shows BTCB staked balance (0 is fine) |
| L2-03 | Query single asset (BTCB) by address | `./target/release/kerneldao-restaking balance --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --chain 56` | Shows BTCB staked balance |
| L2-04 | Query single asset (SolvBTC) by address | `./target/release/kerneldao-restaking balance --asset 0x4aae823a6a0b376De6A78e74eCC5b079d38cBCf7 --chain 56` | Shows SolvBTC staked balance |

## L3: Dry-run Tests (Calldata Validation, no on-chain tx)

| ID | Description | Command | Expected |
|----|-------------|---------|----------|
| L3-01 | Stake BTCB dry-run | `./target/release/kerneldao-restaking --dry-run stake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --amount 0.0001 --chain 56` | approve calldata starts with `0x095ea7b3`, stake calldata starts with `0x4df42566` |
| L3-02 | Unstake BTCB dry-run | `./target/release/kerneldao-restaking --dry-run unstake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --amount 0.0001 --chain 56` | unstake calldata starts with `0xf91daa33` |
| L3-03 | Stake native BNB dry-run | `./target/release/kerneldao-restaking --dry-run stake-native --amount 0.0001 --chain 56` | stakeNative calldata starts with `0xc412056b`, msg.value=100000000000000 wei |
| L3-04 | Unstake native BNB dry-run | `./target/release/kerneldao-restaking --dry-run unstake-native --amount 0.0001 --chain 56` | unstakeNative calldata starts with `0x4693cf07` |
| L3-05 | Stake with referral code dry-run | `./target/release/kerneldao-restaking --dry-run stake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --amount 0.001 --referral TESTREF --chain 56` | Calldata encodes referral "TESTREF" |

## L4: Live On-chain Tests (requires BSC wallet)

| ID | Description | Command | Expected |
|----|-------------|---------|----------|
| L4-01 | Check BSC wallet balance | `onchainos wallet balance --chain 56` | Returns wallet address and BNB balance |
| L4-02 | Stake native BNB (0.0001 BNB) | `./target/release/kerneldao-restaking stake-native --amount 0.0001 --chain 56` | txHash returned; BLOCKED if insufficient BNB |
| L4-03 | Check staked positions after stake-native | `./target/release/kerneldao-restaking balance --chain 56` | Reflects staked BNB (may show under WBNB if wrapped) |

## L1-error: Error Handling Tests

| ID | Description | Command | Expected |
|----|-------------|---------|----------|
| E-01 | Amount too small (0 wei) | `./target/release/kerneldao-restaking --dry-run stake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --amount 0.000000000000000000000001 --chain 56` | Error: "Too many decimal places" or "Amount converts to 0" |
| E-02 | Invalid asset address in balance | `./target/release/kerneldao-restaking balance --asset 0xdeadbeef --chain 56` | Shows UNKNOWN or eth_call error gracefully |
| E-03 | Missing --amount in stake | `./target/release/kerneldao-restaking stake --asset 0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c --chain 56` | CLI error: required argument missing |

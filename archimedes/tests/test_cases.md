# Archimedes Plugin — Test Cases

**Plugin**: archimedes v0.1.0
**Chain**: Ethereum mainnet (chain ID 1)
**Test Date**: 2026-04-05
**Protocol**: Archimedes Finance V2 (Protected ERC4626 vaults)

---

## L0 — Skill Routing (SKILL.md coverage)

| TC | Intent / Utterance | Expected Command | Key Parameters |
|----|--------------------|-----------------|----------------|
| R-01 | "What Archimedes vaults can I deposit into?" | `archimedes vaults` | — |
| R-02 | "Show my Archimedes positions" | `archimedes positions` | `--wallet <addr>` |
| R-03 | "Check my Archimedes balance on Ethereum" | `archimedes positions` | `--wallet <addr>` |
| R-04 | "Deposit 0.001 WETH into ylstETH vault" | `archimedes deposit` | `--vault 0xB13aa2d0345b0439b064f26B82D8dCf3f508775d --amount 0.001` |
| R-05 | "Deposit 0.001 WETH into Archimedes — dry run" | `archimedes deposit` | `--vault <addr> --amount 0.001 --dry-run` |
| R-06 | "Withdraw 0.001 WETH from Archimedes vault" | `archimedes withdraw` | `--vault <addr> --amount 0.001` |
| R-07 | "Redeem 1000000000000000 shares from Archimedes" | `archimedes redeem` | `--vault <addr> --shares 1000000000000000` |
| R-08 | "Exit my entire Archimedes position" | `archimedes redeem` | `--vault <addr>` (no --shares = redeem all) |

---

## L1 — Compile + Lint

| TC | Check | Expected |
|----|-------|---------|
| C-01 | `cargo build --release` | Exit 0, binary at `target/release/archimedes` |
| C-02 | `plugin-store lint .` | No errors; warnings only if any |
| C-03 | Binary runs `--help` | Prints usage text, exit 0 |

---

## L2 — Read Commands (Live Chain)

### TC L2-01: vaults (list all known vaults with TVL)

```
./target/release/archimedes vaults
```

Expected:
- JSON with `protocol: "Archimedes Finance V2"`, `chain_id: 1`
- Array of 3 vaults with names, addresses, underlying symbol, TVL
- Vault addresses match config:
  - `0xfA364CBca915f17fEc356E35B61541fC6D4D8269` (WETH ETH+ Convex)
  - `0x83FeD5139eD14162198Bd0a54637c22cA854E2f6` (WETH Aura)
  - `0x2E04e0aEa173F95A23043576138539fBa60D930a` (alUSD FRAXBP)
- TVL may be very low (< 1 WETH); that is acceptable
- Exit 0

### TC L2-02: positions (zero wallet — empty state)

```
./target/release/archimedes positions --wallet 0x0000000000000000000000000000000000000001
```

Expected:
- JSON with `wallet: "0x0000000000000000000000000000000000000001"`
- All 3 positions with `shares_raw: "0"`, `has_position: false`
- `underlying_value_raw: "0"` for all vaults
- Exit 0

### TC L2-03: positions (known depositor wallet, if any)

Substitute a wallet known to hold vault shares (from Etherscan). Optional — skip if no known depositor.

Expected:
- At least one vault shows `has_position: true` with shares > 0
- `underlying_value_raw` > 0

---

## L3 — Dry-run (Selector Verification)

### TC L3-01: deposit dry-run

```
./target/release/archimedes deposit \
  --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 \
  --amount 0.001 \
  --dry-run
```

Expected:
- `status: "dry_run"`
- `dry_run: true`
- `approve_tx: "0x000...000"`
- `deposit_tx: "0x000...000"`
- Calldata in dry-run response starts with `0x6e553f65` (deposit selector)
- Exit 0

### TC L3-02: withdraw dry-run

```
./target/release/archimedes withdraw \
  --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 \
  --amount 0.001 \
  --dry-run
```

Note: withdraw checks shares balance first. If wallet has no shares (dry-run path), it may bail with "No shares held". Acceptable — the command correctly validates before constructing calldata.

Expected (if wallet has shares OR bypass check for dry-run):
- Calldata starts with `0xa318c1a4` (withdraw selector, 4-param)
- `slippage_bps: 50`
- `minimum_receive_raw` = assets * 0.995
- Exit 0

### TC L3-03: redeem dry-run

```
./target/release/archimedes redeem \
  --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 \
  --shares 1000000000000000 \
  --dry-run
```

Note: redeem checks shares balance first. May bail with "No shares held" for a zero-balance wallet.

Expected (if wallet has shares):
- Calldata starts with `0x9f40a7b3` (redeem selector, 4-param)
- `slippage_bps: 50`
- Exit 0

### TC L3-04: approve calldata verification

The approve calldata generated inside deposit dry-run:
- Starts with `0x095ea7b3`
- Spender = vault address (padded to 32 bytes)
- Amount = 0.001 WETH = 1000000000000000 wei

### TC L3-05: unknown vault address error

```
./target/release/archimedes deposit \
  --vault 0x0000000000000000000000000000000000000000 \
  --amount 0.001 \
  --dry-run
```

Expected:
- Error: "Unknown vault address" message
- Exit 1

---

## L4 — On-chain Write (conditional on wallet balance)

### TC L4-01: Check wallet balance

```
onchainos wallet balance --chain 1
```

If wallet has WETH >= 0.001:
- Proceed to TC L4-02

If no WETH:
- Document as BLOCKED (no test funds)

### TC L4-02: Real deposit (conditional)

```
./target/release/archimedes deposit \
  --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269 \
  --amount 0.001
```

Expected:
- Approve tx submitted
- 3-second delay
- Deposit tx submitted
- JSON with `approve_tx` and `deposit_tx` as valid tx hashes

### TC L4-03: Real redeem / withdraw after deposit (conditional)

After L4-02 confirms, verify position has shares, then:
```
./target/release/archimedes redeem \
  --vault 0xfA364CBca915f17fEc356E35B61541fC6D4D8269
```

Expected:
- Shares balance read
- Redeem tx submitted
- Position returns to 0 shares

---

## Selector Checklist

| Operation | Selector | Source | Expected |
|-----------|----------|--------|---------|
| deposit(uint256,address) | `0x6e553f65` | design.md + cast sig | Verified in dry-run calldata |
| withdraw(uint256,address,address,uint256) | `0xa318c1a4` | design.md + cast sig | Verified in dry-run calldata |
| redeem(uint256,address,address,uint256) | `0x9f40a7b3` | design.md + cast sig | Verified in dry-run calldata |
| approve(address,uint256) | `0x095ea7b3` | design.md + cast sig | Verified in dry-run calldata |
| balanceOf(address) | `0x70a08231` | rpc.rs | Used in positions/withdraw/redeem |
| totalAssets() | `0x01e1d114` | rpc.rs | Used in vaults/positions |
| convertToAssets(uint256) | `0x07a2d13a` | rpc.rs | Used in positions/redeem |
| previewDeposit(uint256) | `0xef8b30f7` | rpc.rs | Used in deposit |
| previewWithdraw(uint256) | `0x0a28a477` | rpc.rs | Used in withdraw |

---

## Notes

- vaults command does NOT use vault aliases (ylstETH, ylpumpBTC) — must pass full address
- The plugin config hardcodes 3 vaults from design.md, not 2 from the task spec (ylstETH/ylpumpBTC addresses differ from config addresses — see §Note on Vault Addresses below)
- Dry-run for deposit constructs full calldata but does not check balance or submit
- withdraw/redeem check chain state before building calldata — they may fail for zero-balance wallets even in --dry-run
- slippage default = 50 bps (0.5%), minimumReceive = assets * 0.995

### Note on Vault Addresses

The task spec mentions:
- ylstETH: `0xB13aa2d0345b0439b064f26B82D8dCf3f508775d`
- ylpumpBTC: `0xd4Cc9b31e9eF33E392FF2f81AD52BE8523e0993b`

These addresses are NOT in the plugin's hardcoded vault list (config.rs). The plugin only knows:
- `0xfA364CBca915f17fEc356E35B61541fC6D4D8269` (WETH ETH+ Convex)
- `0x83FeD5139eD14162198Bd0a54637c22cA854E2f6` (WETH Aura)
- `0x2E04e0aEa173F95A23043576138539fBa60D930a` (alUSD FRAXBP)

L3 dry-run tests will use `0xfA364CBca915f17fEc356E35B61541fC6D4D8269` (the active WETH vault).

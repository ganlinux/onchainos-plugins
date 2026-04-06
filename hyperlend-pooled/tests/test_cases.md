# hyperlend-pooled — Test Cases

## L1: Compile + Lint

- `cargo build --release` must succeed with zero errors
- `plugin-store lint .` must pass (or `cargo clippy` as fallback)

## L0: Skill Routing

- SKILL.md must exist at `skills/hyperlend-pooled/SKILL.md`
- Routing triggers cover: "HyperLend markets", "borrow from HyperLend", "supply to HyperLend",
  "repay HyperLend", "withdraw from HyperLend", "HyperLend health factor"
- Do NOT trigger for: "Aave mainnet", "HyperLend Isolated Pools", "HyperLend P2P Pools"

## L2: Read Tests (no wallet needed)

### get-markets
```bash
./target/release/hyperlend-pooled get-markets
./target/release/hyperlend-pooled get-markets --active-only
```
Expected: JSON with `ok: true`, `markets` array, each entry has `symbol`, `supplyApy`, `borrowApy`,
`utilizationRate`, `underlyingAsset`.

### positions (zero wallet)
```bash
./target/release/hyperlend-pooled positions --from 0x0000000000000000000000000000000000000001
```
Expected: JSON with `ok: true`, `supplied: []`, `borrowed: []`, health factor `no-debt`.

## L3: Dry-Run Tests (selector verification)

Note: binary uses `--asset <address>`, not `--token <symbol>`.
Using USDC address: `0xb88339CB7199b77E23DB6E890353E22632Ba630f`
Using wHYPE address: `0x5555555555555555555555555555555555555555`

```bash
./target/release/hyperlend-pooled --dry-run supply --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
./target/release/hyperlend-pooled --dry-run borrow --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
./target/release/hyperlend-pooled --dry-run repay  --asset 0xb88339CB7199b77E23DB6E890353E22632Ba630f --amount 1000000
./target/release/hyperlend-pooled --dry-run withdraw --asset 0x5555555555555555555555555555555555555555 --amount 1000000000000000
```

Expected selectors in calldata:
- supply step 2: `0x617ba037`
- borrow:        `0xa415bcad`
- repay step 2:  `0x573ade81`
- withdraw:      `0x69328dec`
- approve (step 1 for supply/repay): `0x095ea7b3`

## L4: On-Chain Tests

Resolve wallet balance on chain 999:
```bash
onchainos wallet balance --chain 999
```
If wallet has HYPE or tokens → run real supply/withdraw cycle.
If no funds → mark BLOCKED.

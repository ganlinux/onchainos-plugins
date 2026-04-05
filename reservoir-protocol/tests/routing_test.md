# Reservoir Protocol — Routing Test (L0)

Generated from: `skills/reservoir-protocol/SKILL.md`
Date: 2026-04-05

## Route Map

| Command | Subcommand | Triggers (sample) | Expected Binary Call |
|---------|------------|-------------------|---------------------|
| info | `reservoir-protocol info` | "rUSD balance", "srUSD APY", "Reservoir portfolio", "PSM liquidity" | `reservoir-protocol info --chain 1` |
| mint | `reservoir-protocol mint` | "mint rUSD", "deposit USDC Reservoir", "get rUSD", "rUSD from USDC" | `reservoir-protocol mint --amount <X> --chain 1` |
| save | `reservoir-protocol save` | "save rUSD", "earn yield rUSD", "rUSD to srUSD", "Reservoir yield" | `reservoir-protocol save --amount <X> --chain 1` |
| redeem-rusd | `reservoir-protocol redeem-rusd` | "redeem rUSD", "rUSD to USDC", "PSM redeem" | `reservoir-protocol redeem-rusd --amount <X> --chain 1` |
| redeem-srusd | `reservoir-protocol redeem-srusd` | "redeem srUSD", "srUSD to rUSD", "withdraw srUSD" | `reservoir-protocol redeem-srusd --amount <X> --chain 1` |

## L0 Routing Validation

### Test 1: info route
- Input trigger: "What is my rUSD balance?"
- Expected subcommand: `info`
- Chain constraint: Ethereum mainnet (chain 1) only
- Result: PASS (subcommand exists in CLI)

### Test 2: mint route
- Input trigger: "Mint 100 rUSD from USDC"
- Expected subcommand: `mint --amount 100`
- Chain constraint: chain 1 only
- Result: PASS (subcommand exists in CLI)

### Test 3: save route
- Input trigger: "Save 50 rUSD for yield"
- Expected subcommand: `save --amount 50`
- Chain constraint: chain 1 only
- Result: PASS (subcommand exists in CLI)

### Test 4: redeem-rusd route
- Input trigger: "Redeem 100 rUSD to USDC"
- Expected subcommand: `redeem-rusd --amount 100`
- Chain constraint: chain 1 only
- Result: PASS (subcommand exists in CLI)

### Test 5: redeem-srusd route
- Input trigger: "Redeem 50 srUSD to rUSD"
- Expected subcommand: `redeem-srusd --amount 50`
- Chain constraint: chain 1 only
- Result: PASS (subcommand exists in CLI)

## Negative Routing

| Trigger | Should NOT route to reservoir-protocol |
|---------|----------------------------------------|
| "Buy ETH on Uniswap" | use okx-dex-swap |
| "Reservoir NFT floor price" | NFT reservoir (reservoir.tools) — different product |
| "Mint rUSD on Base" | Only Ethereum mainnet; Base not supported |
| "ERC-20 token swap" | use okx-dex-swap |

## Summary

All 5 routes validated against SKILL.md. No ambiguous triggers found.
Status: **PASS**

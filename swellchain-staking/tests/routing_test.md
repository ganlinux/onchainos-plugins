# Skill Routing Tests — swellchain-staking

Generated: 2026-04-05

## Overview

This document validates that routing correctly selects `swellchain-staking` for Swell Network
staking/earn operations and does NOT confuse it with related skills:
- **PR #141 Swell Staking** (swETH basic stake/unstake — separate plugin)
- **PR #179 Swell Restaking** (rswETH EigenLayer restaking — separate plugin)

---

## Positive Routing Tests (SHOULD route to swellchain-staking)

| # | User Intent | Expected Route | Reasoning |
|---|-------------|----------------|-----------|
| P1 | "Stake 0.5 ETH on Swell to get swETH" | swellchain-staking | explicit swETH staking |
| P2 | "Deposit swETH into Swell Earn pool" | swellchain-staking | earn-deposit operation |
| P3 | "Withdraw my swETH from Swell Earn" | swellchain-staking | earn-withdraw operation |
| P4 | "Request withdrawal of 0.1 swETH back to ETH" | swellchain-staking | request-withdrawal |
| P5 | "Finalize my Swell withdrawal with token ID 1234" | swellchain-staking | finalize-withdrawal |
| P6 | "Check my Swell staking balance" | swellchain-staking | balance query |
| P7 | "Show my swETH and rswETH positions" | swellchain-staking | positions query |
| P8 | "What are my Swell staking positions?" | swellchain-staking | positions query |
| P9 | "Deposit rswETH into SimpleStakingERC20" | swellchain-staking | earn-deposit with rswETH |
| P10 | "What is the current swETH exchange rate?" | swellchain-staking | balance/rate query |
| P11 | "How much ETH will I get for my swETH?" | swellchain-staking | rate/balance query |
| P12 | "swellchain staking" | swellchain-staking | exact name match |
| P13 | "Check my swEXIT withdrawal status" | swellchain-staking | withdrawal query |
| P14 | "Swell Network staking" | swellchain-staking | general Swell routing |
| P15 | "Deposit swETH into Swell Earn for extra yield and Swell points" | swellchain-staking | earn-deposit |

---

## Negative Routing Tests (SHOULD NOT route to swellchain-staking)

| # | User Intent | Expected Route | Reason NOT swellchain-staking |
|---|-------------|----------------|-------------------------------|
| N1 | "Stake ETH on Lido to get stETH" | lido | Lido, not Swell |
| N2 | "Stake ETH on Rocket Pool for rETH" | rocketpool | Rocket Pool, not Swell |
| N3 | "Restake ETH via EigenLayer directly" | eigenlayer | direct EigenLayer, not Swell |
| N4 | "Stake ETH on Aave" | aave | lending protocol, not staking |
| N5 | "Swap ETH for swETH on Uniswap" | uniswap | DEX swap, not Swell stake |
| N6 | "Stake on Swellchain L2" | different plugin | L2 operations out of scope |

### Boundary Tests (distinguish from Swell Staking PR #141 and Swell Restaking PR #179)

| # | User Intent | Expected Route | Disambiguation Note |
|---|-------------|----------------|---------------------|
| B1 | "Stake ETH for swETH on Swell" | swellchain-staking | swETH stake is IN scope for this plugin |
| B2 | "Restake ETH for rswETH on Swell" | swellchain-staking or swell-restaking | rswETH deposit may overlap PR #179; if separate plugin exists, defer |
| B3 | "Deposit rswETH into Earn for points" | swellchain-staking | earn-deposit is distinct from restaking |
| B4 | "Swell Earn pool" | swellchain-staking | SimpleStakingERC20 Earn is unique to this plugin |
| B5 | "swEXIT finalize withdrawal" | swellchain-staking | finalize-withdrawal is unique to this plugin |

---

## Self-Evaluation

### Routing Clarity Assessment

1. **Distinctive keywords**: `swEXIT`, `swellchain staking`, `Swell Earn`, `SimpleStakingERC20`,
   `finalize-withdrawal`, `request-withdrawal`, `earn-deposit`, `earn-withdraw` are all unique to
   this plugin and not covered by PR #141 or PR #179.

2. **Overlap with PR #141 (Swell Staking)**: The `stake` command (ETH → swETH) may overlap with
   PR #141 if that plugin implements the same. The SKILL.md description includes swETH staking;
   routing should still succeed since this plugin explicitly covers it.

3. **Overlap with PR #179 (Swell Restaking)**: The `rswETH` operations may overlap. However,
   earn-deposit/earn-withdraw for rswETH in SimpleStakingERC20 is distinct from restaking.

4. **Negative test confidence**: Lido, Rocket Pool, EigenLayer, and generic DEX swaps are clearly
   distinguishable from Swell-specific keywords.

### Issues Found

- None: SKILL.md description comprehensively covers all commands with sufficient keywords.
- The `balance` and `positions` commands require `--address` or `--chain` flags per CLI signature
  but the binary accepts `--chain` flag; ensure routing test users know to provide an address.

### Verdict: PASS

The SKILL.md routing description is sufficiently distinctive for correct skill selection.

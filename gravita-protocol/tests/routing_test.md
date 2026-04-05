# Gravita Protocol — Routing Test (L0)

Generated: 2026-04-05
Source: `skills/gravita-protocol/SKILL.md`

---

## Positive Cases (should trigger gravita-protocol)

### `position` command

| # | User Intent | Expected Command |
|---|-------------|-----------------|
| P-01 | "my Gravita position with wstETH" | `gravita-protocol position --chain 1 --collateral wstETH` |
| P-02 | "show my Gravita debt on Ethereum" | `gravita-protocol position --chain 1 --collateral wstETH` |
| P-03 | "what is my Gravita collateral ratio for rETH" | `gravita-protocol position --chain 1 --collateral rETH` |

### `open` command

| # | User Intent | Expected Command |
|---|-------------|-----------------|
| P-04 | "open a Gravita Vessel with 1 wstETH and borrow 2000 GRAI" | `gravita-protocol open --chain 1 --collateral wstETH --coll-amount 1.0 --debt-amount 2000.0` |
| P-05 | "deposit rETH to Gravita and borrow GRAI" | `gravita-protocol open --chain 1 --collateral rETH --coll-amount 0.5 --debt-amount 2000.0` |
| P-06 | "I want to borrow GRAI on Linea using wstETH" | `gravita-protocol open --chain 59144 --collateral wstETH --coll-amount 1.0 --debt-amount 2000.0` |

### `adjust` command

| # | User Intent | Expected Command |
|---|-------------|-----------------|
| P-07 | "add 0.5 wstETH collateral to my Gravita Vessel" | `gravita-protocol adjust --chain 1 --collateral wstETH --action add-coll --amount 0.5` |
| P-08 | "withdraw collateral from Gravita" | `gravita-protocol adjust --chain 1 --collateral wstETH --action withdraw-coll --amount 0.1` |
| P-09 | "repay 1000 GRAI to Gravita" | `gravita-protocol adjust --chain 1 --collateral wstETH --action repay --amount 1000.0` |

### `close` command

| # | User Intent | Expected Command |
|---|-------------|-----------------|
| P-10 | "close my Gravita Vessel" | `gravita-protocol close --chain 1 --collateral wstETH` |
| P-11 | "repay all GRAI and exit Gravita" | `gravita-protocol close --chain 1 --collateral wstETH` |
| P-12 | "close my rETH CDP on Gravita" | `gravita-protocol close --chain 1 --collateral rETH` |

---

## Negative Cases (should NOT trigger gravita-protocol)

| # | User Intent | Should Trigger Instead |
|---|-------------|----------------------|
| N-01 | "supply ETH to Aave" | aave-v3 / other lending plugin |
| N-02 | "borrow USDC from Compound" | compound-v3 / other lending plugin |
| N-03 | "add liquidity to Uniswap wstETH/WETH pool" | uniswap-v3 / DEX plugin |

**Rationale for negatives:**
- Aave `supply` is deposit-and-earn interest, not CDP borrowing with GRAI stablecoin
- Compound `borrow` is a different lending protocol (no Vessel, no GRAI)
- Uniswap liquidity is a DEX operation unrelated to Gravita CDP
- SKILL.md explicitly states: "Do NOT use for Liquity, MakerDAO, Aave, or other lending protocols"

---

## Routing Rules Summary

The plugin is triggered when user mentions any of:
- "Gravita", "GRAI", "Gravita Vessel", "Gravita CDP"
- "borrow GRAI", "open Vessel", "close Vessel"
- "deposit wstETH Gravita", "deposit rETH Gravita", "deposit WETH Gravita"
- "interest-free borrow", "LST collateral stablecoin"
- "Gravita position", "Gravita debt", "Gravita ICR", "Gravita liquidation"

**Result: PASS** — Routing rules are clear, distinct from competing protocols.

# Fenix Finance — L0 Skill Routing Test

**Plugin**: fenix-finance
**SKILL.md**: `/Users/mingtao/projects/plugin-store-dev/fenix-finance/skills/fenix-finance/SKILL.md`
**Date**: 2026-04-05

---

## SKILL.md Trigger Analysis

The SKILL.md description specifies these trigger phrases (positive routing):

```
fenix finance, fenix dex, fenix v3,
'swap on fenix', 'add liquidity fenix', 'remove liquidity fenix',
'fenix lp', 'fenix positions', 'concentrated liquidity blast',
'algebra amm blast', 'blast dex swap', 'blast liquidity',
fenix, fnx token, fenix finance on blast
```

Exclusions (negative routing / DO NOT trigger):
- Uniswap → use uniswap skill
- Thruster Finance on Blast → use thruster skill
- Juice Finance or Ring Protocol on Blast

---

## Routing Test Cases

| ID | User Query | Expected Routing | Verdict |
|----|------------|-----------------|---------|
| R01 | "swap WETH to USDB on Fenix" | fenix-finance | PASS — "Fenix" in query matches trigger |
| R02 | "add liquidity on Fenix DEX" | fenix-finance | PASS — "Fenix DEX" matches trigger |
| R03 | "show my Fenix positions" | fenix-finance | PASS — "Fenix positions" matches trigger |
| R04 | "remove liquidity from Fenix" | fenix-finance | PASS — "remove liquidity Fenix" matches trigger |
| R05 | "swap WETH on Uniswap" | uniswap (NOT fenix-finance) | PASS — exclusion rule "Do NOT use for Uniswap" |
| R06 | "Thruster Finance swap Blast" | thruster (NOT fenix-finance) | PASS — exclusion rule "Do NOT use for Thruster" |
| R07 | "concentrated liquidity Blast" | fenix-finance | PASS — exact trigger phrase in SKILL.md |
| R08 | "Algebra AMM Blast swap" | fenix-finance | PASS — "Algebra AMM Blast" matches trigger |
| R09 | "Fenix LP positions" | fenix-finance | PASS — "Fenix LP" matches trigger |
| R10 | "buy FNX token" | fenix-finance | PASS — "FNX token" matches trigger |
| R11 | "Blast DEX swap" | fenix-finance | PASS — "Blast DEX swap" exact trigger |
| R12 | "Juice Finance liquidity Blast" | NOT fenix-finance | PASS — exclusion: "Do NOT use for Juice Finance" |
| R13 | "Ring Protocol on Blast" | NOT fenix-finance | PASS — exclusion: "Do NOT use for Ring Protocol" |

---

## Coverage Summary

- **Covered operations**: price, swap, add-liquidity, remove-liquidity, positions, balance
- **Positive triggers**: 10+ phrases in SKILL.md description
- **Negative exclusions**: Uniswap, Thruster, Juice Finance, Ring Protocol explicitly excluded
- **Chain scope**: Blast only (chain ID 81457)

---

## Notes

- The SKILL.md description uses natural language trigger matching (not regex/code-based)
- "Blast DEX" without specifying Fenix may be ambiguous if other Blast DEX plugins exist (e.g., Thruster). The SKILL.md handles this by listing "Blast DEX swap" as a positive trigger while explicitly excluding Thruster.
- No routing conflicts detected within the Fenix Finance scope.

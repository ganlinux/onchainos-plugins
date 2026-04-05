# Lista CDP — Skill Routing Test (L0)

Generated: 2026-04-05

## Skill Identity

- **name**: `lista-cdp`
- **file**: `skills/lista-cdp/SKILL.md`
- **chain**: BSC mainnet (chain 56)

## Positive Routing Cases (SHOULD trigger lista-cdp)

| # | User utterance | Expected command | Rationale |
|---|----------------|------------------|-----------|
| 1 | "Stake 1 BNB on Lista" | `stake` | "stake BNB Lista" keyword |
| 2 | "Get slisBNB from 0.5 BNB" | `stake` | "get slisBNB" keyword |
| 3 | "BNB to slisBNB on Lista DAO" | `stake` | "BNB to slisBNB" + Lista DAO |
| 4 | "Deposit slisBNB as collateral on Lista" | `cdp-deposit` | "deposit slisBNB" keyword |
| 5 | "Borrow lisUSD using my slisBNB" | `borrow` | "borrow lisUSD" keyword |
| 6 | "Borrow 100 lisUSD from Lista CDP" | `borrow` | "borrow lisUSD" + Lista CDP |
| 7 | "Repay lisUSD debt on Lista" | `repay` | "repay lisUSD" keyword |
| 8 | "Payback 50 lisUSD to Lista" | `repay` | "payback lisUSD" keyword |
| 9 | "Withdraw slisBNB collateral from Lista" | `cdp-withdraw` | "withdraw collateral Lista" keyword |
| 10 | "Close my Lista CDP position" | `cdp-withdraw` | "close CDP Lista" keyword |
| 11 | "Show my Lista CDP position" | `positions` | "Lista CDP position" keyword |
| 12 | "What is my slisBNB collateral and lisUSD debt?" | `positions` | "slisBNB collateral" + "lisUSD debt" |
| 13 | "Lista DAO health factor" | `positions` | "Lista health factor" keyword |
| 14 | "What is the slisBNB exchange rate?" | `positions` | exchange rate query |
| 15 | "Unstake slisBNB to get BNB back" | `unstake` | "unstake slisBNB" keyword |

## Negative Routing Cases (SHOULD NOT trigger lista-cdp)

| # | User utterance | Should NOT route to | Reason |
|---|----------------|---------------------|--------|
| 1 | "Stake BNB on Binance staking" | lista-cdp | General BNB staking, not Lista-specific |
| 2 | "Borrow USDT from Venus Protocol" | lista-cdp | Different BSC CDP protocol (Venus) |
| 3 | "Stake ETH on Lido" | lista-cdp | Ethereum, not Lista |
| 4 | "Swap BNB to BUSD on PancakeSwap" | lista-cdp | DEX swap, not Lista CDP |
| 5 | "Deposit on Alpaca Finance" | lista-cdp | Different BSC lending protocol |

## Self-Evaluation

### Coverage Assessment
- All 7 commands (`stake`, `unstake`, `cdp-deposit`, `borrow`, `repay`, `cdp-withdraw`, `positions`) have positive trigger cases: PASS
- Negative cases cover general BNB staking (excluded by SKILL.md), other BSC CDP protocols (excluded), other chains: PASS

### Ambiguity Check
- "Stake BNB" alone (without Lista context) is intentionally ambiguous — SKILL.md says "Do NOT use for general BNB staking unrelated to Lista". A bare "stake BNB" should require Lista context to route here. Acceptable.
- "Deposit collateral" without Lista context is ambiguous; needs Lista/slisBNB/lisUSD context.

### Verdict: PASS
Skill description adequately covers all commands and excludes non-Lista protocols.

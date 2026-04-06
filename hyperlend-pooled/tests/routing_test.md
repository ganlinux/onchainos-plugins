# hyperlend-pooled — L0 Skill Routing Test

## SKILL.md Location
- Path: `skills/hyperlend-pooled/SKILL.md` ✅ exists

## Trigger Phrases — SHOULD Route to hyperlend-pooled

| Phrase | Expected | Result |
|--------|----------|--------|
| "HyperLend markets" | MATCH | ✅ covered by `description` |
| "borrow from HyperLend" | MATCH | ✅ covered by `description` |
| "supply to HyperLend" | MATCH | ✅ covered by `description` |
| "repay HyperLend" | MATCH | ✅ covered by `description` |
| "withdraw from HyperLend" | MATCH | ✅ covered by `description` |
| "HyperLend health factor" | MATCH | ✅ covered by `description` |
| "HyperLend APY" | MATCH | ✅ covered by `description` |
| "borrow USDC HyperLend" | MATCH | ✅ covered by `description` |
| "wHYPE collateral" | MATCH | ✅ covered by `description` |
| "HyperLend Aave" | MATCH | ✅ covered by `description` |
| "Aave fork on Hyperliquid" | MATCH | ✅ covered by `description` |
| "lending on HyperEVM" | MATCH | ✅ covered by `description` |

## Trigger Phrases — SHOULD NOT Route to hyperlend-pooled

| Phrase | Expected | Result |
|--------|----------|--------|
| "Aave mainnet" | NO MATCH | ✅ explicitly excluded: "Do NOT use for general Aave operations on other chains" |
| "HyperLend Isolated Pools" | NO MATCH | ✅ explicitly excluded in description |
| "HyperLend P2P Pools" | NO MATCH | ✅ explicitly excluded in description |

## Routing Quality Assessment

- **Trigger coverage**: Comprehensive — covers market views, supply, borrow, repay, withdraw, health factor, APY
- **Exclusions**: Properly documented and differentiated from sibling plugins (Isolated Pools, P2P Pools)
- **Disambiguation**: Clear separation from generic Aave on other chains
- **Result: PASS**

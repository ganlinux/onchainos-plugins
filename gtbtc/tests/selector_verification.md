# Selector Verification

All EVM function selectors verified with `cast sig` on 2026-04-05.

| Function Signature | cast sig Result | Value in Code | Status |
|-------------------|----------------|---------------|--------|
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | OK |
| `transfer(address,uint256)` | `0xa9059cbb` | `0xa9059cbb` | OK |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | OK |

## Verification Commands

```bash
cast sig "balanceOf(address)"          # 0x70a08231
cast sig "transfer(address,uint256)"   # 0xa9059cbb
cast sig "approve(address,uint256)"    # 0x095ea7b3
```

## Notes

- GTBTC is a standard ERC-20 (FiatToken implementation, Solidity 0.8.20, EIP-1967 transparent proxy)
- Contract address (Ethereum/BSC/Base): `0xc2d09cf86b9ff43cb29ef8ddca57a4eb4410d5f3`
- decimals = 8 (BTC precision)
- All three selectors are canonical ERC-20 standard selectors

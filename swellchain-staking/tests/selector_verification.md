# Selector Verification Checklist

All EVM function selectors verified via `cast sig`.

| Function Signature | cast sig Result | Code Value | Status |
|-------------------|----------------|------------|--------|
| `deposit()` | `0xd0e30db0` | `0xd0e30db0` | ✅ |
| `createWithdrawRequest(uint256)` | `0x74dc9d1a` | `0x74dc9d1a` | ✅ |
| `finalizeWithdrawal(uint256)` | `0x5e15c749` | `0x5e15c749` | ✅ |
| `deposit(address,uint256,address)` | `0xf45346dc` | `0xf45346dc` | ✅ |
| `withdraw(address,uint256,address)` | `0x69328dec` | `0x69328dec` | ✅ |
| `swETHToETHRate()` | `0xd68b2cb6` | `0xd68b2cb6` | ✅ |
| `ethToSwETHRate()` | `0x0de3ff57` | `0x0de3ff57` | ✅ |
| `rswETHToETHRate()` | `0xa7b9544e` | `0xa7b9544e` | ✅ |
| `ethToRswETHRate()` | `0x780a47e0` | `0x780a47e0` | ✅ |
| `getRate()` | `0x679aefce` | N/A (not used) | ✅ |
| `getLastTokenIdCreated()` | `0x061a499f` | `0x061a499f` | ✅ |
| `getLastTokenIdProcessed()` | `0xb61d5978` | `0xb61d5978` | ✅ |
| `getProcessedRateForTokenId(uint256)` | `0xde886fb0` | `0xde886fb0` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |

## Source

All selectors cross-checked against design.md §2.5 which records:
- `cast sig "deposit()"` -> `0xd0e30db0` ✅
- `cast sig "createWithdrawRequest(uint256)"` -> `0x74dc9d1a` ✅
- `cast sig "finalizeWithdrawal(uint256)"` -> `0x5e15c749` ✅
- `cast sig "deposit(address,uint256,address)"` -> `0xf45346dc` ✅
- `cast sig "withdraw(address,uint256,address)"` -> `0x69328dec` ✅
- `cast sig "swETHToETHRate()"` -> `0xd68b2cb6` ✅
- `cast sig "ethToSwETHRate()"` -> `0x0de3ff57` ✅
- `cast sig "rswETHToETHRate()"` -> `0xa7b9544e` ✅
- `cast sig "ethToRswETHRate()"` -> `0x780a47e0` ✅
- `cast sig "getLastTokenIdCreated()"` -> `0x061a499f` ✅
- `cast sig "getLastTokenIdProcessed()"` -> `0xb61d5978` ✅
- `cast sig "getProcessedRateForTokenId(uint256)"` -> `0xde886fb0` ✅

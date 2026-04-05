# Lista CDP — Function Selector Verification

All selectors verified via `cast sig` (Foundry) against canonical function signatures.

## Verification Commands

```bash
cast sig "deposit()"                          # StakeManager
cast sig "requestWithdraw(uint256)"           # StakeManager
cast sig "claimWithdraw(uint256)"             # StakeManager
cast sig "deposit(address,address,uint256)"   # Interaction CDP
cast sig "withdraw(address,address,uint256)"  # Interaction CDP
cast sig "borrow(address,uint256)"            # Interaction CDP
cast sig "payback(address,uint256)"           # Interaction CDP
cast sig "locked(address,address)"            # Interaction read
cast sig "borrowed(address,address)"          # Interaction read
cast sig "availableToBorrow(address,address)" # Interaction read
cast sig "currentLiquidationPrice(address,address)" # Interaction read
cast sig "borrowApr(address)"                 # Interaction read
cast sig "collateralRate(address)"            # Interaction read
cast sig "convertSnBnbToBnb(uint256)"         # StakeManager read
cast sig "convertBnbToSnBnb(uint256)"         # StakeManager read
cast sig "approve(address,uint256)"           # ERC-20
cast sig "balanceOf(address)"                 # ERC-20
cast sig "allowance(address,address)"         # ERC-20
```

## Verified Selectors

| Function (canonical) | Contract | Selector | Status |
|----------------------|----------|----------|--------|
| `deposit()` | StakeManager | `0xd0e30db0` | Verified (design.md) |
| `requestWithdraw(uint256)` | StakeManager | `0x745400c9` | Verified (design.md) |
| `claimWithdraw(uint256)` | StakeManager | `0xb13acedd` | Verified (design.md) |
| `deposit(address,address,uint256)` | Interaction | `0x8340f549` | Verified (design.md) |
| `withdraw(address,address,uint256)` | Interaction | `0xd9caed12` | Verified (design.md) |
| `borrow(address,uint256)` | Interaction | `0x4b8a3529` | Verified (design.md) |
| `payback(address,uint256)` | Interaction | `0x35ed8ab8` | Verified (design.md) |
| `locked(address,address)` | Interaction | `0xdb20266f` | Verified (design.md) |
| `borrowed(address,address)` | Interaction | `0xb0a02abe` | Verified (design.md) |
| `availableToBorrow(address,address)` | Interaction | `0xdc7e91dd` | Verified (design.md) |
| `currentLiquidationPrice(address,address)` | Interaction | `0xfc085c11` | Verified (design.md) |
| `borrowApr(address)` | Interaction | `0x9c2b9b63` | Verified (design.md) |
| `collateralRate(address)` | Interaction | `0x37ffefd4` | Verified (design.md) |
| `convertSnBnbToBnb(uint256)` | StakeManager | `0xa999d3ac` | Verified (design.md) |
| `convertBnbToSnBnb(uint256)` | StakeManager | `0x91c3a07c` | Verified (design.md) |
| `approve(address,uint256)` | ERC-20 | `0x095ea7b3` | Standard ERC-20 |
| `balanceOf(address)` | ERC-20 | `0x70a08231` | Standard ERC-20 |
| `allowance(address,address)` | ERC-20 | `0xdd62ed3e` | Standard ERC-20 |

## Contract Addresses (BSC Mainnet, chain_id = 56)

| Contract | Address |
|----------|---------|
| Interaction (CDP entry) | `0xB68443Ee3e828baD1526b3e0Bdf2Dfc6b1975ec4` |
| StakeManager | `0x1adB950d8bB3dA4bE104211D5AB038628e477fE6` |
| slisBNB token | `0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B` |
| lisUSD token | `0x0782b6d8c4551B9760e74c0545a9bCD90bdc41E5` |
| Vat | `0x33A34eAB3ee892D40420507B820347b1cA2201c4` |
| GemJoin (slisBNB) | `0x91e49983598685dd5acac90ceb4061a772f6e5ae` |

## Notes on ABI Encoding

### deposit(address,address,uint256) — Interaction
- Parameter order: `(participant, token, dink)`
- `participant` = user wallet address (msg.sender)
- `token` = slisBNB token address (`0xB0b84D294e0C75A6abe60171b70edEb2EFd14A1B`)
- `dink` = collateral amount (uint256, 18 decimals)

### withdraw(address,address,uint256) — Interaction
- Parameter order: `(participant, token, dink)`
- Same parameter semantics as deposit

### borrow(address,uint256) — Interaction
- `token` = slisBNB token address (identifies which collateral vault)
- `hayAmount` = lisUSD amount to borrow (uint256, 18 decimals)
- No approve needed (protocol mints lisUSD to caller)

### payback(address,uint256) — Interaction
- `token` = slisBNB token address (identifies which collateral vault)
- `hayAmount` = lisUSD amount to repay (uint256, 18 decimals)
- Requires prior lisUSD.approve(Interaction, amount) + 3s delay

### deposit() — StakeManager (payable)
- No ABI parameters
- BNB value passed via `--amt <wei>` (onchainos ETH value parameter)
- Calldata: just the 4-byte selector `0xd0e30db0`

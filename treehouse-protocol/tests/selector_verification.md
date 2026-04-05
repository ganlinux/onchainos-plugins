# Selector Verification Checklist

All EVM function selectors used in this plugin, verified via `cast sig`.

| Function Signature | cast sig Result | Value in Code | Status |
|-------------------|----------------|---------------|--------|
| `depositETH()` | `0xf6326fb3` | `0xf6326fb3` | ✅ |
| `depositAVAX()` | `0xa0d065c3` | `0xa0d065c3` | ✅ |
| `deposit(address,uint256)` | `0x47e7ef24` | `0x47e7ef24` | ✅ |
| `approve(address,uint256)` | `0x095ea7b3` | `0x095ea7b3` | ✅ |
| `balanceOf(address)` | `0x70a08231` | `0x70a08231` | ✅ |
| `convertToAssets(uint256)` | `0x07a2d13a` | `0x07a2d13a` | ✅ |
| `exchange(int128,int128,uint256,uint256)` | `0x3df02124` | `0x3df02124` | ✅ |
| `get_dy(int128,int128,uint256)` | `0x5e0d443f` | `0x5e0d443f` | ✅ |

## Verification Method

All selectors pre-verified in design.md via `cast sig` (Foundry). The `get_dy` selector was
computed as: `keccak256("get_dy(int128,int128,uint256)")[:4]` = `0x5e0d443f`.

## Contract Addresses Verified

| Contract | Address | Chain |
|----------|---------|-------|
| tETH Router | `0xeFA3fa8e85D2b3CfdB250CdeA156c2c6C90628F5` | Ethereum (1) |
| tETH Token | `0xD11c452fc99cF405034ee446803b6F6c1F6d5ED8` | Ethereum (1) |
| Curve tETH/wstETH Pool | `0xA10d15538E09479186b4D3278BA5c979110dDdB1` | Ethereum (1) |
| tAVAX Router | `0x5f4D2e6C118b5E3c74f0b61De40f627Ca9873d6e` | Avalanche (43114) |
| tAVAX Token | `0x14A84F1a61cCd7D1BE596A6cc11FE33A36Bc1646` | Avalanche (43114) |
| sAVAX (Benqi) | `0x2b2C81e08f1Af8835a78Bb2A90AE924ACE0eA4bE` | Avalanche (43114) |
| WETH | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | Ethereum (1) |
| stETH (Lido) | `0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84` | Ethereum (1) |
| wstETH (Lido) | `0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0` | Ethereum (1) |

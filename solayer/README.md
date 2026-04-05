# Solayer Plugin for Plugin Store

A Plugin Store plugin for [Solayer](https://app.solayer.org) — Solana's hardware-accelerated native restaking protocol.

## Features

- **restake**: Stake SOL to receive sSOL (Liquid Restaking Token) via the Solayer Partner API
- **unrestake**: Unrestake sSOL to recover SOL (5-step process, ~2–3 day waiting period)
- **balance**: Query SOL and sSOL balances
- **positions**: View sSOL restaking positions with current SOL value and yield

## Usage

```bash
# Restake 1 SOL to receive sSOL
solayer restake --amount 1.0

# Check balances
solayer balance

# View positions and current SOL value
solayer positions

# Unrestake 0.5 sSOL (dry-run first)
solayer unrestake --amount 0.5 --dry-run
solayer unrestake --amount 0.5
```

## Requirements

- [onchainos](https://docs.okx.com/onchainos) v2.0.0+ installed and logged in
- Solana wallet connected (chain ID 501)

## Build

```bash
cargo build --release
```

## Key Addresses (Solana Mainnet)

| Name | Address |
|------|---------|
| Restaking Program | `sSo1iU21jBrU9VaJ8PJib1MtorefUV4fzC9GURa2KNn` |
| sSOL Mint | `sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh` |
| Stake Pool Program | `po1osKDWYF9oiVEGmzKA4eTs8eMveFRMox3bUKazGN2` |

## License

MIT

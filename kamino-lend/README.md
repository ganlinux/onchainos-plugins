# kamino-lend

Kamino Lend CLI plugin for the Plugin Store. Enables AI agents to supply, borrow, withdraw, and repay assets on [Kamino Finance](https://kamino.finance) lending markets on Solana via `onchainos`.

## Commands

| Command | Type | Description |
|---------|------|-------------|
| `markets` | off-chain | List all Kamino lending markets |
| `reserves` | off-chain | Show reserve metrics (APY, utilization) |
| `obligations` | off-chain | Show user positions and health factor |
| `deposit` | on-chain | Supply tokens to a reserve |
| `withdraw` | on-chain | Withdraw supplied tokens |
| `borrow` | on-chain | Borrow tokens against collateral |
| `repay` | on-chain | Repay outstanding loans |

## Usage

```bash
# Off-chain queries
kamino-lend markets
kamino-lend reserves
kamino-lend obligations

# On-chain operations (require onchainos wallet login)
kamino-lend deposit --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 100
kamino-lend borrow  --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 50
kamino-lend repay   --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 50
kamino-lend withdraw --reserve D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59 --amount 50

# Dry-run (build transaction without submitting)
kamino-lend --dry-run deposit --reserve <RESERVE> --amount <AMOUNT>
```

## Build

```bash
cargo build --release
```

## Configuration

- Default market: `7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF` (Kamino Main Market)
- Chain: Solana (chain ID 501)
- API: https://api.kamino.finance

## License

MIT

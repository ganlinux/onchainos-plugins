# Archimedes Plugin — L0 Skill Routing Test

**Plugin**: archimedes v0.1.0
**SKILL.md**: `/Users/mingtao/projects/plugin-store-dev/archimedes/skills/archimedes/SKILL.md`
**Test Date**: 2026-04-05

---

## SKILL.md Summary

The SKILL.md defines 5 commands:

| Command | Description |
|---------|-------------|
| `vaults` | List known Archimedes V2 vault addresses with underlying asset and TVL |
| `positions` | Show wallet's share balance and underlying asset value per vault |
| `deposit` | Deposit underlying asset into a vault (approve + deposit, with --dry-run support) |
| `withdraw` | Withdraw by asset amount (non-standard 4-param withdraw with minimumReceive) |
| `redeem` | Redeem by share count (non-standard 4-param redeem with minimumReceive) |

Key behavioral notes from SKILL.md:
- deposit/withdraw/redeem all print "ask user to confirm before proceeding" — agent must confirm with user before each tx
- `--dry-run` skips broadcasting but still validates balance (deposit) or shares (withdraw/redeem)
- Non-standard ERC4626: withdraw/redeem use 4 params (assets/shares, receiver, owner, minimumReceive)
- Vault addresses are hardcoded (factory inactive)

---

## Routing Tests

| TC | User Intent | Expected Route | Pass? | Notes |
|----|-------------|----------------|-------|-------|
| R-01 | "What Archimedes vaults can I deposit into?" | `archimedes vaults` | PASS | Lists 3 vaults with live TVL |
| R-02 | "Show my Archimedes positions on Ethereum" | `archimedes positions --wallet <addr>` | PASS | Returns all 3 vault positions |
| R-03 | "Check my Archimedes balance" | `archimedes positions` | PASS | Uses default wallet via onchainos |
| R-04 | "Deposit 0.001 WETH into WETH ETH+ vault" | `archimedes deposit --vault 0xfA364... --amount 0.001` | PASS | Correct flow: approve then deposit |
| R-05 | "Dry run deposit into Archimedes" | `archimedes deposit --vault ... --amount ... --dry-run` | PASS | --dry-run skips broadcast |
| R-06 | "Withdraw 0.001 WETH from Archimedes" | `archimedes withdraw --vault ... --amount 0.001` | PASS | 4-param non-standard withdraw |
| R-07 | "Redeem 1000000000000000 vault shares" | `archimedes redeem --vault ... --shares 1000000000000000` | PASS | 4-param non-standard redeem |
| R-08 | "Exit my entire Archimedes position (redeem all)" | `archimedes redeem --vault ...` (no --shares) | PASS | Omitting --shares redeems all |
| R-09 | "List Archimedes vaults with custom RPC" | `archimedes vaults --rpc <url>` | PASS | --rpc flag supported on all cmds |
| R-10 | "Show Archimedes positions for wallet 0x..." | `archimedes positions --wallet 0x...` | PASS | --wallet flag for explicit address |

---

## Command Flag Coverage

| Flag | Commands | Verified in SKILL.md |
|------|----------|----------------------|
| `--vault <ADDR>` | deposit, withdraw, redeem | Yes |
| `--amount <AMOUNT>` | deposit, withdraw | Yes |
| `--shares <AMOUNT>` | redeem | Yes (optional, omit for all) |
| `--from <ADDR>` | deposit, withdraw, redeem | Yes (defaults to wallet) |
| `--wallet <ADDR>` | positions | Yes (defaults to wallet) |
| `--rpc <URL>` | all commands | Yes |
| `--dry-run` | deposit, withdraw, redeem | Yes |
| `--slippage-bps <N>` | withdraw, redeem | Yes (default 50) |

---

## Routing Issues Found

None. The command structure exactly matches SKILL.md. All intent patterns map to clear, unambiguous subcommands with documented parameters.

---

## Notes

- The SKILL.md uses vault addresses (e.g. `0xfA364CBca915f17fEc356E35B61541fC6D4D8269`) not aliases (ylstETH). Users must pass full addresses or rely on agent to look up via `vaults` command.
- The "ask user to confirm" pattern is correctly printed to stderr before each tx submission.
- The plugin has no vault alias lookup (no `--vault ylstETH` shorthand). This is consistent with SKILL.md examples.

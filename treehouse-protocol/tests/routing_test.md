# Routing Tests — treehouse-protocol

Generated: 2026-04-05

## Skill Trigger Description (from SKILL.md)

The `treehouse-protocol` skill is triggered by mentions of:
- Treehouse Protocol, tETH, tAVAX
- Deposit/stake ETH/WETH/stETH/wstETH to Treehouse
- Deposit/stake AVAX/sAVAX to Treehouse
- tETH/tAVAX balance, price, APY
- Redeem/withdraw tETH via Curve
- Fixed income, MEY token

## Positive Tests (3 per command)

### deposit command

| # | User Utterance | Expected Command | Expected Flags |
|---|---------------|-----------------|----------------|
| P1 | "deposit 1 ETH to Treehouse to get tETH" | `deposit` | `--token ETH --amount 1.0 --chain 1` |
| P2 | "stake 0.5 wstETH on Treehouse Protocol" | `deposit` | `--token wstETH --amount 0.5 --chain 1` |
| P3 | "deposit 10 AVAX to Treehouse for tAVAX yield" | `deposit` | `--token AVAX --amount 10.0 --chain 43114` |

### balance command

| # | User Utterance | Expected Command | Expected Flags |
|---|---------------|-----------------|----------------|
| P4 | "what is my tETH balance?" | `balance` | `--chain 1` |
| P5 | "how much tAVAX do I have on Treehouse?" | `balance` | `--chain 43114` |
| P6 | "check my Treehouse balance" | `balance` | `--chain 1` |

### price command

| # | User Utterance | Expected Command | Expected Flags |
|---|---------------|-----------------|----------------|
| P7 | "what is the tETH price in wstETH?" | `price` | `--chain 1` |
| P8 | "tAVAX exchange rate vs sAVAX" | `price` | `--chain 43114` |
| P9 | "Treehouse exchange rate on Ethereum" | `price` | `--chain 1` |

### positions command

| # | User Utterance | Expected Command | Expected Flags |
|---|---------------|-----------------|----------------|
| P10 | "show my Treehouse position" | `positions` | `--chain 1` |
| P11 | "what is tETH APY right now?" | `positions` | `--chain 1` |
| P12 | "Treehouse tAVAX yield and TVL on Avalanche" | `positions` | `--chain 43114` |

### withdraw command

| # | User Utterance | Expected Command | Expected Flags |
|---|---------------|-----------------|----------------|
| P13 | "redeem 5 tETH back to wstETH" | `withdraw` | `--amount 5.0 --chain 1` |
| P14 | "withdraw my tETH from Treehouse" | `withdraw` | `--chain 1` |
| P15 | "sell tETH get wstETH Treehouse" | `withdraw` | `--chain 1` |

## Negative Tests (3 global)

| # | User Utterance | Expected Routing | Reason NOT treehouse-protocol |
|---|---------------|-----------------|-------------------------------|
| N1 | "stake ETH on Lido to get stETH" | Lido plugin | Lido is explicitly excluded in SKILL.md |
| N2 | "deposit ETH to Aave for yield" | Aave plugin | Aave is lending protocol, explicitly excluded |
| N3 | "deposit cmETH on Mantle" | mETH/Mantle plugin | mETH Protocol is explicitly excluded |

## Self-Evaluation

### Parameter Mapping Accuracy

The SKILL.md documents `deposit` using `--token` and `--chain` flags — which matches the binary CLI exactly.
- `balance` and `price` and `positions` use `--chain` (not `--asset`) — verified in source.
- `withdraw` uses `--amount`, `--chain`, and optional `--slippage-bps`.
- The pipeline tester instructions say `price --asset tETH --chain 1` but the binary only takes `--chain` for price; this is an instruction quirk; the actual test command should be `price --chain 1`.

### Issues Found

None. The SKILL.md command examples are consistent with the binary's actual CLI flags.

### Result: PASS

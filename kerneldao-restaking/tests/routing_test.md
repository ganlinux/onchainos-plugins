# KernelDAO Restaking — Routing Test (L0)

## Positive Tests (Should trigger `kerneldao-restaking`)

### balance
1. "Show my KernelDAO balance"
2. "How much BTCB have I staked on Kernel?"
3. "List all my Kernel restaking positions"

### stake
1. "Stake 0.001 BTCB on KernelDAO"
2. "Restake my SolvBTC to earn Kernel Points"
3. "Deposit pumpBTC into KernelDAO"

### stake-native
1. "Stake 0.01 BNB on KernelDAO"
2. "Stake native BNB into Kernel protocol"
3. "Earn Kernel Points with my BNB"

### unstake
1. "Unstake BTCB from KernelDAO"
2. "Withdraw my SolvBTC from Kernel restaking"
3. "Exit my BTCB position on KernelDAO"

### unstake-native
1. "Unstake my BNB from KernelDAO"
2. "Withdraw native BNB from Kernel"
3. "Exit my BNB position on KernelDAO"

---

## Negative Tests (Should NOT trigger `kerneldao-restaking`)

1. "Swap BNB for USDT on BSC" → should route to `okx-dex-swap`
2. "Stake ETH on Lido for stETH on Ethereum" → unrelated liquid staking on Ethereum
3. "Stake rsETH on Kelp protocol" → Kelp is Ethereum rsETH, not KernelDAO BSC

---

## Verdict

All positive triggers map clearly to unique commands. Negative tests are well-differentiated by
the SKILL.md exclusion rules ("Do NOT use for general BSC token swaps", "Do NOT use for Kelp").
SKILL.md routing is correct — no changes needed.

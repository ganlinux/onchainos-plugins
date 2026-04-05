# Stargate V2 Routing Test

## Positive Cases (should trigger stargate-v2)

### quote command
1. "Get me a quote to bridge 100 USDC from Arbitrum to Base"
2. "How much will I receive if I send 0.5 ETH from Ethereum to Optimism via Stargate?"
3. "Quote bridging 500 USDT from BNB chain to Avalanche using bus mode"

### send command
1. "Bridge 1 USDC from Arbitrum to Base using Stargate V2"
2. "Send 0.1 ETH cross-chain from Ethereum to Arbitrum via LayerZero"
3. "Transfer 100 USDT from Polygon to BNB Chain with Stargate, dry run first"

### status command
1. "Check the status of my Stargate bridge transaction 0xabc123def456..."
2. "What is the delivery status of my LayerZero message with tx hash 0x1234abcd?"
3. "Show my recent cross-chain transfer history for wallet 0xMyAddress"

### pools command
1. "List all Stargate V2 supported pools and chains"
2. "Show me which chains support USDC bridging on Stargate"
3. "What tokens can I bridge on Arbitrum using Stargate V2?"

## Negative Cases (should NOT trigger stargate-v2)

1. "Swap ETH for USDC on Uniswap" — this is a DEX swap, not a cross-chain bridge
2. "Stake my ETH on Lido to get stETH" — this is liquid staking, not bridging
3. "Add liquidity to a Curve pool for yield farming" — this is LP provisioning, not cross-chain transfer

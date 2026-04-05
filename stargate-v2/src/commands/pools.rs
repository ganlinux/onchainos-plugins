use clap::Args;

#[derive(Args, Debug)]
pub struct PoolsArgs {
    /// Filter by chain name or chain ID (optional)
    #[arg(long)]
    pub chain: Option<String>,

    /// Filter by token symbol (optional)
    #[arg(long)]
    pub token: Option<String>,
}

struct PoolEntry {
    chain_id: u64,
    chain_name: &'static str,
    token: &'static str,
    pool_addr: &'static str,
    is_native: bool,
    eid: u32,
}

const POOLS: &[PoolEntry] = &[
    PoolEntry { chain_id: 1,      chain_name: "Ethereum",   token: "ETH",  pool_addr: "0x77b2043768d28E9C9aB44E1aBfC95944bcE57931", is_native: true,  eid: 30101 },
    PoolEntry { chain_id: 1,      chain_name: "Ethereum",   token: "USDC", pool_addr: "0xc026395860Db2d07ee33e05fE50ed7bD583189C7", is_native: false, eid: 30101 },
    PoolEntry { chain_id: 1,      chain_name: "Ethereum",   token: "USDT", pool_addr: "0x933597a323Eb81cAe705C5bC29985172fd5A3973", is_native: false, eid: 30101 },
    PoolEntry { chain_id: 42161,  chain_name: "Arbitrum",   token: "ETH",  pool_addr: "0xA45B5130f36CDcA45667738e2a258AB09f4A5f7F", is_native: true,  eid: 30110 },
    PoolEntry { chain_id: 42161,  chain_name: "Arbitrum",   token: "USDC", pool_addr: "0xe8CDF27AcD73a434D661C84887215F7598e7d0d3", is_native: false, eid: 30110 },
    PoolEntry { chain_id: 42161,  chain_name: "Arbitrum",   token: "USDT", pool_addr: "0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0", is_native: false, eid: 30110 },
    PoolEntry { chain_id: 10,     chain_name: "OP Mainnet", token: "ETH",  pool_addr: "0xe8CDF27AcD73a434D661C84887215F7598e7d0d3", is_native: true,  eid: 30111 },
    PoolEntry { chain_id: 10,     chain_name: "OP Mainnet", token: "USDC", pool_addr: "0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0", is_native: false, eid: 30111 },
    PoolEntry { chain_id: 8453,   chain_name: "Base",       token: "ETH",  pool_addr: "0xdc181Bd607330aeeBEF6ea62e03e5e1Fb4B6F7C7", is_native: true,  eid: 30184 },
    PoolEntry { chain_id: 8453,   chain_name: "Base",       token: "USDC", pool_addr: "0x27a16dc786820B16E5c9028b75B99F6f604b5d26", is_native: false, eid: 30184 },
    PoolEntry { chain_id: 137,    chain_name: "Polygon",    token: "USDC", pool_addr: "0x9Aa02D4Fae7F58b8E8f34c66E756cC734DAc7fe4", is_native: false, eid: 30109 },
    PoolEntry { chain_id: 137,    chain_name: "Polygon",    token: "USDT", pool_addr: "0xd47b03ee6d86Cf251ee7860FB2ACf9f91B9fD4d7", is_native: false, eid: 30109 },
    PoolEntry { chain_id: 56,     chain_name: "BNB Chain",  token: "USDC", pool_addr: "0x962Bd449E630b0d928f308Ce63f1A21F02576057", is_native: false, eid: 30102 },
    PoolEntry { chain_id: 56,     chain_name: "BNB Chain",  token: "USDT", pool_addr: "0x138EB30f73BC423c6455C53df6D89CB01d9eBc63", is_native: false, eid: 30102 },
    PoolEntry { chain_id: 43114,  chain_name: "Avalanche",  token: "USDC", pool_addr: "0x5634c4a5FEd09819E3c46D86A965Dd9447d86e47", is_native: false, eid: 30106 },
    PoolEntry { chain_id: 43114,  chain_name: "Avalanche",  token: "USDT", pool_addr: "0x12dC9256Acc9895B076f6638D628382881e62CeE", is_native: false, eid: 30106 },
];

pub async fn run(args: PoolsArgs) -> anyhow::Result<()> {
    let chain_filter = args.chain.as_deref().map(|s| s.to_lowercase());
    let token_filter = args.token.as_deref().map(|s| s.to_uppercase());

    let filtered: Vec<&PoolEntry> = POOLS
        .iter()
        .filter(|p| {
            if let Some(ref cf) = chain_filter {
                let chain_match = p.chain_name.to_lowercase().contains(cf.as_str())
                    || p.chain_id.to_string() == *cf;
                if !chain_match {
                    return false;
                }
            }
            if let Some(ref tf) = token_filter {
                if p.token != tf.as_str() {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        println!("No pools match the given filters.");
        return Ok(());
    }

    println!(
        "{:<12} {:>8} {:>8} {:>6} {:<46} {}",
        "Chain", "ChainID", "EID", "Token", "Pool Address", "Type"
    );
    println!("{}", "-".repeat(100));

    for p in filtered {
        println!(
            "{:<12} {:>8} {:>8} {:>6} {:<46} {}",
            p.chain_name,
            p.chain_id,
            p.eid,
            p.token,
            p.pool_addr,
            if p.is_native { "native" } else { "ERC-20" }
        );
    }

    println!();
    println!("Total: {} pool(s)", POOLS.len());
    println!();
    println!("Notes:");
    println!("  - Native pools: no ERC-20 approve needed; msg.value includes bridge amount.");
    println!("  - ERC-20 pools: approve token to pool contract before calling sendToken.");
    println!("  - Supported modes: taxi (fast, ~1-3 min) and bus (cheap, ~5-20 min).");

    Ok(())
}

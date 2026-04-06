// GraphQL API client for Fenix Finance Goldsky subgraph
//
// Sample response from GetUserPositions query:
// {
//   "data": {
//     "positions": [
//       {
//         "id": "1234",
//         "owner": "0x...",
//         "pool": {
//           "id": "0x...",
//           "token0": { "id": "0x...", "symbol": "USDB", "decimals": "18" },
//           "token1": { "id": "0x...", "symbol": "WETH", "decimals": "18" },
//           "sqrtPrice": "...",
//           "tick": "...",
//           "feeTier": "100"
//         },
//         "tickLower": { "tickIdx": "-100" },
//         "tickUpper": { "tickIdx": "200" },
//         "liquidity": "1000000",
//         "collectedFeesToken0": "0",
//         "collectedFeesToken1": "0"
//       }
//     ]
//   }
// }

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

use crate::config::SUBGRAPH_V3_URL;

fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
    {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            builder = builder.proxy(proxy);
        }
    }
    builder.build().unwrap_or_default()
}

async fn graphql_query(query: &str, variables: Value) -> anyhow::Result<Value> {
    let client = build_client();
    let body = serde_json::json!({
        "query": query,
        "variables": variables
    });
    let resp = client
        .post(SUBGRAPH_V3_URL)
        .json(&body)
        .send()
        .await
        .context("subgraph request failed")?
        .json::<Value>()
        .await
        .context("subgraph response parse")?;
    if let Some(errors) = resp.get("errors") {
        anyhow::bail!("subgraph errors: {}", errors);
    }
    Ok(resp["data"].clone())
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct SubgraphToken {
    pub id: String,
    pub symbol: String,
    pub decimals: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct SubgraphTick {
    #[serde(rename = "tickIdx")]
    pub tick_idx: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct SubgraphPool {
    pub id: String,
    pub token0: SubgraphToken,
    pub token1: SubgraphToken,
    #[serde(rename = "sqrtPrice", default)]
    pub sqrt_price: String,
    #[serde(default)]
    pub tick: String,
    #[serde(rename = "feeTier", default)]
    pub fee_tier: String,
    #[serde(rename = "token0Price", default)]
    pub token0_price: String,
    #[serde(rename = "token1Price", default)]
    pub token1_price: String,
    #[serde(rename = "volumeUSD", default)]
    pub volume_usd: String,
    #[serde(default)]
    pub liquidity: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct SubgraphPosition {
    pub id: String,
    pub owner: String,
    pub pool: SubgraphPool,
    #[serde(rename = "tickLower")]
    pub tick_lower: SubgraphTick,
    #[serde(rename = "tickUpper")]
    pub tick_upper: SubgraphTick,
    pub liquidity: String,
    #[serde(rename = "collectedFeesToken0", default)]
    pub collected_fees_token0: String,
    #[serde(rename = "collectedFeesToken1", default)]
    pub collected_fees_token1: String,
}

/// Get user LP positions from subgraph
pub async fn get_user_positions(owner: &str) -> anyhow::Result<Vec<SubgraphPosition>> {
    let query = r#"
    query GetUserPositions($owner: String!) {
      positions(where: { owner: $owner, liquidity_gt: "0" }) {
        id
        owner
        pool {
          id
          token0 { id symbol decimals }
          token1 { id symbol decimals }
          sqrtPrice
          tick
          feeTier
          token0Price
          token1Price
          volumeUSD
          liquidity
        }
        tickLower { tickIdx }
        tickUpper { tickIdx }
        liquidity
        collectedFeesToken0
        collectedFeesToken1
      }
    }
    "#;
    let vars = serde_json::json!({ "owner": owner.to_lowercase() });
    let data = graphql_query(query, vars).await?;
    let positions: Vec<SubgraphPosition> =
        serde_json::from_value(data["positions"].clone()).context("parse positions")?;
    Ok(positions)
}

/// Get pool info from subgraph (reserved for future use)
#[allow(dead_code)]
pub async fn get_pool_info(token0: &str, token1: &str) -> anyhow::Result<Option<SubgraphPool>> {
    let query = r#"
    query GetPool($token0: String!, $token1: String!) {
      pools(
        where: {
          token0_: { id: $token0 }
          token1_: { id: $token1 }
        }
      ) {
        id
        token0Price
        token1Price
        sqrtPrice
        tick
        feeTier
        liquidity
        volumeUSD
        token0 { id symbol decimals }
        token1 { id symbol decimals }
      }
    }
    "#;

    // Try both orderings
    let vars = serde_json::json!({
        "token0": token0.to_lowercase(),
        "token1": token1.to_lowercase()
    });
    let data = graphql_query(query, vars).await?;
    let mut pools: Vec<SubgraphPool> =
        serde_json::from_value(data["pools"].clone()).unwrap_or_default();

    if pools.is_empty() {
        // try swapped ordering
        let vars2 = serde_json::json!({
            "token0": token1.to_lowercase(),
            "token1": token0.to_lowercase()
        });
        let data2 = graphql_query(query, vars2).await?;
        pools = serde_json::from_value(data2["pools"].clone()).unwrap_or_default();
    }

    Ok(pools.into_iter().next())
}

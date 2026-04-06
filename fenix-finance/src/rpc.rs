use anyhow::Context;
use serde_json::Value;

use crate::config::{BLAST_RPC_FALLBACK, BLAST_RPC_URL};

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

/// Low-level eth_call via JSON-RPC
pub async fn eth_call(to: &str, data: &str) -> anyhow::Result<String> {
    let client = build_client();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": data },
            "latest"
        ],
        "id": 1
    });

    // Try primary RPC, fallback on error
    let resp = client
        .post(BLAST_RPC_URL)
        .json(&body)
        .send()
        .await;

    let json: Value = match resp {
        Ok(r) => r.json().await.context("eth_call primary RPC parse")?,
        Err(_) => {
            client
                .post(BLAST_RPC_FALLBACK)
                .json(&body)
                .send()
                .await
                .context("eth_call fallback RPC")?
                .json()
                .await
                .context("eth_call fallback RPC parse")?
        }
    };

    if let Some(err) = json.get("error") {
        anyhow::bail!("eth_call error: {}", err);
    }
    let result = json["result"]
        .as_str()
        .context("eth_call: missing result")?
        .to_string();
    Ok(result)
}

/// Decode a 32-byte hex word (at position word_index) as u128
pub fn decode_word_u128(hex_data: &str, word_index: usize) -> anyhow::Result<u128> {
    let data = hex_data.trim_start_matches("0x");
    let start = word_index * 64;
    let end = start + 64;
    if data.len() < end {
        anyhow::bail!("decode_word_u128: data too short ({} < {})", data.len(), end);
    }
    let word = &data[start..end];
    u128::from_str_radix(word, 16).context("decode_word_u128: parse hex")
}

/// Decode a 32-byte hex word as address (last 20 bytes)
pub fn decode_word_address(hex_data: &str, word_index: usize) -> anyhow::Result<String> {
    let data = hex_data.trim_start_matches("0x");
    let start = word_index * 64;
    let end = start + 64;
    if data.len() < end {
        anyhow::bail!("decode_word_address: data too short");
    }
    let word = &data[start..end];
    // address is last 40 hex chars (20 bytes)
    Ok(format!("0x{}", &word[24..]))
}

/// Decode word as i32 (int24 from ABI — take last 6 hex chars, sign-extend)
pub fn decode_word_int24(hex_data: &str, word_index: usize) -> anyhow::Result<i32> {
    let data = hex_data.trim_start_matches("0x");
    let start = word_index * 64;
    let end = start + 64;
    if data.len() < end {
        anyhow::bail!("decode_word_int24: data too short");
    }
    let word = &data[start..end];
    // int24: take last 6 hex chars (3 bytes), sign-extend
    let raw = u32::from_str_radix(&word[58..64], 16).context("decode_word_int24: parse")?;
    // sign extend from bit 23
    let signed = if raw & 0x800000 != 0 {
        (raw | 0xFF000000) as i32
    } else {
        raw as i32
    };
    Ok(signed)
}

/// eth_getBlockByNumber to get current timestamp
pub async fn get_block_timestamp() -> anyhow::Result<u64> {
    let client = build_client();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": ["latest", false],
        "id": 1
    });
    let json: Value = client
        .post(BLAST_RPC_URL)
        .json(&body)
        .send()
        .await
        .context("get_block_timestamp")?
        .json()
        .await
        .context("get_block_timestamp parse")?;
    let ts_hex = json["result"]["timestamp"]
        .as_str()
        .context("get_block_timestamp: missing timestamp")?;
    let ts = u64::from_str_radix(ts_hex.trim_start_matches("0x"), 16)
        .context("get_block_timestamp: parse hex")?;
    Ok(ts)
}

fn serialize_u128_as_string<S>(val: &u128, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&val.to_string())
}

/// Position data from NFPM.positions(tokenId)
#[derive(Debug, serde::Serialize)]
pub struct PositionData {
    pub token_id: u64,
    pub token0: String,
    pub token1: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    #[serde(serialize_with = "serialize_u128_as_string")]
    pub liquidity: u128,
    #[serde(serialize_with = "serialize_u128_as_string")]
    pub tokens_owed0: u128,
    #[serde(serialize_with = "serialize_u128_as_string")]
    pub tokens_owed1: u128,
}

/// Query NFPM.positions(tokenId)
/// Returns: (uint96 nonce, address operator, address token0, address token1,
///           int24 tickLower, int24 tickUpper, uint128 liquidity,
///           uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128,
///           uint128 tokensOwed0, uint128 tokensOwed1)
pub async fn nfpm_positions(nfpm: &str, token_id: u64) -> anyhow::Result<PositionData> {
    // positions(uint256) selector = 0x99fbab88
    let token_id_hex = format!("{:064x}", token_id);
    let data = format!("0x99fbab88{}", token_id_hex);
    let result = eth_call(nfpm, &data).await?;
    let hex = result.trim_start_matches("0x");

    // word 0: nonce (uint96)
    // word 1: operator (address)
    // word 2: token0 (address)
    // word 3: token1 (address)
    // word 4: tickLower (int24)
    // word 5: tickUpper (int24)
    // word 6: liquidity (uint128)
    // word 7: feeGrowthInside0LastX128 (uint256)
    // word 8: feeGrowthInside1LastX128 (uint256)
    // word 9: tokensOwed0 (uint128)
    // word 10: tokensOwed1 (uint128)

    let token0 = decode_word_address(hex, 2)?;
    let token1 = decode_word_address(hex, 3)?;
    let tick_lower = decode_word_int24(hex, 4)?;
    let tick_upper = decode_word_int24(hex, 5)?;
    let liquidity = decode_word_u128(hex, 6)?;
    let tokens_owed0 = decode_word_u128(hex, 9)?;
    let tokens_owed1 = decode_word_u128(hex, 10)?;

    Ok(PositionData {
        token_id,
        token0,
        token1,
        tick_lower,
        tick_upper,
        liquidity,
        tokens_owed0,
        tokens_owed1,
    })
}

/// NFPM.balanceOf(address) -> u64
pub async fn nfpm_balance_of(nfpm: &str, owner: &str) -> anyhow::Result<u64> {
    // balanceOf(address) selector = 0x70a08231
    let owner_clean = owner.trim_start_matches("0x");
    let owner_padded = format!("{:0>64}", owner_clean);
    let data = format!("0x70a08231{}", owner_padded);
    let result = eth_call(nfpm, &data).await?;
    let count = decode_word_u128(&result, 0)? as u64;
    Ok(count)
}

/// NFPM.tokenOfOwnerByIndex(address, uint256) -> u64
pub async fn nfpm_token_of_owner_by_index(
    nfpm: &str,
    owner: &str,
    index: u64,
) -> anyhow::Result<u64> {
    // tokenOfOwnerByIndex(address,uint256) selector = 0x2f745c59
    let owner_clean = owner.trim_start_matches("0x");
    let owner_padded = format!("{:0>64}", owner_clean);
    let index_hex = format!("{:064x}", index);
    let data = format!("0x2f745c59{}{}", owner_padded, index_hex);
    let result = eth_call(nfpm, &data).await?;
    let token_id = decode_word_u128(&result, 0)? as u64;
    Ok(token_id)
}

/// AlgebraFactory.poolByPair(tokenA, tokenB) -> address
pub async fn factory_pool_by_pair(
    factory: &str,
    token_a: &str,
    token_b: &str,
) -> anyhow::Result<String> {
    // poolByPair(address,address) selector = 0xd9a641e1
    let a_clean = token_a.trim_start_matches("0x");
    let b_clean = token_b.trim_start_matches("0x");
    let a_padded = format!("{:0>64}", a_clean);
    let b_padded = format!("{:0>64}", b_clean);
    let data = format!("0xd9a641e1{}{}", a_padded, b_padded);
    let result = eth_call(factory, &data).await?;
    let addr = decode_word_address(&result, 0)?;
    Ok(addr)
}

/// ERC-20 allowance(owner, spender) -> u128
pub async fn erc20_allowance(token: &str, owner: &str, spender: &str) -> anyhow::Result<u128> {
    // allowance(address,address) selector = 0xdd62ed3e
    let owner_clean = owner.trim_start_matches("0x");
    let spender_clean = spender.trim_start_matches("0x");
    let data = format!(
        "0xdd62ed3e{:0>64}{:0>64}",
        owner_clean, spender_clean
    );
    let result = eth_call(token, &data).await?;
    decode_word_u128(&result, 0)
}

/// QuoterV2.quoteExactInputSingle((tokenIn,tokenOut,amountIn,limitSqrtPrice)) -> amountOut
/// Selector: 0x5e5e6e0f
pub async fn quoter_quote_exact_input_single(
    quoter: &str,
    token_in: &str,
    token_out: &str,
    amount_in: u128,
) -> anyhow::Result<u128> {
    // quoteExactInputSingle((address,address,uint256,uint160))
    // ABI encode: selector + tuple (static 4 words)
    let in_clean = token_in.trim_start_matches("0x");
    let out_clean = token_out.trim_start_matches("0x");
    // For tuple input: offset to tuple start = 0x20
    // Actually for struct input in solidity, the tuple is encoded inline
    // The function takes a single tuple param — ABI encodes as head=offset to tuple, then tuple contents
    // Since the struct has no dynamic fields, it encodes as 4 static words
    let data = format!(
        "0x5e5e6e0f{:0>64}{:0>64}{:064x}{:064x}",
        in_clean,
        out_clean,
        amount_in,
        0u128 // limitSqrtPrice = 0
    );
    let result = eth_call(quoter, &data).await?;
    // Returns: (uint256 amountOut, uint256 amountIn, uint160 sqrtPriceX96After, uint32 initializedTicksCrossed, uint256 gasEstimate, uint16 fee)
    decode_word_u128(&result, 0)
}

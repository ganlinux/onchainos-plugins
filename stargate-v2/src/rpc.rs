use anyhow::Context;
use serde_json::{json, Value};

pub fn build_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .http1_only()
        .timeout(std::time::Duration::from_secs(30));
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

/// Low-level eth_call via JSON-RPC.
pub async fn eth_call(
    rpc_url: &str,
    to: &str,
    calldata: &str,
    block: &str,
) -> anyhow::Result<String> {
    let client = build_client();
    let body = json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            { "to": to, "data": calldata },
            block
        ],
        "id": 1
    });
    let resp: Value = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .context("eth_call HTTP request failed")?
        .json()
        .await
        .context("eth_call response parse failed")?;
    if let Some(err) = resp.get("error") {
        anyhow::bail!("eth_call RPC error: {}", err);
    }
    let result = resp["result"]
        .as_str()
        .unwrap_or("0x")
        .to_string();
    Ok(result)
}

/// Read token() from a Stargate pool — returns address (lower-hex with 0x prefix).
/// Selector 0xfc0c546a
pub async fn get_pool_token(rpc_url: &str, pool: &str) -> anyhow::Result<String> {
    let result = eth_call(rpc_url, pool, "0xfc0c546a", "latest").await?;
    // result is 32-byte hex; address is last 20 bytes
    let hex = result.trim_start_matches("0x");
    if hex.len() < 40 {
        return Ok("0x0000000000000000000000000000000000000000".to_string());
    }
    let addr = &hex[hex.len() - 40..];
    Ok(format!("0x{}", addr))
}

/// Read allowance(owner, spender) from an ERC-20 token.
/// Selector 0xdd62ed3e
pub async fn get_allowance(
    rpc_url: &str,
    token: &str,
    owner: &str,
    spender: &str,
) -> anyhow::Result<u128> {
    let owner_stripped = owner.strip_prefix("0x").unwrap_or(owner);
    let spender_stripped = spender.strip_prefix("0x").unwrap_or(spender);
    let calldata = format!(
        "0xdd62ed3e{:0>64}{:0>64}",
        owner_stripped, spender_stripped
    );
    let result = eth_call(rpc_url, token, &calldata, "latest").await?;
    let hex = result.trim_start_matches("0x");
    if hex.is_empty() {
        return Ok(0);
    }
    Ok(u128::from_str_radix(&hex[hex.len().saturating_sub(32)..], 16).unwrap_or(0))
}

/// Encode a SendParam tuple to ABI bytes (without selector).
/// SendParam: (uint32 dstEid, bytes32 to, uint256 amountLD, uint256 minAmountLD,
///             bytes extraOptions, bytes composeMsg, bytes oftCmd)
/// Bytes fields are dynamic; the tuple is encoded as a top-level tuple with heads + tails.
pub fn encode_send_param(
    dst_eid: u32,
    to_bytes32: &str,    // 64 hex chars, no 0x
    amount_ld: u128,
    min_amount_ld: u128,
    oft_cmd_bus: bool,   // true = bus (0x00), false = taxi (empty)
) -> String {
    // Static fields offset: 5 static slots (dstEid=1, to=1, amountLD=1, minAmountLD=1) +
    // 3 dynamic bytes refs = 7 words = 7*32 = 224 bytes = 0xe0
    // Layout of the tuple ABI encoding (as if it's the body of an abi.encode call):
    // slot 0: dstEid (padded uint32)
    // slot 1: to (bytes32)
    // slot 2: amountLD (uint256)
    // slot 3: minAmountLD (uint256)
    // slot 4: offset of extraOptions  -> points to word 7 (0xe0)
    // slot 5: offset of composeMsg    -> points to word 9 (0x120) [empty = 32+0]
    // slot 6: offset of oftCmd        -> points to word 11 (0x160) [empty or 1-byte]
    // Then the dynamic data:
    // word 7: length of extraOptions = 0
    // word 8: (padding, no data)   [skip if len=0 — but ABI always includes len word]
    // Actually for empty bytes: just one word with value 0 (length=0, no data words).
    // For oftCmd bus (0x00 = single byte 0x00): len=1, data=0x00 padded to 32 bytes.

    let dst_eid_hex = format!("{:064x}", dst_eid as u64);
    let to_hex = format!("{:0>64}", to_bytes32);
    let amount_hex = format!("{:064x}", amount_ld);
    let min_amount_hex = format!("{:064x}", min_amount_ld);

    // offsets relative to start of the tuple encoding
    // static part = 7 words = 0xe0 bytes
    let offset_extra: u64 = 0xe0;          // word 7
    let offset_compose: u64 = 0xe0 + 0x20; // word 8 (extra has len=0, no data words)
    let offset_oft_cmd: u64 = offset_compose + 0x20; // word 9 (compose has len=0, no data)

    let offset_extra_hex = format!("{:064x}", offset_extra);
    let offset_compose_hex = format!("{:064x}", offset_compose);
    let offset_oft_cmd_hex = format!("{:064x}", offset_oft_cmd);

    // Dynamic data: extraOptions = empty bytes
    let extra_data = format!("{:064x}", 0u64); // len=0
    // composeMsg = empty bytes
    let compose_data = format!("{:064x}", 0u64); // len=0
    // oftCmd: taxi = empty, bus = single byte 0x00
    let oft_cmd_data = if oft_cmd_bus {
        // len=1, data = 0x00 padded to 32
        format!("{:064x}{:064x}", 1u64, 0u64)
    } else {
        // len=0 (empty = taxi)
        format!("{:064x}", 0u64)
    };

    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        dst_eid_hex,
        to_hex,
        amount_hex,
        min_amount_hex,
        offset_extra_hex,
        offset_compose_hex,
        offset_oft_cmd_hex,
        extra_data,
        compose_data,
        oft_cmd_data
    )
}

/// Call quoteOFT on chain and return (amount_sent_ld, amount_received_ld).
/// Selector 0x0d35b415
pub async fn quote_oft(
    rpc_url: &str,
    pool: &str,
    dst_eid: u32,
    to_bytes32: &str,
    amount_ld: u128,
    bus_mode: bool,
) -> anyhow::Result<(u128, u128)> {
    // For quoteOFT: pass minAmountLD = amountLD initially (it's the first call)
    let param_body = encode_send_param(dst_eid, to_bytes32, amount_ld, amount_ld, bus_mode);
    // quoteOFT takes a single tuple arg — ABI encoding: offset to tuple = 0x20, then tuple
    // Actually the function takes (SendParam) — a single struct parameter.
    // ABI encoding of a single tuple param: the tuple is encoded as a sequence of its fields.
    // For a function with one tuple argument, the calldata after selector is just the ABI encoding
    // of the tuple (no extra offset since tuple fields are laid out inline when it's calldata).
    // CORRECTION: In Solidity ABI, a struct (tuple) passed as calldata argument is encoded
    // starting with an offset pointer if any of its fields are dynamic. Since bytes fields
    // are dynamic, the encoding starts with the offset word (0x20), then the tuple body.
    let calldata = format!("0x0d35b415{:064x}{}", 0x20u64, param_body);
    let result = eth_call(rpc_url, pool, &calldata, "latest").await?;
    decode_quote_oft_result(&result)
}

/// Decode quoteOFT return: (OFTLimit, OFTFeeDetail[], OFTReceipt)
/// We only need OFTReceipt which is the last element:
///   struct OFTReceipt { uint256 amountSentLD; uint256 amountReceivedLD; }
///
/// Actual on-chain ABI layout observed from Stargate V2 pools:
///   Word 0: OFTLimit.minAmountLD  (static uint256, inlined)
///   Word 1: OFTLimit.maxAmountLD  (static uint256, inlined)
///   Word 2: offset to OFTFeeDetail[] dynamic array
///   Word 3: OFTReceipt.amountSentLD     <-- we want this
///   Word 4: OFTReceipt.amountReceivedLD <-- and this
///   Word 5+: OFTFeeDetail[] array data
fn decode_quote_oft_result(hex: &str) -> anyhow::Result<(u128, u128)> {
    let data = hex.trim_start_matches("0x");
    if data.len() < 64 {
        anyhow::bail!("quoteOFT result too short");
    }
    let words: Vec<&str> = (0..data.len() / 64).map(|i| &data[i * 64..(i + 1) * 64]).collect();
    if words.len() < 5 {
        anyhow::bail!("quoteOFT result has too few words (expected >= 5, got {})", words.len());
    }
    // Words 3,4 are OFTReceipt.amountSentLD and OFTReceipt.amountReceivedLD
    let amount_sent = u128::from_str_radix(words[3], 16).unwrap_or(0);
    let amount_recv = u128::from_str_radix(words[4], 16).unwrap_or(0);
    Ok((amount_sent, amount_recv))
}

/// Call quoteSend on chain and return (native_fee, lz_token_fee).
/// Selector 0x3b6f743b
pub async fn quote_send(
    rpc_url: &str,
    pool: &str,
    dst_eid: u32,
    to_bytes32: &str,
    amount_ld: u128,
    min_amount_ld: u128,
    bus_mode: bool,
) -> anyhow::Result<(u128, u128)> {
    let param_body = encode_send_param(dst_eid, to_bytes32, amount_ld, min_amount_ld, bus_mode);
    // quoteSend(SendParam, bool) — ABI layout:
    // arg0 (SendParam dynamic tuple): offset from start of args = 2 words = 0x40
    // arg1 (bool static): inlined at word 1, false = 0
    // Then SendParam tuple data follows
    let bool_hex = format!("{:064x}", 0u64); // false = pay in native token
    let calldata = format!(
        "0x3b6f743b{:064x}{}{}",
        0x40u64,  // offset to SendParam tuple (from start of args)
        bool_hex, // bool false
        param_body
    );
    let result = eth_call(rpc_url, pool, &calldata, "latest").await?;
    decode_messaging_fee(&result)
}

/// Decode MessagingFee { uint256 nativeFee; uint256 lzTokenFee; }
fn decode_messaging_fee(hex: &str) -> anyhow::Result<(u128, u128)> {
    let data = hex.trim_start_matches("0x");
    if data.len() < 128 {
        anyhow::bail!("quoteSend result too short: {}", hex);
    }
    let native_fee = u128::from_str_radix(&data[0..64], 16)
        .map_err(|e| anyhow::anyhow!("decode nativeFee: {}", e))?;
    let lz_token_fee = u128::from_str_radix(&data[64..128], 16)
        .map_err(|e| anyhow::anyhow!("decode lzTokenFee: {}", e))?;
    Ok((native_fee, lz_token_fee))
}

/// Convert an address string to 32-byte hex (no 0x), left-padded with zeros.
pub fn address_to_bytes32(addr: &str) -> String {
    let stripped = addr.strip_prefix("0x").unwrap_or(addr);
    format!("{:0>64}", stripped)
}

/// Encode sendToken calldata.
/// sendToken(SendParam, MessagingFee, address) selector = 0xcbef2aa9
pub fn encode_send_token(
    dst_eid: u32,
    to_bytes32: &str,
    amount_ld: u128,
    min_amount_ld: u128,
    native_fee: u128,
    refund_address: &str,
    bus_mode: bool,
) -> String {
    let param_body = encode_send_param(dst_eid, to_bytes32, amount_ld, min_amount_ld, bus_mode);
    let native_fee_hex = format!("{:064x}", native_fee);
    let lz_token_fee_hex = format!("{:064x}", 0u128);
    let refund_stripped = refund_address.strip_prefix("0x").unwrap_or(refund_address);
    let refund_hex = format!("{:0>64}", refund_stripped);

    // sendToken(SendParam _sendParam, MessagingFee _fee, address _refundAddress)
    // ABI layout:
    //   word 0: offset to _sendParam (dynamic tuple) from start of args
    //   word 1: nativeFee (part of MessagingFee — but MessagingFee is a struct too)
    //   Actually: MessagingFee is also a tuple. Both are passed as parameters.
    //   sendToken has 3 params: tuple(dynamic), tuple(static: 2 uint256), address(static)
    //   So layout:
    //     word 0: offset to SendParam
    //     word 1: nativeFee (MessagingFee.nativeFee) — wait, MessagingFee tuple is static (no dynamic fields)
    //     MessagingFee = (uint256, uint256) — fully static, so it's inlined
    //     word 1: MessagingFee.nativeFee
    //     word 2: MessagingFee.lzTokenFee
    //     word 3: refundAddress
    //     word 4+: SendParam data
    // Total head = 4 words = 0x80 bytes, so offset to SendParam = 0x80

    format!(
        "0xcbef2aa9{:064x}{}{}{}{}",
        0x80u64,         // offset to SendParam (from start of args = 4 words)
        native_fee_hex,  // MessagingFee.nativeFee
        lz_token_fee_hex,// MessagingFee.lzTokenFee
        refund_hex,      // refundAddress
        param_body       // SendParam data
    )
}

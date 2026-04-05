use anyhow::Context;
use clap::Args;
use serde::Deserialize;

use crate::rpc::build_client;

const LAYERZERO_SCAN_BASE: &str = "https://scan.layerzero-api.com/v1";

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Source chain transaction hash from sendToken
    #[arg(long)]
    pub tx_hash: Option<String>,

    /// Wallet address to query message history
    #[arg(long)]
    pub wallet: Option<String>,

    /// LayerZero GUID to look up a specific message
    #[arg(long)]
    pub guid: Option<String>,

    /// Number of history records to return (for --wallet query)
    #[arg(long, default_value = "10")]
    pub limit: u32,

    /// Override LayerZero Scan API base URL
    #[arg(long)]
    pub scan_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ScanResponse {
    // LayerZero Scan API v1 returns results in a "data" array
    data: Option<Vec<Message>>,
    // older/alternate field name
    messages: Option<Vec<Message>>,
}

#[derive(Debug, Deserialize)]
struct StatusObj {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StatusField {
    Str(String),
    Obj(StatusObj),
}

#[derive(Debug, Deserialize)]
struct Message {
    pathway: Option<Pathway>,
    source: Option<TxInfo>,
    destination: Option<TxInfo>,
    status: Option<StatusField>,
}

#[derive(Debug, Deserialize)]
struct Pathway {
    #[serde(rename = "srcEid")]
    src_eid: Option<u32>,
    #[serde(rename = "dstEid")]
    dst_eid: Option<u32>,
    sender: Option<AddrObj>,
    receiver: Option<AddrObj>,
}

#[derive(Debug, Deserialize)]
struct AddrObj {
    address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TxInfo {
    tx: Option<TxDetail>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TxDetail {
    #[serde(rename = "txHash")]
    tx_hash: Option<String>,
    #[serde(rename = "blockNumber")]
    block_number: Option<serde_json::Value>,
}

fn describe_status(status: &str) -> &'static str {
    match status {
        "INFLIGHT" => "Waiting for source chain confirmation...",
        "CONFIRMING" => "Received on destination chain, awaiting finality...",
        "DELIVERED" => "Cross-chain transfer complete! Funds delivered on destination.",
        "FAILED" => "Transaction failed.",
        "PAYLOAD_STORED" => "Arrived on destination but execution reverted (retryable).",
        "BLOCKED" => "Blocked by pending nonce — previous message not yet cleared.",
        _ => "Unknown status",
    }
}

pub async fn run(args: StatusArgs) -> anyhow::Result<()> {
    let base_url = args
        .scan_url
        .as_deref()
        .unwrap_or(LAYERZERO_SCAN_BASE);

    let client = build_client();

    if let Some(tx_hash) = &args.tx_hash {
        query_by_tx_hash(&client, base_url, tx_hash).await?;
    } else if let Some(wallet) = &args.wallet {
        query_by_wallet(&client, base_url, wallet, args.limit).await?;
    } else if let Some(guid) = &args.guid {
        query_by_guid(&client, base_url, guid).await?;
    } else {
        anyhow::bail!("Provide one of: --tx-hash, --wallet, --guid");
    }

    Ok(())
}

async fn query_by_tx_hash(
    client: &reqwest::Client,
    base_url: &str,
    tx_hash: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/messages/tx/{}", base_url, tx_hash);
    println!("Querying LayerZero Scan: {}", url);

    let http_resp = client
        .get(&url)
        .send()
        .await
        .context("LayerZero Scan API request failed")?;

    let body_text = http_resp
        .text()
        .await
        .context("LayerZero Scan API response read failed")?;

    if body_text.is_empty() {
        println!("(LayerZero Scan API returned empty response. Transaction may not yet be indexed.)");
        return Ok(());
    }

    // Trim potential whitespace/BOM
    let trimmed = body_text.trim_start_matches('\u{FEFF}').trim();
    if trimmed.is_empty() {
        println!("(LayerZero Scan API returned whitespace-only response.)");
        return Ok(());
    }

    let resp: serde_json::Value = serde_json::from_str(trimmed)
        .with_context(|| format!("LayerZero Scan API response parse failed (body len={}, first 100 chars: {:?})",
            trimmed.len(), &trimmed[..std::cmp::min(100, trimmed.len())]))?;

    let scan: ScanResponse = serde_json::from_value(resp.clone())
        .unwrap_or(ScanResponse { data: None, messages: None });

    let messages = scan.data.or(scan.messages).unwrap_or_default();
    if messages.is_empty() {
        println!("No messages found for tx hash: {}", tx_hash);
        println!("(Transaction may not yet be indexed. Try again in 30 seconds.)");
    } else {
        for (i, msg) in messages.iter().enumerate() {
            print_message(i, msg);
        }
    }
    Ok(())
}

async fn query_by_wallet(
    client: &reqwest::Client,
    base_url: &str,
    wallet: &str,
    limit: u32,
) -> anyhow::Result<()> {
    let url = format!("{}/messages/wallet/{}?limit={}", base_url, wallet, limit);
    println!("Querying LayerZero Scan for wallet: {}", wallet);

    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .context("LayerZero Scan API request failed")?
        .json()
        .await
        .context("LayerZero Scan API response parse failed")?;

    let scan: ScanResponse = serde_json::from_value(resp)
        .unwrap_or(ScanResponse { data: None, messages: None });

    let messages = scan.data.or(scan.messages).unwrap_or_default();
    if messages.is_empty() {
        println!("No cross-chain messages found for wallet: {}", wallet);
    } else {
        println!("Found {} message(s):", messages.len());
        for (i, msg) in messages.iter().enumerate() {
            print_message(i, msg);
        }
    }
    Ok(())
}

async fn query_by_guid(
    client: &reqwest::Client,
    base_url: &str,
    guid: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/messages/guid/{}", base_url, guid);
    println!("Querying LayerZero Scan by GUID: {}", guid);

    let resp: serde_json::Value = client
        .get(&url)
        .send()
        .await
        .context("LayerZero Scan API request failed")?
        .json()
        .await
        .context("LayerZero Scan API response parse failed")?;

    let scan: ScanResponse = serde_json::from_value(resp)
        .unwrap_or(ScanResponse { data: None, messages: None });

    let messages = scan.data.or(scan.messages).unwrap_or_default();
    if messages.is_empty() {
        println!("No message found for GUID: {}", guid);
    } else {
        for (i, msg) in messages.iter().enumerate() {
            print_message(i, msg);
        }
    }
    Ok(())
}

fn print_message(index: usize, msg: &Message) {
    println!();
    println!("--- Message #{} ---", index + 1);

    if let Some(pathway) = &msg.pathway {
        if let (Some(src_eid), Some(dst_eid)) = (pathway.src_eid, pathway.dst_eid) {
            println!("  Route     : EID {} -> EID {}", src_eid, dst_eid);
        }
        if let Some(sender) = &pathway.sender {
            if let Some(addr) = &sender.address {
                println!("  Sender    : {}", addr);
            }
        }
        if let Some(receiver) = &pathway.receiver {
            if let Some(addr) = &receiver.address {
                println!("  Receiver  : {}", addr);
            }
        }
    }

    let overall_status = match &msg.status {
        Some(StatusField::Str(s)) => s.as_str().to_string(),
        Some(StatusField::Obj(o)) => o.name.as_deref().unwrap_or("UNKNOWN").to_string(),
        None => "UNKNOWN".to_string(),
    };
    println!("  Status    : {} — {}", overall_status, describe_status(&overall_status));

    if let Some(src) = &msg.source {
        if let Some(tx) = &src.tx {
            if let Some(hash) = &tx.tx_hash {
                println!("  Src tx    : {}", hash);
            }
            if let Some(block) = &tx.block_number {
                println!("  Src block : {}", block);
            }
        }
        if let Some(s) = &src.status {
            println!("  Src status: {}", s);
        }
    }

    if let Some(dst) = &msg.destination {
        if let Some(tx) = &dst.tx {
            if let Some(hash) = &tx.tx_hash {
                println!("  Dst tx    : {}", hash);
            }
        }
        if let Some(s) = &dst.status {
            println!("  Dst status: {}", s);
        }
    }
}

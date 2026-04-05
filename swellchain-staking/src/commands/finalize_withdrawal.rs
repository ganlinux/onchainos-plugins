use clap::Args;
use serde_json::json;

use crate::{
    config::{ETHEREUM_CHAIN_ID, SWEXIT_PROXY},
    onchainos,
    rpc::{self, print_json},
};

#[derive(Args, Debug)]
pub struct FinalizeWithdrawalArgs {
    /// swEXIT NFT token ID to finalize
    #[arg(long)]
    pub token_id: u128,

    /// Override sender address (defaults to logged-in wallet)
    #[arg(long)]
    pub from: Option<String>,

    /// Simulate without broadcasting
    #[arg(long)]
    pub dry_run: bool,
}

/// finalize-withdrawal — finalize a matured swEXIT withdrawal request.
/// Selector: finalizeWithdrawal(uint256) = 0x5e15c749
pub async fn run(args: FinalizeWithdrawalArgs) -> anyhow::Result<()> {
    let calldata = build_finalize_calldata(args.token_id);

    if args.dry_run {
        print_json(&json!({
            "ok": true,
            "dry_run": true,
            "action": "finalize-withdrawal",
            "contract": SWEXIT_PROXY,
            "token_id": args.token_id.to_string(),
            "calldata": calldata,
            "description": "finalizeWithdrawal(uint256) — claim ETH from processed withdrawal NFT"
        }));
        return Ok(());
    }

    let wallet = onchainos::resolve_wallet(ETHEREUM_CHAIN_ID)?;
    if wallet.is_empty() {
        anyhow::bail!("Cannot resolve wallet address. Please pass --from or ensure onchainos is logged in.");
    }
    let from_addr = args.from.as_deref().unwrap_or(&wallet);

    // Check if the withdrawal is processed before attempting finalization
    let (is_processed, processed_rate) =
        rpc::get_processed_rate_for_token_id(SWEXIT_PROXY, args.token_id).await?;
    if !is_processed {
        anyhow::bail!(
            "Withdrawal request tokenId={} is not yet processed. Please wait and try again later.",
            args.token_id
        );
    }

    let result = onchainos::wallet_contract_call(
        ETHEREUM_CHAIN_ID,
        SWEXIT_PROXY,
        &calldata,
        Some(from_addr),
        None,
        false,
    )
    .await?;
    let tx_hash = onchainos::extract_tx_hash(&result);

    print_json(&json!({
        "ok": true,
        "action": "finalize-withdrawal",
        "contract": SWEXIT_PROXY,
        "token_id": args.token_id.to_string(),
        "processed_rate": processed_rate.to_string(),
        "txHash": tx_hash,
        "raw": result
    }));
    Ok(())
}

/// Build finalizeWithdrawal(uint256) calldata.
/// Selector: 0x5e15c749
fn build_finalize_calldata(token_id: u128) -> String {
    let token_id_hex = format!("{:064x}", token_id);
    format!("0x5e15c749{}", token_id_hex)
}

use clap::Args;

use crate::onchainos;

#[derive(Args, Debug)]
pub struct BalanceArgs {
    /// Override wallet address
    #[arg(long)]
    pub from: Option<String>,
}

pub async fn execute(_args: &BalanceArgs, chain_id: u64) -> anyhow::Result<()> {
    let result = onchainos::wallet_balance(chain_id)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

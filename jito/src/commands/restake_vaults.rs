use crate::api;
use anyhow::Result;

pub async fn run() -> Result<()> {
    println!("Fetching Jito Restaking Vaults...");
    println!("(Jito has 2000+ vault accounts; showing first 20 by program account order)");
    println!();

    match api::get_vault_accounts().await {
        Ok(vaults) if !vaults.is_empty() => {
            println!("Jito Restaking Vaults (sample of {}):", vaults.len());
            println!("==============================================");
            for (i, v) in vaults.iter().enumerate() {
                let pubkey = v["pubkey"].as_str().unwrap_or("unknown");
                println!("{}. {}", i + 1, pubkey);
            }
            println!();
            println!("For complete vault list with APY and capacity info, visit:");
            println!("  https://www.jito.network/restaking/vaults/");
            println!();
            println!("To deposit JitoSOL into a vault:");
            println!("  jito restake-deposit --vault <VAULT_ADDRESS> --amount <JITOSOL_AMOUNT>");
        }
        _ => {
            println!("No vaults found or RPC request timed out.");
            println!();
            println!("View all Jito Restaking Vaults at:");
            println!("  https://www.jito.network/restaking/vaults/");
            println!();
            println!("To deposit JitoSOL into a vault:");
            println!("  jito restake-deposit --vault <VAULT_ADDRESS> --amount <JITOSOL_AMOUNT>");
        }
    }

    Ok(())
}

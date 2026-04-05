use crate::onchainos;
use anyhow::{anyhow, Result};

const VAULT_PROGRAM: &str = "Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8";

pub async fn run(vault: &str, amount_jitosol: f64, dry_run: bool) -> Result<()> {
    if amount_jitosol <= 0.0 {
        return Err(anyhow!("Amount must be greater than 0"));
    }
    // Basic base58 length check
    if vault.len() < 32 || vault.len() > 44 {
        return Err(anyhow!("Invalid vault address: expected a base58 Solana public key (32-44 chars)"));
    }

    let wallet = onchainos::resolve_wallet_solana()?;
    println!("Wallet: {}", wallet);
    println!("Vault:  {}", vault);
    println!("Amount: {} JitoSOL", amount_jitosol);
    println!("Vault Program: {}", VAULT_PROGRAM);
    println!();

    if dry_run {
        println!("[dry-run] Would deposit {} JitoSOL into vault {}", amount_jitosol, vault);
        println!("[dry-run] Operation: Vault MintTo instruction");
        println!("[dry-run] Command would be:");
        println!("  onchainos wallet contract-call --chain 501 --to {} --unsigned-tx <BASE58_TX> --force",
            VAULT_PROGRAM);
        return Ok(());
    }

    // v1: Jito Vault SDK required to construct MintTo serialized tx.
    // Direct SDK integration is deferred to v2. Guide user to the web interface.
    println!("Note: Jito Restaking deposits require constructing a Vault MintTo instruction.");
    println!("This requires the Jito Vault SDK (Rust: jito-vault-sdk) to build the transaction.");
    println!();
    println!("To complete this deposit, use the Jito Restaking web interface:");
    println!("  https://www.jito.network/restaking/vaults/{}", vault);
    println!();
    println!("Advanced: once you have a base58-encoded serialized MintTo transaction,");
    println!("submit it via:");
    println!("  onchainos wallet contract-call --chain 501 --to {} --unsigned-tx <BASE58_TX> --force",
        VAULT_PROGRAM);
    Ok(())
}

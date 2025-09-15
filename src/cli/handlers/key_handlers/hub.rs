use crate::cli::types::HubKeyCommands;
use anyhow::Result;

/// Handle Hub key management commands
pub async fn handle_hub_key_command(command: HubKeyCommands) -> Result<()> {
    match command {
        HubKeyCommands::Import { fid } => {
            handle_hub_key_import(fid).await?;
        }
        HubKeyCommands::List => {
            handle_hub_key_list().await?;
        }
        HubKeyCommands::Delete { fid } => {
            handle_hub_key_delete(fid).await?;
        }
        HubKeyCommands::FromMnemonic { fid } => {
            handle_hub_key_from_mnemonic(fid).await?;
        }
    }
    Ok(())
}



async fn handle_hub_key_import(fid: u64) -> Result<()> {
    println!("📥 Importing ECDSA private key");
    println!("{}", "=".repeat(40));

    // Check if key already exists
    let eth_keys_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let mut eth_manager =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&eth_keys_file)?;

    let eth_exists = eth_manager.has_key(fid);

    if eth_exists {
        println!("⚠️  ECDSA key already exists for FID: {fid}");

        print!("\nDo you want to replace the existing key? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response != "y" && response != "yes" {
            println!("❌ Operation cancelled. Existing key will not be replaced.");
            return Ok(());
        }

        // Remove existing key
        eth_manager.remove_key(fid)?;
        println!("🗑️  Removed existing ECDSA key");
    }

    // Prompt for private key interactively
    let private_key = crate::encrypted_eth_key_manager::prompt_password(
        "Enter ECDSA private key (hex format): ",
    )?;

    // Prompt for password
    let password =
        crate::encrypted_eth_key_manager::prompt_password("Enter password for encryption: ")?;
    let confirm_password =
        crate::encrypted_eth_key_manager::prompt_password("Confirm password: ")?;

    if password != confirm_password {
        println!("❌ Passwords do not match!");
        return Ok(());
    }

    // Import ECDSA key (Custody Key)
    println!("\n🔐 Importing ECDSA key (Custody Key)...");

    match eth_manager
        .import_and_encrypt(fid, &private_key, &password)
        .await
    {
        Ok(_) => {
            if let Err(e) = eth_manager.save_to_file(&eth_keys_file) {
                println!("❌ Failed to save ECDSA keys: {e}");
                return Ok(());
            }
            let eth_address = eth_manager.get_address(fid)?;
            println!("✅ ECDSA key imported and encrypted successfully!");
            println!("   FID: {fid}");
            println!("   Address: {eth_address}");
            println!("   Type: Ethereum wallet (custody key)");
        }
        Err(e) => {
            println!("❌ Failed to import ECDSA key: {e}");
            return Ok(());
        }
    }

    Ok(())
}


async fn handle_hub_key_list() -> Result<()> {
    println!("📋 All ECDSA Keys");
    println!("{}", "=".repeat(50));

    let keys_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let manager = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(
        &keys_file,
    )?;

    let keys = manager.list_keys();
    if keys.is_empty() {
        println!("❌ No ECDSA keys found.");
        println!("💡 Use 'castorix hub key import <fid>' to import your first ECDSA key!");
        println!("   Or use 'castorix hub key from-mnemonic <fid>' to generate from recovery phrase");
    } else {
        println!("🔒 ECDSA keys found:");

        // Show detailed info with addresses (no password needed)
        match manager.list_keys_with_info("") {
            Ok(key_infos) => {
                for key_info in key_infos {
                    let created_at =
                        chrono::DateTime::from_timestamp(key_info.created_at as i64, 0)
                            .unwrap_or_default()
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string();

                    println!("\n🔑 FID: {}", key_info.fid);
                    println!("   Type: ECDSA (Custody Key)");
                    println!("   Address: {}", key_info.address);
                    println!("   Created: {created_at}");
                    println!("   Status: ✅ Address available");
                }
            }
            Err(e) => {
                println!("❌ Error loading ECDSA key information: {e}");
                // Fallback to basic info
                for (fid, address, created_at) in &keys {
                    let created_at_str = chrono::DateTime::from_timestamp(*created_at as i64, 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string();

                    println!("\n🔑 FID: {fid}");
                    println!("   Type: ECDSA (Custody Key)");
                    println!("   Address: {address}");
                    println!("   Created: {created_at_str}");
                    println!("   Status: 🔒 Encrypted");
                }
            }
        }

        // Summary
        println!("\n📊 Summary: {} ECDSA key(s) found", keys.len());
        println!("💡 Use 'castorix hub key delete <fid>' to remove a key");
    }

    Ok(())
}

async fn handle_hub_key_from_mnemonic(fid: u64) -> Result<()> {
    println!("🌱 Generating ECDSA key (Custody Key) from recovery phrase");
    println!("{}", "=".repeat(60));

    // Check if key already exists
    let eth_keys_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let mut eth_manager =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&eth_keys_file)?;

    let eth_exists = eth_manager.has_key(fid);

    if eth_exists {
        println!("⚠️  ECDSA key already exists for FID: {fid}");

        print!("\nDo you want to replace the existing key? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response != "y" && response != "yes" {
            println!("❌ Operation cancelled. Existing key will not be replaced.");
            return Ok(());
        }

        // Remove existing key
        eth_manager.remove_key(fid)?;
        println!("🗑️  Removed existing ECDSA key");
    }

    // Prompt for recovery phrase interactively
    let recovery_phrase =
        crate::encrypted_eth_key_manager::prompt_password("Enter recovery phrase (mnemonic): ")?;

    // Prompt for password
    let password =
        crate::encrypted_eth_key_manager::prompt_password("Enter password for encryption: ")?;
    let confirm_password = crate::encrypted_eth_key_manager::prompt_password("Confirm password: ")?;

    if password != confirm_password {
        println!("❌ Passwords do not match!");
        return Ok(());
    }

    // Generate ECDSA key (Custody Key)
    println!("\n🔐 Generating ECDSA key (Custody Key)...");

    match eth_manager
        .generate_from_recovery_phrase(fid, &recovery_phrase, &password)
        .await
    {
        Ok(_) => {
            if let Err(e) = eth_manager.save_to_file(&eth_keys_file) {
                println!("❌ Failed to save ECDSA keys: {e}");
                return Ok(());
            }
            let eth_address = eth_manager.get_address(fid)?;
            println!("✅ ECDSA key generated successfully!");
            println!("   FID: {fid}");
            println!("   Address: {eth_address}");
            println!("   Type: Ethereum wallet (custody key)");
        }
        Err(e) => {
            println!("❌ Failed to generate ECDSA key: {e}");
            return Ok(());
        }
    }

    println!("\n🎉 ECDSA key setup completed successfully!");
    println!("   Note: Ed25519 (Signer) key must be imported separately using 'hub key import'");

    Ok(())
}

async fn handle_hub_key_delete(fid: u64) -> Result<()> {
    println!("🗑️ Deleting ECDSA key for FID: {fid}");
    println!("{}", "=".repeat(40));

    let eth_keys_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let mut eth_manager =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&eth_keys_file)?;

    // Check if key exists
    if !eth_manager.has_key(fid) {
        println!("❌ No ECDSA key found for FID: {fid}");
        return Ok(());
    }

    // Get key info before deletion for confirmation
    let address = eth_manager.get_address(fid)?;
    println!("🔍 Found ECDSA key for FID {fid}:");
    println!("   Address: {address}");
    println!("   Type: Ethereum wallet (custody key)");

    // Confirm deletion
    print!("\n⚠️  Are you sure you want to delete this key? (y/N): ");
    use std::io::{self, Write};
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let response = input.trim().to_lowercase();

    if response != "y" && response != "yes" {
        println!("❌ Operation cancelled. Key will not be deleted.");
        return Ok(());
    }

    // Delete the key
    match eth_manager.remove_key(fid) {
        Ok(_) => {
            if let Err(e) = eth_manager.save_to_file(&eth_keys_file) {
                println!("❌ Failed to save changes: {e}");
                return Ok(());
            }
            println!("✅ ECDSA key deleted successfully!");
            println!("   FID: {fid}");
            println!("   Address: {address}");
            println!("   Storage: {eth_keys_file:?}");
            println!("⚠️  Note: This only removes the local encrypted key file.");
            println!("   The on-chain state remains unchanged.");
        }
        Err(e) => {
            println!("❌ Failed to delete key: {e}");
        }
    }

    Ok(())
}

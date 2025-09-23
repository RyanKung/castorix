use crate::cli::types::CustodyCommands;
use anyhow::Result;

/// Handle custody commands
pub async fn handle_custody_command(command: CustodyCommands) -> Result<()> {
    match command {
        CustodyCommands::List => {
            handle_custody_list().await?;
        }
        CustodyCommands::Import { fid } => {
            handle_custody_import(fid).await?;
        }
        CustodyCommands::FromMnemonic { fid } => {
            handle_custody_from_mnemonic(fid).await?;
        }
        CustodyCommands::Delete { fid } => {
            handle_custody_delete(fid).await?;
        }
    }
    Ok(())
}

async fn handle_custody_list() -> Result<()> {
    println!("📋 All Custody Keys (ECDSA)");
    println!("{}", "=".repeat(50));

    // List all custody key files in the custody directory
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let custody_dir = home_dir.join(".castorix").join("custody");

    if !custody_dir.exists() {
        println!("❌ No custody keys found.");
        println!("💡 Use 'castorix custody import <fid>' to import your first custody key!");
        println!("💡 Use 'castorix custody from-mnemonic <fid>' to generate from mnemonic!");
        return Ok(());
    }

    let mut custody_keys = Vec::new();

    // Scan for FID-specific custody key files
    if let Ok(entries) = std::fs::read_dir(&custody_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.starts_with("fid-") && file_name.ends_with("-custody.json") {
                    // Extract FID from filename
                    if let Some(fid_str) = file_name
                        .strip_prefix("fid-")
                        .and_then(|s| s.strip_suffix("-custody.json"))
                    {
                        if let Ok(fid) = fid_str.parse::<u64>() {
                            let file_path = entry.path().to_string_lossy().to_string();
                            if let Ok(manager) = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&file_path) {
                                if let Ok(address) = manager.get_address(fid) {
                                    custody_keys.push((fid, address, file_path));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if custody_keys.is_empty() {
        println!("❌ No custody keys found.");
        println!("💡 Use 'castorix custody import <fid>' to import your first custody key!");
        println!("💡 Use 'castorix custody from-mnemonic <fid>' to generate from mnemonic!");
    } else {
        println!("🔒 Custody keys found:");
        println!("\n{:<8} {:<42} {:<50}", "FID", "Address", "File");
        println!("{}", "-".repeat(100));
        for (fid, address, file_path) in custody_keys {
            println!("{:<8} {:<42} {:<50}", fid, address, file_path);
        }

        println!("\n💡 Use 'castorix custody delete <fid>' to remove a key");
        println!("💡 Use 'castorix custody import <fid>' to add a new key");
    }

    Ok(())
}

async fn handle_custody_import(fid: u64) -> Result<()> {
    println!("📥 Importing ECDSA private key for FID: {fid}");

    // Prompt for private key
    let private_key = crate::encrypted_key_manager::prompt_password(
        "Enter ECDSA private key (hex format, 64 characters): ",
    )?;

    // Prompt for password
    let password =
        crate::encrypted_key_manager::prompt_password("Enter password to encrypt the key: ")?;

    // Confirm password
    let password_confirm = crate::encrypted_key_manager::prompt_password("Confirm password: ")?;

    if password != password_confirm {
        return Err(anyhow::anyhow!(
            "❌ Passwords do not match. Please try again."
        ));
    }

    // Use FID-specific custody key file
    let custody_key_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::custody_key_file(fid)?;
    let mut encrypted_manager =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(
            &custody_key_file,
        )?;

    // Import and encrypt the key
    encrypted_manager
        .import_and_encrypt(fid, &private_key, &password)
        .await?;

    // Save to FID-specific file
    encrypted_manager.save_to_file(&custody_key_file)?;

    // Get the address from the encrypted manager
    let address = encrypted_manager.get_address(fid)?;

    println!("✅ ECDSA key imported and encrypted successfully!");
    println!("🔑 Address: {}", address);
    println!("📁 FID: {}", fid);
    println!("💾 Key stored securely with password protection");

    Ok(())
}

async fn handle_custody_from_mnemonic(fid: u64) -> Result<()> {
    println!("🔑 Generating ECDSA key from mnemonic for FID: {fid}");

    // Prompt for mnemonic
    let mnemonic = crate::encrypted_key_manager::prompt_password(
        "Enter BIP39 mnemonic phrase (12 or 24 words): ",
    )?;

    // Prompt for password
    let password =
        crate::encrypted_key_manager::prompt_password("Enter password to encrypt the key: ")?;

    // Confirm password
    let password_confirm = crate::encrypted_key_manager::prompt_password("Confirm password: ")?;

    if password != password_confirm {
        return Err(anyhow::anyhow!(
            "❌ Passwords do not match. Please try again."
        ));
    }

    // Use FID-specific custody key file
    let custody_key_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::custody_key_file(fid)?;
    let mut encrypted_manager =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(
            &custody_key_file,
        )?;

    // Generate from mnemonic and encrypt
    encrypted_manager
        .generate_from_recovery_phrase(fid, &mnemonic, &password)
        .await?;

    // Get the address from the encrypted manager
    let address = encrypted_manager.get_address(fid)?;

    // Verify that the generated address matches the FID's actual custody address
    println!("🔍 Verifying address matches FID custody address...");

    // Create Farcaster client to get custody address from Hub API
    let config = crate::consts::get_config();
    let hub_client =
        crate::core::client::hub_client::FarcasterClient::new(config.farcaster_hub_url.clone(), None);

    // Get custody address from Hub API
    let actual_custody_address = hub_client
        .get_custody_address(fid)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get custody address from Hub API: {}", e))?;

    if address.to_lowercase() != actual_custody_address.to_lowercase() {
        // Clean up the generated file since address doesn't match
        let _ = std::fs::remove_file(&custody_key_file);

        return Err(anyhow::anyhow!(
            "❌ Generated address {} does not match FID {} custody address {}\n\n\
            💡 The mnemonic you provided does not correspond to the custody wallet for this FID.\n\
            📝 FID {} custody address: {}\n\
            🔑 Generated address: {}\n\n\
            🔍 To manage this FID, you need the mnemonic for the correct custody wallet.\n\
            💡 If you have the correct mnemonic, try again with the right one.\n\
            💡 If you have the private key instead, use: castorix custody import {}",
            address,
            fid,
            actual_custody_address,
            fid,
            actual_custody_address,
            address,
            fid
        ));
    }

    // Save to FID-specific file
    encrypted_manager.save_to_file(&custody_key_file)?;

    println!("✅ ECDSA key generated from mnemonic and encrypted successfully!");
    println!("🔑 Address: {} ✓", address);
    println!("📁 FID: {}", fid);
    println!("💾 Key stored securely with password protection");
    println!("📂 Saved to: {}", custody_key_file);

    Ok(())
}

async fn handle_custody_delete(fid: u64) -> Result<()> {
    println!("🗑️  Deleting ECDSA key for FID: {fid}");

    // Check if FID-specific custody key file exists
    let custody_key_file =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::custody_key_file(fid)?;

    if !std::path::Path::new(&custody_key_file).exists() {
        return Err(anyhow::anyhow!("❌ No ECDSA key found for FID: {}", fid));
    }

    // Load the encrypted key manager
    let encrypted_manager =
        crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(
            &custody_key_file,
        )?;

    // Check if key exists
    if !encrypted_manager.has_key(fid) {
        return Err(anyhow::anyhow!("❌ No ECDSA key found for FID: {}", fid));
    }

    // Get key info before deletion
    let address = encrypted_manager.get_address(fid)?;

    println!("🔍 Key to delete:");
    println!("   FID: {}", fid);
    println!("   Address: {}", address);
    println!("   File: {}", custody_key_file);

    // Confirm deletion
    let confirm = crate::encrypted_eth_key_manager::prompt_password(
        "Are you sure you want to delete this key? Type 'DELETE' to confirm: ",
    )?;

    if confirm != "DELETE" {
        println!("❌ Deletion cancelled");
        return Ok(());
    }

    // Delete the file
    std::fs::remove_file(&custody_key_file)
        .map_err(|e| anyhow::anyhow!("Failed to delete custody key file: {}", e))?;

    println!("✅ ECDSA key deleted successfully!");
    println!("🗑️  FID {} key removed from local storage", fid);
    println!("📁 Deleted file: {}", custody_key_file);

    Ok(())
}

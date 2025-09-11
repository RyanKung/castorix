use anyhow::Result;
use crate::cli::types::HubKeyCommands;

/// Handle Hub Ed25519 key management commands
pub async fn handle_hub_key_command(command: HubKeyCommands) -> Result<()> {
    match command {
        HubKeyCommands::List => {
            handle_hub_key_list().await?;
        }
        HubKeyCommands::Generate { fid } => {
            handle_hub_key_generate(fid).await?;
        }
        HubKeyCommands::Import { fid } => {
            handle_hub_key_import(fid).await?;
        }
        HubKeyCommands::Delete { fid } => {
            handle_hub_key_delete(fid).await?;
        }
        HubKeyCommands::Info { fid } => {
            handle_hub_key_info(fid).await?;
        }
        HubKeyCommands::FromMnemonic { fid } => {
            handle_hub_key_from_mnemonic(fid).await?;
        }
    }
    Ok(())
}

async fn handle_hub_key_list() -> Result<()> {
    println!("📋 All Encrypted Keys");
    println!("{}", "=".repeat(50));
    
    // List Ed25519 keys
    println!("\n🔐 Ed25519 Keys (Signer Keys)");
    println!("{}", "-".repeat(30));
    
    let ed25519_keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
    let ed25519_manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&ed25519_keys_file)?;
    
    let ed25519_keys = ed25519_manager.list_keys();
    if ed25519_keys.is_empty() {
        println!("❌ No Ed25519 keys found.");
        println!("💡 Use 'castorix hub key generate <fid>' to create your first Ed25519 key!");
    } else {
        println!("🔒 Ed25519 keys found:");
        
        // Show detailed info with public keys (no password needed)
        match ed25519_manager.list_keys_with_info("") {
            Ok(key_infos) => {
                for key_info in key_infos {
                    let created_at = chrono::DateTime::from_timestamp(key_info.created_at as i64, 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string();
                    
                    println!("\n🔑 FID: {}", key_info.fid);
                    println!("   Type: Ed25519 (Signer Key)");
                    println!("   Public Key: {}", key_info.public_key);
                    println!("   Created: {}", created_at);
                    println!("   Status: ✅ Public key available");
                }
            }
            Err(e) => {
                println!("❌ Error loading Ed25519 key information: {}", e);
                // Fallback to basic info
                for (fid, created_at_str, _created_at) in &ed25519_keys {
                    println!("\n🔑 FID: {}", fid);
                    println!("   Type: Ed25519 (Signer Key)");
                    println!("   Created: {}", created_at_str);
                    println!("   Status: 🔒 Encrypted");
                }
            }
        }
    }
    
    // List Ethereum keys
    println!("\n🔐 Ethereum Keys (Custody Keys)");
    println!("{}", "-".repeat(30));
    
    let eth_keys_file = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let eth_manager = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&eth_keys_file)?;
    
    let eth_keys = eth_manager.list_keys();
    if eth_keys.is_empty() {
        println!("❌ No Ethereum keys found.");
        println!("💡 Use 'castorix hub key from-recovery <fid>' to create your first Ethereum key!");
    } else {
        println!("🔒 Ethereum keys found:");
        
        // Show detailed info with addresses (no password needed)
        match eth_manager.list_keys_with_info("") {
            Ok(key_infos) => {
                for key_info in key_infos {
                    let created_at = chrono::DateTime::from_timestamp(key_info.created_at as i64, 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string();
                    
                    println!("\n🔑 FID: {}", key_info.fid);
                    println!("   Type: Ethereum (Custody Key)");
                    println!("   Address: {}", key_info.address);
                    println!("   Created: {}", created_at);
                    println!("   Status: ✅ Address available");
                }
            }
            Err(e) => {
                println!("❌ Error loading Ethereum key information: {}", e);
                // Fallback to basic info
                for (fid, address, created_at) in &eth_keys {
                    let created_at_str = chrono::DateTime::from_timestamp(*created_at as i64, 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string();
                    
                    println!("\n🔑 FID: {}", fid);
                    println!("   Type: Ethereum (Custody Key)");
                    println!("   Address: {}", address);
                    println!("   Created: {}", created_at_str);
                    println!("   Status: 🔒 Encrypted");
                }
            }
        }
    }
    
    // Summary
    let total_keys = ed25519_keys.len() + eth_keys.len();
    if total_keys == 0 {
        println!("\n📊 Summary: No keys found");
        println!("💡 Create keys using:");
        println!("   - Ed25519: castorix hub key generate <fid>");
        println!("   - Ethereum: castorix hub key from-recovery <fid>");
    } else {
        println!("\n📊 Summary: {} total keys ({} Ed25519, {} Ethereum)", 
            total_keys, ed25519_keys.len(), eth_keys.len());
    }
    
    Ok(())
}

async fn handle_hub_key_generate(fid: u64) -> Result<()> {
    println!("🆕 Generating encrypted Ed25519 key pair");
    println!("{}", "=".repeat(40));
    
    // Prompt for password
    let password = crate::encrypted_ed25519_key_manager::prompt_password("Enter password for encryption: ")?;
    let confirm_password = crate::encrypted_ed25519_key_manager::prompt_password("Confirm password: ")?;
    
    if password != confirm_password {
        println!("❌ Passwords do not match!");
        return Ok(());
    }
    
    let keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
    let mut manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&keys_file)?;
    
    match manager.generate_and_encrypt(fid, &password).await {
        Ok(_) => {
            // Save to file
            if let Err(e) = manager.save_to_file(&keys_file) {
                println!("❌ Failed to save keys: {}", e);
                return Ok(());
            }
            
            let public_key = manager.get_verifying_key(fid, &password)?;
            println!("✅ Encrypted Ed25519 key pair generated successfully!");
            println!("   FID: {}", fid);
            println!("   Public Key: {}", hex::encode(public_key.to_bytes()));
            println!("   Storage: {:?}", keys_file);
        }
        Err(e) => println!("❌ Failed to generate key: {}", e),
    }
    
    Ok(())
}

async fn handle_hub_key_import(fid: u64) -> Result<()> {
    println!("📥 Importing Ed25519 private key");
    println!("{}", "=".repeat(40));
    
    // Prompt for private key interactively
    let private_key = crate::encrypted_ed25519_key_manager::prompt_password("Enter Ed25519 private key (hex or base58, 32 or 64 bytes): ")?;
    
    // Prompt for password
    let password = crate::encrypted_ed25519_key_manager::prompt_password("Enter password for encryption: ")?;
    let confirm_password = crate::encrypted_ed25519_key_manager::prompt_password("Confirm password: ")?;
    
    if password != confirm_password {
        println!("❌ Passwords do not match!");
        return Ok(());
    }
    
    let keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
    let mut manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&keys_file)?;
    
    match manager.import_and_encrypt(fid, &private_key, &password).await {
        Ok(_) => {
            // Save to file
            if let Err(e) = manager.save_to_file(&keys_file) {
                println!("❌ Failed to save keys: {}", e);
                return Ok(());
            }
            
            let public_key = manager.get_verifying_key(fid, &password)?;
            println!("✅ Ed25519 key imported and encrypted successfully!");
            println!("   FID: {}", fid);
            println!("   Public Key: {}", hex::encode(public_key.to_bytes()));
            println!("   Storage: {:?}", keys_file);
        }
        Err(e) => println!("❌ Failed to import key: {}", e),
    }
    
    Ok(())
}

async fn handle_hub_key_delete(fid: u64) -> Result<()> {
    println!("🗑️ Deleting keys for FID: {}", fid);
    println!("{}", "=".repeat(40));
    
    let mut deleted_any = false;
    
    // Delete Ed25519 key
    let ed25519_keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
    let mut ed25519_manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&ed25519_keys_file)?;
    
    match ed25519_manager.remove_key(fid) {
        Ok(_) => {
            if let Err(e) = ed25519_manager.save_to_file(&ed25519_keys_file) {
                println!("❌ Failed to save Ed25519 keys: {}", e);
            } else {
                println!("✅ Ed25519 key deleted successfully!");
                deleted_any = true;
            }
        }
        Err(_) => {
            // Key doesn't exist, that's fine
        }
    }
    
    // Delete Ethereum key
    let eth_keys_file = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let mut eth_manager = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&eth_keys_file)?;
    
    match eth_manager.remove_key(fid) {
        Ok(_) => {
            if let Err(e) = eth_manager.save_to_file(&eth_keys_file) {
                println!("❌ Failed to save Ethereum keys: {}", e);
            } else {
                println!("✅ Ethereum key deleted successfully!");
                deleted_any = true;
            }
        }
        Err(_) => {
            // Key doesn't exist, that's fine
        }
    }
    
    if !deleted_any {
        println!("❌ No keys found for FID: {}", fid);
    } else {
        println!("   FID: {}", fid);
    }
    
    Ok(())
}

async fn handle_hub_key_info(fid: u64) -> Result<()> {
    println!("🔍 Ed25519 Key Information");
    println!("{}", "=".repeat(40));
    
    // Prompt for password
    let password = crate::encrypted_ed25519_key_manager::prompt_password(&format!("Enter password for FID {}: ", fid))?;
    
    let keys_file = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::default_keys_file()?;
    let manager = crate::encrypted_ed25519_key_manager::EncryptedEd25519KeyManager::load_from_file(&keys_file)?;
    
    match manager.get_verifying_key(fid, &password) {
        Ok(public_key) => {
            println!("✅ Key found!");
            println!("   FID: {}", fid);
            println!("   Public Key: {}", hex::encode(public_key.to_bytes()));
            println!("   Storage: {:?}", keys_file);
        }
        Err(e) => println!("❌ Key not found or wrong password: {}", e),
    }
    
    Ok(())
}

async fn handle_hub_key_from_mnemonic(fid: u64) -> Result<()> {
    println!("🌱 Generating ECDSA key (Custody Key) from recovery phrase");
    println!("{}", "=".repeat(60));
    
    // Check if key already exists
    let eth_keys_file = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::default_keys_file()?;
    let mut eth_manager = crate::encrypted_eth_key_manager::EncryptedEthKeyManager::load_from_file(&eth_keys_file)?;
    
    let eth_exists = eth_manager.has_key(fid);
    
    if eth_exists {
        println!("⚠️  ECDSA key already exists for FID: {}", fid);
        
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
    let recovery_phrase = crate::encrypted_eth_key_manager::prompt_password("Enter recovery phrase (mnemonic): ")?;
    
    // Prompt for password
    let password = crate::encrypted_eth_key_manager::prompt_password("Enter password for encryption: ")?;
    let confirm_password = crate::encrypted_eth_key_manager::prompt_password("Confirm password: ")?;
    
    if password != confirm_password {
        println!("❌ Passwords do not match!");
        return Ok(());
    }
    
    // Generate ECDSA key (Custody Key)
    println!("\n🔐 Generating ECDSA key (Custody Key)...");
    
    match eth_manager.generate_from_recovery_phrase(fid, &recovery_phrase, &password).await {
        Ok(_) => {
            if let Err(e) = eth_manager.save_to_file(&eth_keys_file) {
                println!("❌ Failed to save ECDSA keys: {}", e);
                return Ok(());
            }
            let eth_address = eth_manager.get_address(fid)?;
            println!("✅ ECDSA key generated successfully!");
            println!("   FID: {}", fid);
            println!("   Address: {}", eth_address);
            println!("   Type: Ethereum wallet (custody key)");
        }
        Err(e) => {
            println!("❌ Failed to generate ECDSA key: {}", e);
            return Ok(());
        }
    }
    
    println!("\n🎉 ECDSA key setup completed successfully!");
    println!("   Note: Ed25519 (Signer) key must be imported separately using 'hub key import'");
    
    Ok(())
}

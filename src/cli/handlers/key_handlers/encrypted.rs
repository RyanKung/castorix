use anyhow::Result;

pub async fn handle_generate_encrypted(storage_path: Option<&str>) -> Result<()> {
    use std::io::Write;
    use std::io::{
        self,
    };

    use ethers::signers::Signer;

    use crate::encrypted_key_manager::prompt_password;
    use crate::encrypted_key_manager::EncryptedKeyManager;

    println!("🔐 Generate Encrypted Private Key");
    println!("{}", "=".repeat(40));

    // Get key name
    print!("Enter key name: ");
    io::stdout().flush()?;
    let mut key_name = String::new();
    io::stdin().read_line(&mut key_name)?;
    let key_name = key_name.trim().to_string();

    if key_name.is_empty() {
        println!("❌ Key name cannot be empty!");
        return Ok(());
    }

    // Generate private key and show address
    println!("\n🔑 Generating new private key...");
    let mut manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };
    let temp_private_key = manager.generate_private_key()?;
    let private_key_bytes = temp_private_key.to_bytes();
    let temp_wallet = ethers::signers::LocalWallet::from(temp_private_key);
    let address = format!("{:?}", temp_wallet.address());

    println!("✅ New private key generated!");
    println!("   Address: {address}");

    // Confirm if user wants to continue
    print!("\nDo you want to encrypt and save this key? (y/N): ");
    io::stdout().flush()?;
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if !confirm.trim().to_lowercase().starts_with('y') {
        println!("❌ Key generation cancelled.");
        return Ok(());
    }

    // Get password
    let password = prompt_password("Enter password for encryption: ")?;
    let confirm_password = prompt_password("Confirm password: ")?;

    if password != confirm_password {
        println!("❌ Passwords do not match!");
        return Ok(());
    }

    // Encrypt and save
    match manager
        .import_and_encrypt(
            &hex::encode(private_key_bytes),
            &password,
            &key_name,
            &key_name,
        )
        .await
    {
        Ok(_) => {
            println!("✅ Encrypted key saved successfully!");
            println!("   Key Name: {key_name}");
            println!("   Address: {address}");
            println!("   Storage: ~/.castorix/keys/{key_name}.json");
        }
        Err(e) => println!("❌ Failed to save encrypted key: {e}"),
    }

    Ok(())
}

pub async fn handle_load_key(key_name: String, storage_path: Option<&str>) -> Result<()> {
    use crate::encrypted_key_manager::prompt_password;
    use crate::encrypted_key_manager::EncryptedKeyManager;

    println!("🔓 Loading encrypted key: {key_name}");
    let mut manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };

    if !manager.key_exists(&key_name) {
        println!("❌ Key '{key_name}' not found!");
        return Ok(());
    }

    let password = prompt_password("Enter password: ")?;
    match manager.load_and_decrypt(&password, &key_name).await {
        Ok(_) => {
            println!("✅ Key loaded successfully!");
            println!("   Key Name: {key_name}");
            println!("   Address: {:?}", manager.address().unwrap());
        }
        Err(e) => println!("❌ Failed to load key: {e}"),
    }

    Ok(())
}

pub async fn handle_list_keys(storage_path: Option<&str>) -> Result<()> {
    use crate::encrypted_key_manager::EncryptedKeyManager;

    let manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };
    match manager.list_keys_with_info() {
        Ok(key_infos) => {
            if key_infos.is_empty() {
                println!("📝 No encrypted keys found.");
                println!("💡 Use 'castorix key generate-encrypted' to create your first key!");
            } else {
                println!("📝 Available encrypted keys:");
                println!("{:<20} {:<30} {:<42} Created", "Name", "Alias", "Address");
                println!("{}", "-".repeat(100));
                for info in key_infos {
                    let created_date = chrono::DateTime::from_timestamp(info.created_at as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!(
                        "{:<20} {:<30} {:<42} {}",
                        info.name, info.alias, info.address, created_date
                    );
                }
            }
        }
        Err(e) => println!("❌ Failed to list keys: {e}"),
    }

    Ok(())
}

pub async fn handle_delete_key(key_name: String, storage_path: Option<&str>) -> Result<()> {
    use std::fs;

    use crate::encrypted_key_manager::prompt_password;
    use crate::encrypted_key_manager::EncryptedKeyManager;

    println!("🗑️  Deleting encrypted key: {key_name}");
    let manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };

    if !manager.key_exists(&key_name) {
        println!("❌ Key '{key_name}' not found!");
        return Ok(());
    }

    let password = prompt_password("Enter password to confirm deletion: ")?;

    // Verify password by trying to load the key
    let mut temp_manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };
    match temp_manager.load_and_decrypt(&password, &key_name).await {
        Ok(_) => {
            // Password is correct, proceed with deletion
            let key_path = if let Some(path) = storage_path {
                format!("{}/{key_name}.json", path)
            } else {
                format!("~/.castorix/keys/{key_name}.json")
            };
            let expanded_path = shellexpand::tilde(&key_path).to_string();

            match fs::remove_file(&expanded_path) {
                Ok(_) => println!("✅ Key '{key_name}' deleted successfully!"),
                Err(e) => println!("❌ Failed to delete key: {e}"),
            }
        }
        Err(_) => {
            println!("❌ Wrong password! Key not deleted.");
        }
    }

    Ok(())
}

pub async fn handle_rename_key(
    old_name: String,
    new_name: String,
    storage_path: Option<&str>,
) -> Result<()> {
    use crate::encrypted_key_manager::prompt_password;
    use crate::encrypted_key_manager::EncryptedKeyManager;

    println!("🔄 Renaming encrypted key: {old_name} → {new_name}");
    let mut manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };

    if !manager.key_exists(&old_name) {
        println!("❌ Key '{old_name}' not found!");
        return Ok(());
    }

    let password = prompt_password("Enter password to confirm rename: ")?;
    match manager.rename_key(&old_name, &new_name, &password).await {
        Ok(_) => {
            println!("✅ Key renamed successfully!");
            println!("   Old name: {old_name}");
            println!("   New name: {new_name}");
        }
        Err(e) => println!("❌ Failed to rename key: {e}"),
    }

    Ok(())
}

pub async fn handle_update_alias(
    key_name: String,
    new_alias: String,
    storage_path: Option<&str>,
) -> Result<()> {
    use crate::encrypted_key_manager::prompt_password;
    use crate::encrypted_key_manager::EncryptedKeyManager;

    println!("🏷️  Updating alias for key: {key_name}");
    let mut manager = if let Some(path) = storage_path {
        EncryptedKeyManager::new(path)
    } else {
        EncryptedKeyManager::default_config()
    };

    if !manager.key_exists(&key_name) {
        println!("❌ Key '{key_name}' not found!");
        return Ok(());
    }

    let password = prompt_password("Enter password to confirm alias update: ")?;
    match manager.update_alias(&key_name, &new_alias, &password).await {
        Ok(_) => {
            println!("✅ Alias updated successfully!");
            println!("   Key: {key_name}");
            println!("   New alias: {new_alias}");
        }
        Err(e) => println!("❌ Failed to update alias: {e}"),
    }

    Ok(())
}

pub async fn handle_import_key(storage_path: Option<&str>) -> Result<()> {
    use std::io::Write;
    use std::io::{
        self,
    };
    use std::str::FromStr;

    use ethers::signers::Signer;

    use crate::encrypted_key_manager::prompt_password;
    use crate::encrypted_key_manager::EncryptedKeyManager;

    println!("📥 Import Private Key");
    println!("{}", "=".repeat(40));

    // Get key name
    print!("Enter key name: ");
    io::stdout().flush()?;
    let mut key_name = String::new();
    io::stdin().read_line(&mut key_name)?;
    let key_name = key_name.trim().to_string();

    if key_name.is_empty() {
        println!("❌ Key name cannot be empty!");
        return Ok(());
    }

    // Get private key
    let private_key = prompt_password("Enter private key (hex): ")?;

    // Validate private key and show address
    println!("\n🔍 Validating private key...");
    match ethers::signers::LocalWallet::from_str(&private_key) {
        Ok(wallet) => {
            let address = format!("{:?}", wallet.address());
            println!("✅ Private key is valid!");
            println!("   Address: {address}");

            // Confirm if user wants to continue
            print!("\nDo you want to encrypt and save this key? (y/N): ");
            io::stdout().flush()?;
            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

            if !confirm.trim().to_lowercase().starts_with('y') {
                println!("❌ Key import cancelled.");
                return Ok(());
            }

            // Get password
            let password = prompt_password("Enter password for encryption: ")?;
            let confirm_password = prompt_password("Confirm password: ")?;

            if password != confirm_password {
                println!("❌ Passwords do not match!");
                return Ok(());
            }

            // Encrypt and save
            let mut manager = if let Some(path) = storage_path {
                EncryptedKeyManager::new(path)
            } else {
                EncryptedKeyManager::default_config()
            };
            match manager
                .import_and_encrypt(&private_key, &password, &key_name, &key_name)
                .await
            {
                Ok(_) => {
                    println!("✅ Private key imported successfully!");
                    println!("   Key Name: {key_name}");
                    println!("   Address: {address}");
                    println!("   Storage: ~/.castorix/keys/{key_name}.json");
                }
                Err(e) => println!("❌ Failed to save encrypted key: {e}"),
            }
        }
        Err(e) => {
            println!("❌ Invalid private key: {e}");
            println!("💡 Please check your private key format (hex string)");
        }
    }

    Ok(())
}

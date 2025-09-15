use crate::cli::types::KeyCommands;
use anyhow::{Context, Result};

/// Handle key management commands (legacy)
pub async fn handle_key_command(
    command: KeyCommands,
    key_manager: &crate::key_manager::KeyManager,
) -> Result<()> {
    match command {
        KeyCommands::Info => {
            use crate::encrypted_key_manager::EncryptedKeyManager;

            println!("📋 Stored Encrypted Keys Information:");
            println!("{}", "=".repeat(50));

            let manager = EncryptedKeyManager::default_config();
            match manager.list_keys_with_info() {
                Ok(key_infos) => {
                    if key_infos.is_empty() {
                        println!("❌ No encrypted keys found.");
                        println!(
                            "💡 Use 'castorix key generate-encrypted' to create your first key!"
                        );
                    } else {
                        for info in key_infos {
                            println!("\n🔑 Key: {}", info.name);
                            println!("   Address: {}", info.address);
                            let created_date =
                                chrono::DateTime::from_timestamp(info.created_at as i64, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unknown".to_string());
                            println!("   Created: {created_date}");
                            println!("   Storage: ~/.castorix/keys/{}.json", info.name);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to list keys: {e}"),
            }
        }
        KeyCommands::Sign { message } => {
            println!("✍️  Signing message: {message}");
            match key_manager.sign_message(&message).await {
                Ok(signature) => {
                    println!("✅ Signature: {signature:?}");
                    println!("📋 Hex: {}", hex::encode(signature.to_vec()));
                }
                Err(e) => println!("❌ Failed to sign message: {e}"),
            }
        }
        KeyCommands::Verify { message, signature } => {
            println!("🔍 Verifying signature for message: {message}");
            let sig_bytes =
                hex::decode(&signature).with_context(|| "Failed to decode signature from hex")?;

            match ethers::types::Signature::try_from(sig_bytes.as_slice()) {
                Ok(sig) => match key_manager.verify_signature(&message, &sig).await {
                    Ok(valid) => {
                        if valid {
                            println!("✅ Signature is valid!");
                        } else {
                            println!("❌ Signature is invalid!");
                        }
                    }
                    Err(e) => println!("❌ Failed to verify signature: {e}"),
                },
                Err(e) => println!("❌ Invalid signature format: {e}"),
            }
        }
        KeyCommands::Generate => {
            use ethers::core::k256::ecdsa::SigningKey;
            use ethers::signers::Signer;
            use rand::rngs::OsRng;

            let signing_key = SigningKey::random(&mut OsRng);
            let private_key_bytes = signing_key.to_bytes();
            let wallet = ethers::signers::LocalWallet::from(signing_key);

            println!("🔐 Generated new private key:");
            println!("   Private Key: {}", hex::encode(private_key_bytes));
            println!("   Address: {:?}", wallet.address());
            println!("   ⚠️  Keep this private key secure and never share it!");
        }
        KeyCommands::GenerateEncrypted => {
            super::encrypted::handle_generate_encrypted().await?;
        }
        KeyCommands::Load { key_name } => {
            super::encrypted::handle_load_key(key_name).await?;
        }
        KeyCommands::List => {
            super::encrypted::handle_list_keys().await?;
        }
        KeyCommands::Delete { key_name } => {
            super::encrypted::handle_delete_key(key_name).await?;
        }
        KeyCommands::Rename { old_name, new_name } => {
            super::encrypted::handle_rename_key(old_name, new_name).await?;
        }
        KeyCommands::UpdateAlias {
            key_name,
            new_alias,
        } => {
            super::encrypted::handle_update_alias(key_name, new_alias).await?;
        }
        KeyCommands::Import => {
            super::encrypted::handle_import_key().await?;
        }
    }
    Ok(())
}

use crate::cli::types::HubCommands;
use anyhow::Result;

/// Handle Farcaster Hub commands
pub async fn handle_hub_command(
    command: HubCommands,
    hub_client: &crate::farcaster_client::FarcasterClient,
) -> Result<()> {
    match command {
        HubCommands::User { fid } => {
            println!("👤 Getting user information for FID: {fid}");
            match hub_client.get_user(fid).await {
                Ok(user_data) => {
                    println!("✅ User data retrieved:");
                    println!("{}", serde_json::to_string_pretty(&user_data)?);
                }
                Err(e) => println!("❌ Failed to get user data: {e}"),
            }
        }
        HubCommands::Cast {
            text: _,
            fid: _,
            parent_cast_id: _,
        } => {
            println!("❌ Cast submission not yet implemented with new protobuf structure");
            println!("💡 This feature will be re-implemented in a future update");
        }
        HubCommands::SubmitProof {
            proof_file,
            fid,
            wallet_name,
        } => {
            handle_submit_proof(hub_client, proof_file, fid, wallet_name).await?;
        }
        HubCommands::SubmitProofEip712 {
            proof_file,
            wallet_name,
        } => {
            handle_submit_proof_eip712(hub_client, proof_file, wallet_name).await?;
        }
        HubCommands::VerifyEth { fid: _, address: _ } => {
            println!("❌ Ethereum verification not yet implemented with new protobuf structure");
            println!("💡 This feature will be re-implemented in a future update");
        }
        HubCommands::EthAddresses { fid } => {
            println!("🔍 Getting Ethereum addresses for FID: {fid}");
            match hub_client.get_eth_addresses(fid).await {
                Ok(addresses) => {
                    if addresses.is_empty() {
                        println!("❌ No Ethereum addresses found for FID: {fid}");
                    } else {
                        println!("✅ Found {} Ethereum address(es):", addresses.len());
                        for (i, address) in addresses.iter().enumerate() {
                            println!("   {}. {}", i + 1, address);
                        }
                    }
                }
                Err(e) => println!("❌ Failed to get Ethereum addresses: {e}"),
            }
        }
        HubCommands::EnsDomains { fid } => {
            println!("🌐 Getting ENS domains with proofs for FID: {fid}");
            // Create a dummy EnsProof for the API call
            let dummy_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
            if let Ok(key_manager) = crate::key_manager::KeyManager::from_private_key(dummy_key) {
                let ens_proof = crate::ens_proof::EnsProof::new(
                    key_manager,
                    "https://eth-mainnet.g.alchemy.com/v2/dummy".to_string(),
                );
                match ens_proof
                    .get_ens_domains_by_fid(hub_client.hub_url(), fid)
                    .await
                {
                    Ok(domains) => {
                        if domains.is_empty() {
                            println!("❌ No ENS domains with proofs found for FID: {fid}");
                        } else {
                            println!("✅ Found {} ENS domain(s) with proofs:", domains.len());
                            for (i, domain) in domains.iter().enumerate() {
                                println!("   {}. {}", i + 1, domain);
                            }
                        }
                    }
                    Err(e) => println!("❌ Failed to get ENS domains: {e}"),
                }
            } else {
                println!("❌ Failed to create key manager for ENS query");
            }
        }
        HubCommands::CustodyAddress { fid } => {
            println!("🏠 Getting custody address for FID: {fid}");
            match hub_client.get_custody_address(fid).await {
                Ok(custody_address) => {
                    println!("✅ Custody address for FID {fid}:");
                    println!("   Address: {custody_address}");
                    println!("   Type: Ethereum address (custody key)");
                    println!("   Source: Farcaster Hub (onchain events)");
                }
                Err(e) => println!("❌ Failed to get custody address: {e}"),
            }
        }
        HubCommands::Info => {
            handle_hub_info(hub_client).await?;
        }
    }
    Ok(())
}

async fn handle_submit_proof(
    hub_client: &crate::farcaster_client::FarcasterClient,
    proof_file: String,
    fid: u64,
    wallet_name: Option<String>,
) -> Result<()> {
    println!("📤 Submitting username proof from file: {proof_file} for FID: {fid}");

    let proof_content = std::fs::read_to_string(&proof_file)?;
    let proof_data: serde_json::Value = serde_json::from_str(&proof_content)?;

    // Create UserNameProof from JSON
    let mut proof = crate::username_proof::UserNameProof::new();
    proof.set_timestamp(proof_data["timestamp"].as_u64().unwrap_or(0));
    proof.set_name(
        proof_data["name"]
            .as_str()
            .unwrap_or("")
            .as_bytes()
            .to_vec(),
    );
    proof.set_owner(hex::decode(proof_data["owner"].as_str().unwrap_or(""))?);
    proof.set_signature(hex::decode(proof_data["signature"].as_str().unwrap_or(""))?);
    proof.set_fid(proof_data["fid"].as_u64().unwrap_or(0));

    // Create a new FarcasterClient with the specified wallet if provided
    let client = if let Some(wallet_name) = wallet_name {
        // Load encrypted key manager and decrypt the key
        let mut encrypted_manager =
            crate::encrypted_key_manager::EncryptedKeyManager::default_config();

        // Prompt for password
        let password = crate::encrypted_key_manager::prompt_password(&format!(
            "Enter password for wallet '{wallet_name}': "
        ))?;

        // Load and decrypt the key
        encrypted_manager
            .load_and_decrypt(&password, &wallet_name)
            .await?;

        // Get the decrypted key manager
        let key_manager = encrypted_manager
            .key_manager()
            .ok_or_else(|| {
                anyhow::anyhow!("Failed to load key manager for wallet: {}", wallet_name)
            })?
            .clone();

        crate::farcaster_client::FarcasterClient::new(
            hub_client.hub_url().to_string(),
            Some(key_manager),
        )
    } else {
        crate::farcaster_client::FarcasterClient::new(
            hub_client.hub_url().to_string(),
            hub_client.key_manager().cloned(),
        )
    };

    // Submit using Ed25519 key for the specified FID
    let result = client.submit_username_proof_with_ed25519(&proof, fid).await;

    match result {
        Ok(response) => {
            println!("✅ Username proof submitted successfully!");
            println!("📋 Response: {response:?}");
        }
        Err(e) => println!("❌ Failed to submit username proof: {e}"),
    }

    Ok(())
}

async fn handle_submit_proof_eip712(
    hub_client: &crate::farcaster_client::FarcasterClient,
    proof_file: String,
    wallet_name: String,
) -> Result<()> {
    println!("📤 Submitting username proof with EIP-712 signature from file: {proof_file}");
    println!("🔑 Using wallet: {wallet_name}");

    let proof_content = std::fs::read_to_string(&proof_file)?;
    let proof_data: serde_json::Value = serde_json::from_str(&proof_content)?;

    // Create UserNameProof from JSON
    let mut proof = crate::username_proof::UserNameProof::new();
    proof.set_timestamp(proof_data["timestamp"].as_u64().unwrap_or(0));
    proof.set_name(
        proof_data["name"]
            .as_str()
            .unwrap_or("")
            .as_bytes()
            .to_vec(),
    );
    proof.set_owner(hex::decode(proof_data["owner"].as_str().unwrap_or(""))?);
    proof.set_signature(hex::decode(proof_data["signature"].as_str().unwrap_or(""))?);
    proof.set_fid(proof_data["fid"].as_u64().unwrap_or(0));

    // Load encrypted key manager and decrypt the key
    let mut encrypted_manager = crate::encrypted_key_manager::EncryptedKeyManager::default_config();

    // Prompt for password
    let password = crate::encrypted_key_manager::prompt_password(&format!(
        "Enter password for wallet '{wallet_name}': "
    ))?;

    // Load and decrypt the key
    encrypted_manager
        .load_and_decrypt(&password, &wallet_name)
        .await?;

    // Get the decrypted key manager
    let key_manager = encrypted_manager
        .key_manager()
        .ok_or_else(|| anyhow::anyhow!("Failed to load key manager for wallet: {}", wallet_name))?
        .clone();

    // Create FarcasterClient with the specified wallet
    let client = crate::farcaster_client::FarcasterClient::new(
        hub_client.hub_url().to_string(),
        Some(key_manager),
    );

    // Submit using EIP-712 signature
    let result = client.submit_username_proof_with_eip712(&proof).await;

    match result {
        Ok(response) => {
            println!("✅ Username proof submitted successfully with EIP-712 signature!");
            println!("📋 Response: {response:?}");
        }
        Err(e) => println!("❌ Failed to submit username proof: {e}"),
    }

    Ok(())
}

async fn handle_hub_info(hub_client: &crate::farcaster_client::FarcasterClient) -> Result<()> {
    println!("📊 Getting Hub information and sync status...");
    
    // Get Hub info from the API
    match hub_client.get_hub_info().await {
        Ok(hub_info) => {
            println!("✅ Hub information retrieved:");
            println!("{}", serde_json::to_string_pretty(&hub_info)?);
        }
        Err(e) => {
            println!("❌ Failed to get Hub information: {e}");
            println!("💡 This might be because the Hub doesn't support the info endpoint");
        }
    }
    
    Ok(())
}


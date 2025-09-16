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
        HubCommands::Followers { fid, limit } => {
            handle_followers(hub_client, fid, limit).await?;
        }
            HubCommands::Following { fid, limit } => {
                handle_following(hub_client, fid, limit).await?;
            }
            HubCommands::Profile { fid, all } => {
                handle_profile(hub_client, fid, all).await?;
            }
            HubCommands::Stats { fid } => {
                handle_stats(hub_client, fid).await?;
            }
            HubCommands::Spam { fids } => {
                handle_spam_check(fids).await?;
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

async fn handle_followers(
    hub_client: &crate::farcaster_client::FarcasterClient,
    fid: u64,
    limit: u32,
) -> Result<()> {
    let limit_text = if limit == 0 { "all".to_string() } else { limit.to_string() };
    println!("👥 Getting followers for FID: {fid} (limit: {limit_text})");
    
    match hub_client.get_followers(fid, limit).await {
        Ok(followers) => {
            if followers.is_empty() {
                println!("❌ No followers found for FID: {fid}");
            } else {
                println!("✅ Found {} follower(s):", followers.len());
                for (i, follower) in followers.iter().enumerate() {
                    // Extract FID from the link message
                    let follower_fid = follower
                        .get("data")
                        .and_then(|d| d.get("fid"))
                        .and_then(|f| f.as_u64())
                        .unwrap_or(0);
                    
                    // Extract timestamp for when they followed
                    let timestamp = follower
                        .get("data")
                        .and_then(|d| d.get("timestamp"))
                        .and_then(|t| t.as_u64())
                        .unwrap_or(0);
                    
                    println!("   {}. FID: {} (followed at timestamp: {})", 
                        i + 1, follower_fid, timestamp);
                }
            }
        }
        Err(e) => println!("❌ Failed to get followers: {e}"),
    }
    
    Ok(())
}

async fn handle_following(
    hub_client: &crate::farcaster_client::FarcasterClient,
    fid: u64,
    limit: u32,
) -> Result<()> {
    let limit_text = if limit == 0 { "all".to_string() } else { limit.to_string() };
    println!("👤 Getting following for FID: {fid} (limit: {limit_text})");
    
    match hub_client.get_following(fid, limit).await {
        Ok(following) => {
            if following.is_empty() {
                println!("❌ No following found for FID: {fid}");
            } else {
                println!("✅ Found {} following:", following.len());
                for (i, user) in following.iter().enumerate() {
                    // Extract target FID from the link message
                    let target_fid = user
                        .get("data")
                        .and_then(|d| d.get("linkBody"))
                        .and_then(|lb| lb.get("targetFid"))
                        .and_then(|f| f.as_u64())
                        .unwrap_or(0);
                    
                    // Extract timestamp for when they followed
                    let timestamp = user
                        .get("data")
                        .and_then(|d| d.get("timestamp"))
                        .and_then(|t| t.as_u64())
                        .unwrap_or(0);
                    
                    println!("   {}. FID: {} (followed at timestamp: {})", 
                        i + 1, target_fid, timestamp);
                }
            }
        }
        Err(e) => println!("❌ Failed to get following: {e}"),
    }
    
    Ok(())
}

async fn handle_profile(
    hub_client: &crate::farcaster_client::FarcasterClient,
    fid: u64,
    show_all: bool,
) -> Result<()> {
    println!("👤 Getting profile for FID: {fid}");
    
    match hub_client.get_user_profile(fid).await {
        Ok(profile_data) => {
            if profile_data.is_empty() {
                println!("❌ No profile data found for FID: {fid}");
            } else {
                println!("✅ Profile for FID: {fid}");
                println!("{}", "─".repeat(50));
                
                // Parse and display profile information
                let mut username = "Unknown".to_string();
                let mut display_name = "Unknown".to_string();
                let mut bio = "No bio".to_string();
                let mut pfp_url = "No profile picture".to_string();
                let mut location = "No location".to_string();
                let mut twitter = "No Twitter".to_string();
                let mut github = "No GitHub".to_string();
                let mut url = "No website".to_string();
                let mut eth_address = "No Ethereum address".to_string();
                let mut sol_address = "No Solana address".to_string();
                
                for data in &profile_data {
                    if let Some(user_data_body) = data.get("data").and_then(|d| d.get("userDataBody")) {
                        if let Some(data_type) = user_data_body.get("type").and_then(|t| t.as_str()) {
                            if let Some(value) = user_data_body.get("value").and_then(|v| v.as_str()) {
                                match data_type {
                                    "USER_DATA_TYPE_USERNAME" => username = value.to_string(),
                                    "USER_DATA_TYPE_DISPLAY" => display_name = value.to_string(),
                                    "USER_DATA_TYPE_BIO" => bio = value.to_string(),
                                    "USER_DATA_TYPE_PFP" => pfp_url = value.to_string(),
                                    "USER_DATA_TYPE_LOCATION" => location = value.to_string(),
                                    "USER_DATA_TYPE_TWITTER" => twitter = format!("@{}", value),
                                    "USER_DATA_TYPE_GITHUB" => github = format!("@{}", value),
                                    "USER_DATA_TYPE_URL" => url = value.to_string(),
                                    "USER_DATA_PRIMARY_ADDRESS_ETHEREUM" => eth_address = value.to_string(),
                                    "USER_DATA_PRIMARY_ADDRESS_SOLANA" => sol_address = value.to_string(),
                                    _ => {} // Ignore other types
                                }
                            }
                        }
                    }
                }
                
                if show_all {
                    // Display all profile information
                    println!("📝 Display Name: {}", display_name);
                    println!("👤 Username: @{}", username);
                    println!("📄 Bio: {}", bio);
                    println!("📍 Location: {}", location);
                    println!("🐦 Twitter: {}", twitter);
                    println!("💻 GitHub: {}", github);
                    println!("🌐 Website: {}", url);
                    println!("🔗 Ethereum: {}", eth_address);
                    println!("🔗 Solana: {}", sol_address);
                    println!("🖼️  Profile Picture: {}", pfp_url);
                    
                    // Display profile picture if available
                    if pfp_url != "No profile picture" && !pfp_url.is_empty() {
                        if let Err(e) = crate::image_display::ImageDisplay::smart_display(&pfp_url).await {
                            println!("❌ Failed to display profile picture: {}", e);
                        }
                    }
                    
                    println!("{}", "─".repeat(50));
                    println!("📊 Total profile fields: {}", profile_data.len());
                } else {
                    // Display only basic information
                    println!("👤 @{}", username);
                    println!("📝 {}", display_name);
                    println!("📄 {}", bio);
                    
                    // Display profile picture if available
                    if pfp_url != "No profile picture" && !pfp_url.is_empty() {
                        if let Err(e) = crate::image_display::ImageDisplay::smart_display(&pfp_url).await {
                            println!("❌ Failed to display profile picture: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => println!("❌ Failed to get profile: {e}"),
    }
    
    Ok(())
}

async fn handle_stats(
    hub_client: &crate::farcaster_client::FarcasterClient,
    fid: u64,
) -> Result<()> {
    println!("📊 Getting statistics for FID: {fid}");
    
    // Get storage limits which includes following count
    match hub_client.get_storage_limits(fid).await {
        Ok(storage_data) => {
            println!("✅ Storage limits retrieved:");
            
            if let Some(limits) = storage_data.get("limits").and_then(|l| l.as_array()) {
                for limit in limits {
                    if let (Some(store_type), Some(name), Some(limit_val), Some(used)) = (
                        limit.get("storeType").and_then(|s| s.as_str()),
                        limit.get("name").and_then(|n| n.as_str()),
                        limit.get("limit").and_then(|l| l.as_u64()),
                        limit.get("used").and_then(|u| u.as_u64()),
                    ) {
                        match name {
                            "LINKS" => {
                                println!("   👥 Following: {}/{} ({}%)", 
                                    used, limit_val, 
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                            "CASTS" => {
                                println!("   📝 Casts: {}/{} ({}%)", 
                                    used, limit_val,
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                            "REACTIONS" => {
                                println!("   ❤️  Reactions: {}/{} ({}%)", 
                                    used, limit_val,
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                            "USER_DATA" => {
                                println!("   👤 Profile Data: {}/{} ({}%)", 
                                    used, limit_val,
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                            "VERIFICATIONS" => {
                                println!("   ✅ Verifications: {}/{} ({}%)", 
                                    used, limit_val,
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                            "USERNAME_PROOFS" => {
                                println!("   🏷️  Username Proofs: {}/{} ({}%)", 
                                    used, limit_val,
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                            _ => {
                                println!("   {} {}: {}/{} ({}%)", 
                                    store_type, name, used, limit_val,
                                    if limit_val > 0 { (used * 100) / limit_val } else { 0 }
                                );
                            }
                        }
                    }
                }
            }
            
            
            // Show tier information if available
            if let Some(tier_subscriptions) = storage_data.get("tier_subscriptions").and_then(|t| t.as_array()) {
                if !tier_subscriptions.is_empty() {
                    println!("\n💎 Tier Information:");
                    for tier in tier_subscriptions {
                        if let (Some(tier_type), Some(expires_at)) = (
                            tier.get("tier_type").and_then(|t| t.as_str()),
                            tier.get("expires_at").and_then(|e| e.as_u64()),
                        ) {
                            if expires_at > 0 {
                                let expire_date = chrono::DateTime::from_timestamp(expires_at as i64, 0)
                                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|| "Unknown".to_string());
                                println!("   {} (expires: {})", tier_type, expire_date);
                            } else {
                                println!("   {} (permanent)", tier_type);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => println!("❌ Failed to get storage limits: {e}"),
    }
    
    Ok(())
}

async fn handle_spam_check(fids: Vec<u64>) -> Result<()> {
    println!("🚫 Checking spam status for FIDs: {:?}", fids);
    
    // Load spam checker
    let spam_checker = match crate::spam_checker::SpamChecker::load_from_file("labels/labels/spam.jsonl") {
        Ok(checker) => checker,
        Err(e) => {
            println!("❌ Failed to load spam labels: {e}");
            println!("💡 Make sure the labels submodule is properly initialized");
            return Ok(());
        }
    };
    
    // Get statistics
    let (total, spam_count, non_spam_count) = spam_checker.get_stats();
    println!("📊 Spam labels loaded: {} total, {} spam, {} non-spam", total, spam_count, non_spam_count);
    
    // Check each FID
    for fid in fids {
        match spam_checker.get_label(fid) {
            Some(label) => {
                let status = match label.label_value {
                    0 => "🚫 SPAM",
                    2 => "✅ CLEAN",
                    _ => "❓ UNKNOWN",
                };
                println!("   FID {}: {} (label_value: {})", fid, status, label.label_value);
            }
            None => {
                println!("   FID {}: ❓ NOT FOUND (not in dataset)", fid);
            }
        }
    }
    
    Ok(())
}


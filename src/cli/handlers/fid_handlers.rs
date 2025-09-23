use crate::cli::types::FidCommands;
use crate::farcaster::contracts::{
    contract_client::FarcasterContractClient,
    types::{ContractAddresses, ContractResult},
};
use anyhow::{Context, Result};
use ethers::{
    middleware::Middleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
    utils::format_ether,
};

/// Handle FID registration and management commands
pub async fn handle_fid_command(command: FidCommands) -> Result<()> {
    match command {
        FidCommands::Register {
            wallet,
            extra_storage,
            recovery,
            dry_run,
            yes,
        } => {
            handle_fid_register(wallet, extra_storage, recovery, dry_run, yes).await?;
        }
        FidCommands::Price { extra_storage } => {
            handle_fid_price(extra_storage).await?;
        }
        FidCommands::List { wallet } => {
            handle_fid_list(wallet).await?;
        }
    }
    Ok(())
}

async fn handle_fid_register(
    wallet_name: Option<String>,
    extra_storage: u64,
    recovery: Option<String>,
    dry_run: bool,
    yes: bool,
) -> Result<()> {
    println!("🆕 Register New FID");
    println!("{}", "=".repeat(40));

    // Get RPC URL from configuration (Farcaster contracts are on Optimism)
    let config = crate::consts::get_config();
    let rpc_url = config.eth_op_rpc_url().to_string();
    
    // Check if using placeholder values
    if rpc_url.contains("your_api_key_here") || rpc_url == "https://www.optimism.io/" {
        println!("⚠️  Configuration Warning:");
        println!("   ETH_OP_RPC_URL contains placeholder value: {}", rpc_url);
        println!("   Please set up your configuration:");
        println!("   1. Copy .env.example to .env: cp .env.example .env");
        println!("   2. Edit .env and set ETH_OP_RPC_URL to a valid Optimism RPC endpoint");
        println!("   3. Or set ETH_OP_RPC_URL environment variable");
        println!("   4. For example: export ETH_OP_RPC_URL=https://optimism-mainnet.g.alchemy.com/v2/your_api_key");
        return Ok(());
    }

    // Load wallet and get private key
    let private_key = if let Some(name) = wallet_name {
        // Load from encrypted storage
        use crate::encrypted_key_manager::{prompt_password, EncryptedKeyManager};
        
        let mut manager = EncryptedKeyManager::default_config();
        if !manager.key_exists(&name) {
            println!("❌ Wallet '{name}' not found!");
            println!("💡 Use 'castorix key list' to see available wallets");
            return Ok(());
        }

        let password = prompt_password(&format!("Enter password for wallet '{name}': "))?;
        match manager.load_and_decrypt(&password, &name).await {
            Ok(_) => {
                let wallet_address = manager.address().unwrap();
                println!("✅ Wallet loaded: {wallet_address}");
                manager.key_manager().unwrap().wallet().signer().to_bytes().to_vec()
            }
            Err(e) => {
                println!("❌ Failed to load wallet: {e}");
                return Ok(());
            }
        }
    } else {
        println!("❌ No wallet specified!");
        println!("💡 Please use 'castorix fid register --wallet <wallet-name>'");
        return Ok(());
    };

    // Create wallet from private key bytes
    let wallet = LocalWallet::from_bytes(&private_key)?;
    println!("   Wallet Address: {}", wallet.address());

    // Get recovery address
    let recovery_address = if let Some(recovery_addr) = recovery {
        recovery_addr.parse::<Address>()
            .with_context(|| "Invalid recovery address format")?
    } else {
        // Default to same as registration wallet
        wallet.address()
    };

    println!("\n📋 Registration Details:");
    println!("   Recovery Address: {recovery_address}");
    println!("   Extra Storage Units: {extra_storage}");

    // Create contract client
    println!("\n🔧 Setting up contract client...");
    let contract_client = FarcasterContractClient::new_with_wallet(
        rpc_url.clone(),
        ContractAddresses::default(),
        wallet.clone(),
    )?;

    // Get registration price
    println!("💰 Getting registration price...");
    let price = contract_client.get_registration_price().await?;
    println!("   Base Registration Price: {} ETH", format_ether(price));

    if extra_storage > 0 {
        let storage_price = contract_client.get_storage_price(extra_storage).await?;
        println!("   Extra Storage Price ({extra_storage} units): {} ETH", format_ether(storage_price));
        let total_price = price + storage_price;
        println!("   Total Price: {} ETH", format_ether(total_price));
    }

    // Check wallet balance
    let provider = Provider::<Http>::try_from(&rpc_url)?;
    let balance = provider.get_balance(wallet.address(), None).await?;
    println!("   Wallet Balance: {} ETH", format_ether(balance));

    if dry_run {
        println!("\n🔍 DRY RUN MODE - No transaction will be sent");
        println!("✅ Registration simulation completed successfully");
        return Ok(());
    }

    // ⚠️  IMPORTANT: This will trigger on-chain operations
    println!("\n⚠️  ON-CHAIN OPERATION WARNING:");
    println!("   • This will register a new FID on the Farcaster network");
    println!("   • The operation will consume gas fees and registration cost");
    println!("   • This action cannot be undone");
    println!("   • Make sure you have sufficient ETH for gas and registration");

    // Ask for user confirmation (skip if --yes is provided)
    if !yes {
        print!("\n❓ Do you want to proceed with FID registration? (yes/no): ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut confirmation = String::new();
        io::stdin().read_line(&mut confirmation)?;
        let confirmation = confirmation.trim().to_lowercase();

        if confirmation != "yes" && confirmation != "y" {
            println!("❌ Operation cancelled by user");
            return Ok(());
        }
    } else {
        println!("\n✅ Auto-confirmed with --yes flag");
    }

    println!("✅ Proceeding with FID registration...");

    // Register FID
    let result = if extra_storage > 0 {
        println!("🚀 Registering FID with {extra_storage} extra storage units...");
        contract_client.register_fid_with_storage(recovery_address, extra_storage).await?
    } else {
        println!("🚀 Registering FID...");
        contract_client.register_fid(recovery_address).await?
    };

    match result {
        ContractResult::Success((fid, overpayment)) => {
            println!("✅ FID registration successful!");
            println!("   FID: {}", fid);
            if !overpayment.is_zero() {
                println!("   Overpayment: {} ETH", format_ether(overpayment));
            }
        }
        ContractResult::Error(e) => {
            println!("❌ FID registration failed: {}", e);
            return Err(anyhow::anyhow!("FID registration failed: {}", e));
        }
    }
    
    Ok(())
}

async fn handle_fid_price(extra_storage: u64) -> Result<()> {
    println!("💰 FID Registration Price");
    println!("{}", "=".repeat(40));

    // Get RPC URL from configuration (Farcaster contracts are on Optimism)
    let config = crate::consts::get_config();
    let rpc_url = config.eth_op_rpc_url().to_string();
    
    // Check if using placeholder values
    if rpc_url.contains("your_api_key_here") || rpc_url == "https://www.optimism.io/" {
        println!("⚠️  Configuration Warning:");
        println!("   ETH_OP_RPC_URL contains placeholder value: {}", rpc_url);
        println!("   Please set up your configuration:");
        println!("   1. Copy .env.example to .env: cp .env.example .env");
        println!("   2. Edit .env and set ETH_OP_RPC_URL to a valid Optimism RPC endpoint");
        println!("   3. Or set ETH_OP_RPC_URL environment variable");
        println!("   4. For example: export ETH_OP_RPC_URL=https://optimism-mainnet.g.alchemy.com/v2/your_api_key");
        return Ok(());
    }

    // Create contract client (read-only)
    let contract_client = FarcasterContractClient::new(rpc_url, ContractAddresses::default())?;

    // Get registration price
    println!("🔍 Querying current registration prices...");
    let base_price = contract_client.get_registration_price().await?;
    println!("   Base Registration Price: {} ETH", format_ether(base_price));

    let mut total_price = base_price;
    
    if extra_storage > 0 {
        let storage_price = contract_client.get_storage_price(extra_storage).await?;
        println!("   Extra Storage Price ({extra_storage} units): {} ETH", format_ether(storage_price));
        total_price += storage_price;
    }

    println!("\n📊 Price Summary:");
    println!("   Base Registration: {} ETH", format_ether(base_price));
    if extra_storage > 0 {
        println!("   Extra Storage ({extra_storage} units): {} ETH", format_ether(total_price - base_price));
    }
    println!("   Total Registration Cost: {} ETH", format_ether(total_price));
    println!("   Estimated Gas Fees: ~0.002-0.005 ETH (varies with network)");
    
    Ok(())
}

async fn handle_fid_list(wallet_name: Option<String>) -> Result<()> {
    println!("📋 FIDs Owned by Wallet");
    println!("{}", "=".repeat(40));

    // Get RPC URL from configuration
    let rpc_url = crate::consts::get_config().eth_rpc_url().to_string();

    // Get wallet address
    let wallet_address = if let Some(name) = wallet_name {
        // Load from encrypted storage
        use crate::encrypted_key_manager::{prompt_password, EncryptedKeyManager};
        
        let mut manager = EncryptedKeyManager::default_config();
        if !manager.key_exists(&name) {
            println!("❌ Wallet '{name}' not found!");
            println!("💡 Use 'castorix key list' to see available wallets");
            return Ok(());
        }

        let password = prompt_password(&format!("Enter password for wallet '{name}': "))?;
        match manager.load_and_decrypt(&password, &name).await {
            Ok(_) => {
                let address = manager.address().unwrap();
                println!("✅ Wallet loaded: {address}");
                address
            }
            Err(e) => {
                println!("❌ Failed to load wallet: {e}");
                return Ok(());
            }
        }
    } else {
        println!("❌ No wallet specified!");
        println!("💡 Please use 'castorix fid list --wallet <wallet-name>'");
        return Ok(());
    };

    println!("   Wallet Address: {wallet_address}");

    // Create contract client (read-only)
    let contract_client = FarcasterContractClient::new(rpc_url, ContractAddresses::default())?;

    // Query FID for this address
    println!("\n🔍 Querying FID for wallet address...");
    match contract_client.id_registry.id_of(wallet_address).await? {
        ContractResult::Success(fid) => {
            if fid > 0 {
                println!("✅ Found FID: {}", fid);
                
                // Get additional FID information
                let fid_info = contract_client.get_fid_info(fid.into()).await?;
                println!("\n📋 FID Information:");
                println!("   FID: {}", fid);
                println!("   Custody Address: {}", fid_info.custody);
                println!("   Recovery Address: {}", fid_info.recovery);
                // Note: registration_time is not available in FidInfo struct
            } else {
                println!("ℹ️  No FID found for this wallet address");
                println!("💡 This wallet doesn't own any Farcaster ID");
            }
        }
        ContractResult::Error(e) => {
            println!("❌ Failed to query FID: {}", e);
            return Err(anyhow::anyhow!("Failed to query FID: {}", e));
        }
    }
    
    Ok(())
}

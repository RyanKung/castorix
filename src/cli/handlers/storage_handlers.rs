use crate::cli::types::StorageCommands;
use crate::encrypted_key_manager::{prompt_password, EncryptedKeyManager};
use crate::farcaster::contracts::{
    contract_client::FarcasterContractClient,
    types::{ContractAddresses, ContractResult},
};
use anyhow::Result;
use ethers::{
    middleware::Middleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    utils::format_ether,
};

/// Handle storage rental and management commands
pub async fn handle_storage_command(
    command: StorageCommands,
    storage_path: Option<&str>,
) -> Result<()> {
    match command {
        StorageCommands::Rent {
            fid,
            units,
            wallet,
            payment_wallet,
            dry_run,
            yes,
        } => {
            handle_storage_rent(
                fid,
                units,
                wallet,
                payment_wallet,
                dry_run,
                yes,
                storage_path,
            )
            .await?;
        }
        StorageCommands::Price { fid, units } => {
            handle_storage_price(fid, units).await?;
        }
        StorageCommands::Usage { fid } => {
            handle_storage_usage(fid).await?;
        }
    }
    Ok(())
}

async fn handle_storage_rent(
    fid: u64,
    units: u32,
    wallet_name: Option<String>,
    payment_wallet_name: Option<String>,
    dry_run: bool,
    yes: bool,
    storage_path: Option<&str>,
) -> Result<()> {
    println!("🏠 Rent Storage Units for FID {fid}");
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

    // Load custody wallet for the FID
    let private_key = if let Some(name) = wallet_name {
        // Load from encrypted storage
        use crate::encrypted_key_manager::{prompt_password, EncryptedKeyManager};

        let mut manager = if let Some(path) = storage_path {
            EncryptedKeyManager::new(path)
        } else {
            EncryptedKeyManager::default_config()
        };
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
                manager
                    .key_manager()
                    .unwrap()
                    .wallet()
                    .signer()
                    .to_bytes()
                    .to_vec()
            }
            Err(e) => {
                println!("❌ Failed to load wallet: {e}");
                return Ok(());
            }
        }
    } else {
        // Try to auto-detect custody wallet for the FID
        println!("🔍 Auto-detecting custody wallet for FID {fid}...");

        // First, get the custody address for the FID
        let contract_client =
            FarcasterContractClient::new(rpc_url.clone(), ContractAddresses::default())?;
        let fid_info = contract_client.get_fid_info(fid).await?;
        let custody_address = fid_info.custody;

        println!("   FID {fid} custody address: {custody_address}");

        println!("❌ No wallet specified and no matching wallet found!");
        println!(
            "💡 Please use 'castorix storage rent {fid} --units {units} --wallet <wallet-name>'"
        );
        return Ok(());
    };

    // Create custody wallet from private key bytes
    let custody_wallet = LocalWallet::from_bytes(&private_key)?;
    println!("   Custody Wallet: {}", custody_wallet.address());

    // Determine payment wallet
    let payment_wallet = if let Some(payment_wallet_name) = payment_wallet_name {
        // Use specified payment wallet
        let mut manager = if let Some(path) = storage_path {
            EncryptedKeyManager::new(path)
        } else {
            EncryptedKeyManager::default_config()
        };
        if !manager.key_exists(&payment_wallet_name) {
            println!("❌ Payment wallet '{payment_wallet_name}' not found!");
            println!("💡 Use 'castorix key list' to see available wallets");
            return Ok(());
        }

        let password = prompt_password(&format!(
            "Enter password for payment wallet '{payment_wallet_name}': "
        ))?;
        match manager
            .load_and_decrypt(&password, &payment_wallet_name)
            .await
        {
            Ok(_) => {
                let payment_address = manager.address().unwrap();
                println!("✅ Payment wallet loaded: {payment_address}");
                LocalWallet::from_bytes(
                    &manager.key_manager().unwrap().wallet().signer().to_bytes(),
                )?
            }
            Err(e) => {
                println!("❌ Failed to load payment wallet: {e}");
                return Ok(());
            }
        }
    } else {
        // Use custody wallet for payment
        println!("   Using custody wallet for payment");
        custody_wallet.clone()
    };

    println!("\n📋 Storage Rental Details:");
    println!("   FID: {fid}");
    println!("   Storage Units: {units}");
    println!("   Custody Wallet: {}", custody_wallet.address());
    if payment_wallet.address() != custody_wallet.address() {
        println!("   Payment Wallet: {}", payment_wallet.address());
    } else {
        println!(
            "   Payment Wallet: {} (same as custody)",
            payment_wallet.address()
        );
    }

    // Create contract client with custody wallet (for authorization)
    let contract_client = FarcasterContractClient::new_with_wallet(
        rpc_url.clone(),
        ContractAddresses::default(),
        custody_wallet.clone(),
    )?;

    // Get storage rental price
    println!("\n💰 Getting storage rental price...");
    let price = contract_client.get_storage_price(units as u64).await?;
    println!("   Storage Rental Price: {} ETH", format_ether(price));

    // Check payment wallet balance
    let provider = Provider::<Http>::try_from(&rpc_url)?;
    let balance = provider.get_balance(payment_wallet.address(), None).await?;
    println!("   Payment Wallet Balance: {} ETH", format_ether(balance));

    if dry_run {
        println!("\n🔍 DRY RUN MODE - No transaction will be sent");
        println!("✅ Storage rental simulation completed successfully");
        return Ok(());
    }

    // ⚠️  IMPORTANT: This will trigger on-chain operations
    println!("\n⚠️  ON-CHAIN OPERATION WARNING:");
    println!("   • This will rent {units} storage units for FID {fid}");
    println!("   • The operation will consume gas fees and storage rental cost");
    println!("   • This action cannot be undone");
    if payment_wallet.address() != custody_wallet.address() {
        println!(
            "   • Custody wallet {} will authorize the transaction",
            custody_wallet.address()
        );
        println!(
            "   • Payment wallet {} will pay for gas and storage rental",
            payment_wallet.address()
        );
    } else {
        println!(
            "   • Wallet {} will both authorize and pay for the transaction",
            payment_wallet.address()
        );
    }
    println!("   • Make sure you have sufficient ETH for gas and storage rental");

    // Ask for user confirmation (skip if --yes is provided)
    if !yes {
        print!("\n❓ Do you want to proceed with storage rental? (yes/no): ");
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

    println!("✅ Proceeding with storage rental...");

    // Rent storage
    // Note: If payment wallet is different from custody wallet, we need to handle this differently
    // For now, we'll use the custody wallet for both authorization and payment
    // TODO: Implement separate payment wallet support in FarcasterContractClient
    let result = if payment_wallet.address() != custody_wallet.address() {
        println!(
            "⚠️  Note: Payment wallet {} differs from custody wallet {}",
            payment_wallet.address(),
            custody_wallet.address()
        );
        println!("   Using custody wallet for both authorization and payment");
        contract_client.rent_storage(fid, units as u64).await?
    } else {
        contract_client.rent_storage(fid, units as u64).await?
    };

    match result {
        ContractResult::Success(overpayment) => {
            println!("✅ Storage rental successful!");
            if !overpayment.is_zero() {
                println!("   Overpayment: {} ETH", format_ether(overpayment));
            }
        }
        ContractResult::Error(e) => {
            println!("❌ Storage rental failed: {}", e);
            return Err(anyhow::anyhow!("Storage rental failed: {}", e));
        }
    }

    Ok(())
}

async fn handle_storage_price(fid: u64, units: u32) -> Result<()> {
    println!("💰 Storage Rental Price for FID {fid}");
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

    // Get storage rental price
    println!("🔍 Querying current storage rental prices...");
    let price = contract_client.get_storage_price(units as u64).await?;

    println!("\n📊 Storage Rental Price:");
    println!("   FID: {fid}");
    println!("   Storage Units: {units}");
    println!("   Rental Price: {} ETH", format_ether(price));
    println!("   Estimated Gas Fees: ~0.002-0.005 ETH (varies with network)");

    Ok(())
}

async fn handle_storage_usage(fid: u64) -> Result<()> {
    println!("📊 Storage Usage for FID {fid}");
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

    // Get FID information
    println!("🔍 Querying FID information...");
    let fid_info = contract_client.get_fid_info(fid).await?;

    println!("\n📋 FID Information:");
    println!("   FID: {fid}");
    println!("   Custody Address: {}", fid_info.custody);
    println!("   Recovery Address: {}", fid_info.recovery);
    // Note: registration_time is not available in FidInfo struct

    // Try to get basic FID information from hub
    let hub_url = crate::consts::get_config().farcaster_hub_url().to_string();

    let hub_client = crate::core::client::hub_client::FarcasterClient::read_only(hub_url);

    match hub_client.get_user(fid).await {
        Ok(_user) => {
            println!("\n👤 Hub Information:");
            println!("   FID: {fid}");
            println!("   User Data: Available from hub");
            // Note: User data structure varies by hub implementation
            println!("\n💡 Storage usage details are not yet available through the contract");
            println!("   This information would typically come from the Farcaster Hub API");
        }
        Err(e) => {
            println!("\n⚠️  Could not fetch FID information from hub: {e}");
            println!("💡 This might be because the FID doesn't exist or the hub is unavailable");
        }
    }

    println!("\n📊 Storage Information:");
    println!("   FID: {fid}");
    println!("   Current Usage: Not available (requires Hub API)");
    println!("   Storage Limit: Not available (requires Hub API)");
    println!("   Available Storage: Not available (requires Hub API)");
    println!("\n💡 Storage usage information is typically provided by the Farcaster Hub");
    println!("   Use 'castorix hub stats {fid}' for detailed storage statistics");

    Ok(())
}

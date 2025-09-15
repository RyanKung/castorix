use anyhow::Result;
use castorix::consts::get_config;
use castorix::farcaster::contracts::FarcasterContractClient;
use ethers::types::Address;

/// Example demonstrating how to read data from Farcaster contracts on-chain
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Farcaster On-Chain Data Reader");
    println!("================================\n");

    // Initialize configuration
    let config = get_config();
    println!(
        "📡 Using Optimism RPC: {}",
        mask_url(config.eth_op_rpc_url())
    );

    // Create Farcaster contract client
    let client =
        FarcasterContractClient::new_with_default_addresses(config.eth_op_rpc_url().to_string())?;

    println!("✅ Connected to Farcaster contracts on Optimism\n");

    // Display contract addresses
    let addresses = client.addresses();
    println!("📋 Contract Addresses:");
    println!("  ID Registry: {:?}", addresses.id_registry);
    println!("  Key Registry: {:?}", addresses.key_registry);
    println!("  Storage Registry: {:?}", addresses.storage_registry);
    println!("  ID Gateway: {:?}", addresses.id_gateway);
    println!("  Key Gateway: {:?}", addresses.key_gateway);
    println!();

    // Verify contract connectivity
    println!("🔍 Verifying contract connectivity...");
    match client.verify_contracts().await {
        Ok(result) => {
            println!("📊 Contract Status:");
            println!(
                "  ID Registry: {}",
                if result.id_registry { "✅" } else { "❌" }
            );
            println!(
                "  Key Registry: {}",
                if result.key_registry { "✅" } else { "❌" }
            );
            println!(
                "  Storage Registry: {}",
                if result.storage_registry {
                    "✅"
                } else {
                    "❌"
                }
            );
            println!(
                "  ID Gateway: {}",
                if result.id_gateway { "✅" } else { "❌" }
            );
            println!(
                "  Key Gateway: {}",
                if result.key_gateway { "✅" } else { "❌" }
            );

            if !result.all_working {
                println!("\n⚠️  Some contracts are not accessible:");
                for error in result.errors {
                    println!("  - {}", error);
                }
            }
        }
        Err(e) => {
            println!("❌ Contract verification failed: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // Get network information
    println!("🌐 Network Information:");
    match client.get_network_info().await {
        Ok(info) => {
            println!("  Chain ID: {} (Optimism)", info.chain_id);
            println!("  Current Block: {}", info.block_number);
            println!("  Gas Price: {} wei", info.gas_price);
        }
        Err(e) => {
            println!("❌ Failed to get network info: {}", e);
        }
    }
    println!();

    // Read data from specific FIDs
    let test_fids = vec![1u64, 2u64, 3u64, 10u64, 100u64];
    println!("👥 Reading FID Data:");
    println!("===================");

    for fid in test_fids {
        println!("\n🔍 FID {}:", fid);

        // Read owner
        match client.id_registry().owner_of(fid).await {
            Ok(result) => match result {
                castorix::farcaster::contracts::types::ContractResult::Success(owner) => {
                    if owner != Address::zero() {
                        println!("  👤 Owner: {:?}", owner);
                    } else {
                        println!("  👤 Owner: Not registered");
                    }
                }
                castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                    println!("  ⚠️  Owner read failed: {}", msg);
                }
            },
            Err(e) => {
                println!("  ❌ Owner read error: {}", e);
            }
        }

        // Read recovery address
        match client.id_registry().recovery_of(fid).await {
            Ok(result) => match result {
                castorix::farcaster::contracts::types::ContractResult::Success(recovery) => {
                    if recovery != Address::zero() {
                        println!("  🔐 Recovery: {:?}", recovery);
                    } else {
                        println!("  🔐 Recovery: Not set");
                    }
                }
                castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                    println!("  ⚠️  Recovery read failed: {}", msg);
                }
            },
            Err(e) => {
                println!("  ❌ Recovery read error: {}", e);
            }
        }

        // Read key count
        match client.key_registry().key_count_of(fid).await {
            Ok(result) => {
                match result {
                    castorix::farcaster::contracts::types::ContractResult::Success(count) => {
                        println!("  🔑 Keys: {} registered", count);

                        // If there are keys, try to read them
                        if count > 0 {
                            match client.key_registry().keys_of(fid).await {
                                Ok(result) => {
                                    match result {
                                        castorix::farcaster::contracts::types::ContractResult::Success(keys) => {
                                            println!("    📝 Key details:");
                                            for (i, key) in keys.iter().enumerate().take(3) {
                                                println!("      Key {}: {} bytes", i + 1, key.len());
                                            }
                                            if keys.len() > 3 {
                                                println!("      ... and {} more keys", keys.len() - 3);
                                            }
                                        }
                                        castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                                            println!("    ⚠️  Keys read failed: {}", msg);
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("    ❌ Keys read error: {}", e);
                                }
                            }
                        }
                    }
                    castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                        println!("  ⚠️  Key count read failed: {}", msg);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Key count read error: {}", e);
            }
        }
    }

    // Read storage registry data
    println!("\n💾 Storage Registry Data:");
    println!("========================");

    match client.storage_registry().price_per_unit().await {
        Ok(result) => {
            match result {
                castorix::farcaster::contracts::types::ContractResult::Success(price) => {
                    println!("💰 Storage price per unit: {} wei", price);
                    // Handle potential overflow when converting to u128
                    match TryInto::<u128>::try_into(price) {
                        Ok(price_u128) => {
                            println!(
                                "💰 Storage price per unit: {} ETH",
                                format!("{:.18}", price_u128 as f64 / 1e18)
                            );
                        }
                        Err(_) => {
                            println!(
                                "💰 Storage price per unit: {} (too large to convert to ETH)",
                                price
                            );
                        }
                    }
                }
                castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                    println!("⚠️  Storage price read failed: {}", msg);
                }
            }
        }
        Err(e) => {
            println!("❌ Storage price read error: {}", e);
        }
    }

    // Read ID Gateway data
    println!("\n🚪 ID Gateway Data:");
    println!("==================");

    match client.id_gateway().total_supply().await {
        Ok(result) => match result {
            castorix::farcaster::contracts::types::ContractResult::Success(supply) => {
                println!("📊 Total FIDs minted: {}", supply);
            }
            castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                println!("⚠️  Total supply read failed: {}", msg);
            }
        },
        Err(e) => {
            println!("❌ Total supply read error: {}", e);
        }
    }

    println!("\n🎉 On-chain data reading completed successfully!");
    println!(
        "💡 This demonstrates the ability to read real data from Farcaster contracts on Optimism."
    );

    Ok(())
}

/// Helper function to mask sensitive information in URLs
fn mask_url(url: &str) -> String {
    if url.contains("your_api_key_here") {
        format!("{} (⚠️  Please set your actual API key)", url)
    } else if let Some(api_key_start) = url.find("/v2/") {
        if api_key_start + 4 < url.len() {
            format!("{}***", &url[..api_key_start + 4])
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

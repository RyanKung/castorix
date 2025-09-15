use castorix::farcaster::contracts::FarcasterContractClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Farcaster Contracts Demo");

    // Create client from environment variables
    let client = FarcasterContractClient::from_env()?;

    println!("✅ Client created successfully");

    // Get contract addresses
    let addresses = client.addresses();
    println!("📋 Contract addresses:");
    println!("  ID Gateway: {:?}", addresses.id_gateway);
    println!("  ID Registry: {:?}", addresses.id_registry);
    println!("  Key Gateway: {:?}", addresses.key_gateway);
    println!("  Key Registry: {:?}", addresses.key_registry);
    println!("  Storage Registry: {:?}", addresses.storage_registry);

    // Verify contract connections
    println!("\n🔍 Verifying contract connections...");
    match client.verify_contracts().await {
        Ok(result) => {
            if result.all_working {
                println!("✅ All contract connections are working");
            } else {
                println!("⚠️  Some contract connections failed:");
                for error in result.errors {
                    println!("  - {}", error);
                }
                println!("📊 Contract status:");
                println!(
                    "  - ID Registry: {}",
                    if result.id_registry { "✅" } else { "❌" }
                );
                println!(
                    "  - Key Registry: {}",
                    if result.key_registry { "✅" } else { "❌" }
                );
                println!(
                    "  - Storage Registry: {}",
                    if result.storage_registry {
                        "✅"
                    } else {
                        "❌"
                    }
                );
                println!(
                    "  - ID Gateway: {}",
                    if result.id_gateway { "✅" } else { "❌" }
                );
                println!(
                    "  - Key Gateway: {}",
                    if result.key_gateway { "✅" } else { "❌" }
                );
            }
        }
        Err(e) => {
            println!("❌ Verification failed: {}", e);
        }
    }

    // Test ID Registry
    println!("\n🔍 Testing ID Registry...");
    match client.id_registry().owner_of(1).await {
        Ok(result) => match result {
            castorix::farcaster::contracts::types::ContractResult::Success(owner) => {
                println!("✅ Owner of FID 1: {:?}", owner);
            }
            castorix::farcaster::contracts::types::ContractResult::Error(e) => {
                println!("⚠️  Query failed: {}", e);
            }
        },
        Err(e) => {
            println!("❌ Call failed: {}", e);
        }
    }

    // Test Storage Registry
    println!("\n🔍 Testing Storage Registry...");
    match client.storage_registry().price_per_unit().await {
        Ok(result) => match result {
            castorix::farcaster::contracts::types::ContractResult::Success(price) => {
                println!("✅ Storage price: {}", price);
            }
            castorix::farcaster::contracts::types::ContractResult::Error(e) => {
                println!("⚠️  Query failed: {}", e);
            }
        },
        Err(e) => {
            println!("❌ Call failed: {}", e);
        }
    }

    println!("\n🎉 Demo completed!");
    Ok(())
}

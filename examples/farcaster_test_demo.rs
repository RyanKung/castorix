use anyhow::Result;
use castorix::farcaster::contracts::FarcasterContractClient;

/// Simple demo showing how to use the Farcaster testing framework
#[tokio::main]
async fn main() -> Result<()> {
    println!("🌟 Farcaster Test Demo");
    println!("=====================");

    // Check if we're in a test environment
    if std::env::var("RUNNING_TESTS").is_ok() {
        println!("🧪 Running in test environment");

        // Create a test client
        let config = match std::env::var("LOCAL_TEST") {
            Ok(_) => {
                println!("🏠 Using local test configuration");
                // Use local Anvil configuration
                "http://localhost:8545".to_string()
            }
            Err(_) => {
                println!("🌐 Using testnet configuration");
                // Use testnet configuration
                std::env::var("ETH_OP_RPC_URL").unwrap_or_else(|_| {
                    "https://goerli-optimism.g.alchemy.com/v2/your-api-key".to_string()
                })
            }
        };

        // Create client
        let client = FarcasterContractClient::new_with_default_addresses(config.to_string())?;

        // Test contract connectivity
        println!("\n🔍 Testing contract connectivity...");
        match client.verify_contracts().await {
            Ok(result) => {
                if result.all_working {
                    println!("✅ All contracts are accessible");
                } else {
                    println!("⚠️  Some contracts are not accessible:");
                    for error in result.errors {
                        println!("  - {}", error);
                    }
                }
            }
            Err(e) => {
                println!("❌ Contract verification failed: {}", e);
            }
        }

        // Get network info
        println!("\n🌐 Network information:");
        match client.get_network_info().await {
            Ok(info) => {
                println!("   Chain ID: {}", info.chain_id);
                println!("   Block Number: {}", info.block_number);
                println!("   Gas Price: {} wei", info.gas_price);
            }
            Err(e) => {
                println!("❌ Failed to get network info: {}", e);
            }
        }

        println!("\n🎉 Demo completed successfully!");
    } else {
        println!("📖 This demo shows how to use the Farcaster testing framework.");
        println!();
        println!("To run the actual tests, use one of these commands:");
        println!();
        println!("1. Run the complete example:");
        println!("   LOCAL_TEST=1 cargo run --example complete_farcaster_test");
        println!();
        println!("2. Run integration tests:");
        println!("   ./scripts/run-farcaster-tests.sh");
        println!();
        println!("3. Run specific tests:");
        println!("   ./scripts/run-farcaster-tests.sh connectivity");
        println!("   ./scripts/run-farcaster-tests.sh fid");
        println!("   ./scripts/run-farcaster-tests.sh storage");
        println!("   ./scripts/run-farcaster-tests.sh signer");
        println!("   ./scripts/run-farcaster-tests.sh fname");
        println!();
        println!("4. Run with Cargo directly:");
        println!("   RUNNING_TESTS=1 cargo test farcaster_integration_test");
        println!();
        println!("📚 See FARCASTER_TESTING_GUIDE.md for detailed instructions.");
    }

    Ok(())
}

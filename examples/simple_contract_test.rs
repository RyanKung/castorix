use anyhow::Result;
use castorix::consts::get_config;
use castorix::farcaster::contracts::FarcasterContractClient;

/// Simple test to verify basic contract connectivity
#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Simple Farcaster Contract Test");
    println!("================================\n");

    let config = get_config();
    println!(
        "📡 Using Optimism RPC: {}",
        mask_url(config.eth_op_rpc_url())
    );

    let client =
        FarcasterContractClient::new(config.eth_op_rpc_url().to_string(), castorix::farcaster::contracts::types::ContractAddresses::default())?;

    println!("✅ Connected to Farcaster contracts on Optimism\n");

    // Test network info first
    println!("🌐 Network Information:");
    match client.get_network_info().await {
        Ok(info) => {
            println!("  Chain ID: {} (Expected: 10 for Optimism)", info.chain_id);
            println!("  Current Block: {}", info.block_number);
            println!("  Gas Price: {} wei", info.gas_price);

            if info.chain_id == 10 {
                println!("✅ Confirmed: Connected to Optimism mainnet");
            } else {
                println!(
                    "⚠️  Warning: Not connected to Optimism (chain ID: {})",
                    info.chain_id
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to get network info: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // Test contract verification
    println!("🔍 Contract Verification:");
    match client.verify_contracts().await {
        Ok(result) => {
            println!("📊 Results:");
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

            let working_count = [
                result.id_registry,
                result.key_registry,
                result.storage_registry,
                result.id_gateway,
                result.key_gateway,
            ]
            .iter()
            .filter(|&&x| x)
            .count();

            println!("📈 Summary: {}/5 contracts accessible", working_count);

            if !result.errors.is_empty() {
                println!("\n⚠️  Errors:");
                for error in result.errors {
                    println!("  - {}", error);
                }
            }

            if working_count >= 3 {
                println!("✅ Good: Most contracts are accessible");
            } else if working_count > 0 {
                println!("⚠️  Partial: Some contracts are accessible");
            } else {
                println!("❌ Poor: No contracts are accessible");
            }
        }
        Err(e) => {
            println!("❌ Contract verification failed: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // Test a simple storage registry call (this should work)
    println!("💾 Testing Storage Registry (simple call):");
    match client.storage_registry().price_per_unit().await {
        Ok(result) => match result {
            castorix::farcaster::contracts::types::ContractResult::Success(price) => {
                println!("✅ Storage price: {} wei", price);
                println!("✅ This confirms basic contract communication is working");
            }
            castorix::farcaster::contracts::types::ContractResult::Error(msg) => {
                println!("⚠️  Storage price read failed: {}", msg);
                println!("   This might indicate a contract interface mismatch");
            }
        },
        Err(e) => {
            println!("❌ Storage price read error: {}", e);
            println!("   This indicates a network or RPC issue");
        }
    }

    println!("\n🎉 Basic contract test completed!");
    println!("💡 If storage registry works but ID registry doesn't,");
    println!("   it might indicate different contract interfaces or addresses.");

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

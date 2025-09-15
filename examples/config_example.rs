use castorix::consts::{get_config, init_config};

/// Example showing how to use the configuration module
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Castorix Configuration Example ===\n");

    // Initialize and validate configuration
    match init_config() {
        Ok(()) => {
            println!("✅ Configuration loaded and validated successfully!\n");
        }
        Err(e) => {
            eprintln!("❌ Configuration error: {}\n", e);
            println!("💡 Make sure you have a .env file with the required environment variables.");
            println!("   You can copy env.example to .env and fill in your actual values.\n");
            return Err(e);
        }
    }

    // Get the global configuration instance
    let config = get_config();

    // Example: Use configuration in your application
    println!("=== Using Configuration in Application ===");

    // Example: Create a Farcaster client with the configured URLs
    println!(
        "📡 Ethereum RPC URL: {}",
        mask_sensitive_info(config.eth_rpc_url())
    );
    println!("🔗 Base RPC URL: {}", config.eth_base_rpc_url());
    println!("⚡ Optimism RPC URL: {}", config.eth_op_rpc_url());
    println!("🌐 Farcaster Hub URL: {}", config.farcaster_hub_url());

    // Example: Initialize Farcaster contract client
    println!("\n=== Initializing Farcaster Contract Client ===");
    match castorix::farcaster::contracts::FarcasterContractClient::new_with_default_addresses(
        config.eth_op_rpc_url().to_string(),
    ) {
        Ok(client) => {
            println!("✅ Farcaster contract client initialized successfully!");
            println!("📋 Contract addresses:");
            let addresses = client.get_addresses_map();
            for (name, address) in addresses {
                println!("   {}: {}", name, address);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize Farcaster contract client: {}", e);
            println!("💡 This might be due to network connectivity or invalid RPC URL.");
        }
    }

    println!("\n=== Example Complete ===");
    Ok(())
}

/// Helper function to mask sensitive information in URLs
fn mask_sensitive_info(url: &str) -> String {
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

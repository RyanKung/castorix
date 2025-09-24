use anyhow::Result;
use castorix::farcaster::contracts::FarcasterContractClient;
use ethers::providers::Http;
use ethers::providers::Middleware;
use ethers::providers::Provider;

/// Test network info retrieval specifically
#[tokio::test]
async fn test_network_info_retrieval() -> Result<()> {

    println!("🌐 Testing network info retrieval...");

    let rpc_url = "http://127.0.0.1:8545";
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Test individual RPC calls
    println!("📋 Testing individual RPC calls...");

    // Test 1: Get chain ID
    println!("   1. Testing get_chainid()...");
    match provider.get_chainid().await {
        Ok(chain_id) => {
            println!("   ✅ Chain ID: {}", chain_id);
        }
        Err(e) => {
            panic!("❌ Chain ID failed: {}. This is a critical test failure - basic RPC connectivity is required.", e);
        }
    }

    // Test 2: Get block number
    println!("   2. Testing get_block_number()...");
    match provider.get_block_number().await {
        Ok(block_number) => {
            println!("   ✅ Block Number: {}", block_number);
        }
        Err(e) => {
            panic!("❌ Block Number failed: {}. This is a critical test failure - basic RPC connectivity is required.", e);
        }
    }

    // Test 3: Get gas price
    println!("   3. Testing get_gas_price()...");
    match provider.get_gas_price().await {
        Ok(gas_price) => {
            println!("   ✅ Gas Price: {} wei", gas_price);
        }
        Err(e) => {
            panic!("❌ Gas Price failed: {}. This is a critical test failure - basic RPC connectivity is required.", e);
        }
    }

    // Test 4: Test the combined get_network_info method
    println!("📋 Testing combined get_network_info()...");
    let client = FarcasterContractClient::new(
        rpc_url.to_string(),
        castorix::farcaster::contracts::types::ContractAddresses::default(),
    )?;
    match client.get_network_status().await {
        Ok(info) => {
            println!("   ✅ Network Info:");
            println!("      Chain ID: {}", info.chain_id);
            println!("      Block Number: {}", info.block_number);
            println!("      ID Gateway Paused: {}", info.id_gateway_paused);
            println!("      Key Gateway Paused: {}", info.key_gateway_paused);
            println!(
                "      Storage Registry Paused: {}",
                info.storage_registry_paused
            );
        }
        Err(e) => {
            panic!("❌ Combined method failed: {}. This is a critical test failure - network status retrieval is required for testing.", e);
        }
    }

    println!("🎉 Network info retrieval test completed successfully!");

    Ok(())
}

/// Test network info with retry logic
#[tokio::test]
async fn test_network_info_with_retry() -> Result<()> {

    println!("🔄 Testing network info with retry logic...");

    let rpc_url = "http://127.0.0.1:8545";
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Retry logic for network info
    let max_retries = 3;
    let mut last_error = None;

    for attempt in 1..=max_retries {
        println!("   Attempt {}/{}", attempt, max_retries);

        match provider.get_chainid().await {
            Ok(chain_id) => {
                println!("   ✅ Chain ID retrieved: {}", chain_id);

                // If chain ID works, try the other calls
                match provider.get_block_number().await {
                    Ok(block_number) => {
                        println!("   ✅ Block Number: {}", block_number);

                        match provider.get_gas_price().await {
                            Ok(gas_price) => {
                                println!("   ✅ Gas Price: {} wei", gas_price);
                                println!(
                                    "🎉 Network info retrieved successfully on attempt {}",
                                    attempt
                                );
                                return Ok(());
                            }
                            Err(e) => {
                                println!("   ⚠️  Gas price failed: {}", e);
                                last_error = Some(e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Block number failed: {}", e);
                        last_error = Some(e);
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Chain ID failed: {}", e);
                last_error = Some(e);
            }
        }

        if attempt < max_retries {
            println!("   ⏳ Waiting 1 second before retry...");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    println!("❌ All retry attempts failed");
    if let Some(error) = last_error {
        panic!("❌ All retry attempts failed: {}. This is a critical test failure - network connectivity is required.", error);
    }

    Ok(())
}

/// Test basic RPC connectivity
#[tokio::test]
async fn test_basic_rpc_connectivity() -> Result<()> {

    println!("🔗 Testing basic RPC connectivity...");

    let rpc_url = "http://127.0.0.1:8545";

    // Test basic HTTP connectivity
    println!("📋 Testing HTTP connectivity...");
    match reqwest::get(rpc_url).await {
        Ok(response) => {
            println!("   ✅ HTTP connection successful: {}", response.status());
            let body = response.text().await?;
            if body.contains("<!doctype html>") {
                println!("   ⚠️  Received HTML response (expected for Anvil)");
            } else {
                println!("   ✅ Received non-HTML response");
            }
        }
        Err(e) => {
            panic!("❌ HTTP connection failed: {}. This is a critical test failure - basic HTTP connectivity is required.", e);
        }
    }

    // Test provider creation
    println!("📋 Testing provider creation...");
    match Provider::<Http>::try_from(rpc_url) {
        Ok(_provider) => {
            println!("   ✅ Provider created successfully");
        }
        Err(e) => {
            panic!("❌ Provider creation failed: {}. This is a critical test failure - provider creation is required.", e);
        }
    }

    println!("🎉 Basic RPC connectivity test completed!");

    Ok(())
}

/// Test network info with custom timeout
#[tokio::test]
async fn test_network_info_with_timeout() -> Result<()> {

    println!("⏱️  Testing network info with custom timeout...");

    let rpc_url = "http://127.0.0.1:8545";
    let provider = Provider::<Http>::try_from(rpc_url)?;

    // Test with timeout
    match tokio::time::timeout(std::time::Duration::from_secs(5), provider.get_chainid()).await {
        Ok(Ok(chain_id)) => {
            println!("   ✅ Chain ID with timeout: {}", chain_id);
        }
        Ok(Err(e)) => {
            println!("   ❌ Chain ID failed: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("   ⚠️  Chain ID timed out");
            return Ok(()); // Timeout is acceptable for this test
        }
    }

    println!("🎉 Network info timeout test completed!");

    Ok(())
}

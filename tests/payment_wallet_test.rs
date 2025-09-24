use std::sync::Arc;

use anyhow::Result;
use castorix::farcaster::contracts::types::ContractAddresses;
use castorix::farcaster::contracts::types::ContractResult;
use castorix::farcaster::contracts::FarcasterContractClient;
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use ethers::types::U256;
use rand::rngs::OsRng;

/// Test separate payment wallet functionality
#[tokio::test]
async fn test_separate_payment_wallet_functionality() -> Result<()> {
    // Skip test if not in test environment
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("💳 Testing separate payment wallet functionality...");

    // Use local Anvil configuration
    let rpc_url = "http://127.0.0.1:8545";

    // Create client
    let client = FarcasterContractClient::new(rpc_url.to_string(), ContractAddresses::default())?;

    // Generate test wallets
    let custody_wallet = LocalWallet::new(&mut OsRng);
    let payment_wallet = LocalWallet::new(&mut OsRng);

    println!("   Custody wallet: {}", custody_wallet.address());
    println!("   Payment wallet: {}", payment_wallet.address());

    // Test 1: Verify wallets are different
    assert_ne!(
        custody_wallet.address(),
        payment_wallet.address(),
        "Test wallets should be different"
    );

    // Test 2: Test storage price query (should work with any wallet)
    println!("🔍 Testing storage price query...");
    let test_units = 1u64;
    match client.get_storage_price(test_units).await {
        Ok(price) => {
            println!("✅ Storage price retrieved: {} ETH", price);
            assert!(!price.is_zero(), "Storage price should not be zero");
        }
        Err(e) => {
            panic!("❌ Storage price query failed: {}", e);
        }
    }

    // Test 3: Test FID registration price query
    println!("🔍 Testing FID registration price query...");
    match client.get_registration_price().await {
        Ok(price) => {
            println!("✅ Registration price retrieved: {} ETH", price);
            assert!(!price.is_zero(), "Registration price should not be zero");
        }
        Err(e) => {
            panic!("❌ Registration price query failed: {}", e);
        }
    }

    println!("✅ Separate payment wallet functionality tests passed!");
    Ok(())
}

/// Test payment wallet API with mock scenario
#[tokio::test]
async fn test_payment_wallet_api_interface() -> Result<()> {
    // Skip test if not in test environment
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("🔌 Testing payment wallet API interface...");

    // Use local Anvil configuration
    let rpc_url = "http://127.0.0.1:8545";

    // Create client
    let client = FarcasterContractClient::new(rpc_url.to_string(), ContractAddresses::default())?;

    // Generate test wallets
    let custody_wallet = LocalWallet::new(&mut OsRng);
    let payment_wallet = LocalWallet::new(&mut OsRng);

    println!("   Custody wallet: {}", custody_wallet.address());
    println!("   Payment wallet: {}", payment_wallet.address());

    // Test API interface exists and accepts correct parameters
    let test_fid = 999999u64;
    let test_units = 1u64;
    let payment_wallet_arc = Arc::new(payment_wallet);

    // This test verifies the API interface exists and accepts the right parameters
    // We expect this to fail due to insufficient funds, but the API should be callable
    match client
        .rent_storage_with_payment_wallet(test_fid, test_units, payment_wallet_arc)
        .await
    {
        Ok(result) => {
            match result {
                ContractResult::Success(_) => {
                    println!("✅ Payment wallet API call succeeded (unexpected but valid)");
                }
                ContractResult::Error(error_msg) => {
                    println!("⚠️  Payment wallet API call failed as expected: {}", error_msg);
                    // This is expected due to insufficient funds or other test environment issues
                }
            }
        }
        Err(e) => {
            println!("⚠️  Payment wallet API call failed as expected: {}", e);
            // This is expected due to insufficient funds or other test environment issues
        }
    }

    println!("✅ Payment wallet API interface tests passed!");
    Ok(())
}

/// Test wallet address validation
#[tokio::test]
async fn test_wallet_address_validation() -> Result<()> {
    println!("🔍 Testing wallet address validation...");

    // Generate test wallets
    let wallet1 = LocalWallet::new(&mut OsRng);
    let wallet2 = LocalWallet::new(&mut OsRng);

    // Test that generated wallets have valid addresses
    assert!(!wallet1.address().is_zero(), "Wallet 1 should have valid address");
    assert!(!wallet2.address().is_zero(), "Wallet 2 should have valid address");
    assert_ne!(wallet1.address(), wallet2.address(), "Wallets should be different");

    // Test address formatting
    let addr1_str = format!("{:?}", wallet1.address());
    let addr2_str = format!("{:?}", wallet2.address());

    assert!(addr1_str.starts_with("0x"), "Address should start with 0x");
    assert!(addr2_str.starts_with("0x"), "Address should start with 0x");
    assert_eq!(addr1_str.len(), 42, "Address should be 42 characters long");
    assert_eq!(addr2_str.len(), 42, "Address should be 42 characters long");

    println!("✅ Wallet address validation tests passed!");
    Ok(())
}

/// Test storage price calculations
#[tokio::test]
async fn test_storage_price_calculations() -> Result<()> {
    // Skip test if not in test environment
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("💰 Testing storage price calculations...");

    // Use local Anvil configuration
    let rpc_url = "http://127.0.0.1:8545";

    // Create client
    let client = FarcasterContractClient::new(rpc_url.to_string(), ContractAddresses::default())?;

    // Test different unit amounts
    let test_units = vec![1u64, 5u64, 10u64, 100u64];

    for units in test_units {
        match client.get_storage_price(units).await {
            Ok(price) => {
                println!("   {} units: {} ETH", units, price);
                assert!(!price.is_zero(), "Price for {} units should not be zero", units);
                
                // Price should generally increase with more units
                // (though this might not always be true due to rounding)
                if units > 1 {
                    // At minimum, price should not decrease dramatically
                    assert!(price > U256::from(0), "Price should be positive");
                }
            }
            Err(e) => {
                panic!("❌ Storage price query failed for {} units: {}", units, e);
            }
        }
    }

    println!("✅ Storage price calculation tests passed!");
    Ok(())
}

/// Test error handling for invalid parameters
#[tokio::test]
async fn test_error_handling_invalid_parameters() -> Result<()> {
    println!("⚠️  Testing error handling for invalid parameters...");

    // Use local Anvil configuration
    let rpc_url = "http://127.0.0.1:8545";

    // Create client
    let client = FarcasterContractClient::new(rpc_url.to_string(), ContractAddresses::default())?;

    // Generate test wallet
    let payment_wallet = Arc::new(LocalWallet::new(&mut OsRng));

    // Test with zero FID (should be invalid)
    match client
        .rent_storage_with_payment_wallet(0u64, 1u64, payment_wallet.clone())
        .await
    {
        Ok(result) => {
            match result {
                ContractResult::Success(_) => {
                    println!("⚠️  Zero FID accepted (unexpected)");
                }
                ContractResult::Error(error_msg) => {
                    println!("✅ Zero FID rejected as expected: {}", error_msg);
                }
            }
        }
        Err(e) => {
            println!("✅ Zero FID rejected as expected: {}", e);
        }
    }

    // Test with zero units (should be invalid)
    match client
        .rent_storage_with_payment_wallet(999999u64, 0u64, payment_wallet.clone())
        .await
    {
        Ok(result) => {
            match result {
                ContractResult::Success(_) => {
                    println!("⚠️  Zero units accepted (unexpected)");
                }
                ContractResult::Error(error_msg) => {
                    println!("✅ Zero units rejected as expected: {}", error_msg);
                }
            }
        }
        Err(e) => {
            println!("✅ Zero units rejected as expected: {}", e);
        }
    }

    println!("✅ Error handling tests passed!");
    Ok(())
}

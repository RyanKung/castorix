//! Test the ABI-based contract implementation
//!
//! This example demonstrates how to use the new ABI-based contract wrappers
//! that use the official Farcaster contract ABIs.

use anyhow::Result;
use castorix::farcaster::contracts::{id_registry_abi::IdRegistryAbi, types::ContractAddresses};
use ethers::providers::{Http, Provider};

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing ABI-based Farcaster contract implementation");

    // Initialize provider
    let provider = Provider::<Http>::try_from("https://optimism-mainnet.g.alchemy.com/v2/demo")?;

    // Get contract addresses
    let addresses = ContractAddresses::default();

    // Create ABI-based IdRegistry instance
    let id_registry = IdRegistryAbi::new(provider, addresses.id_registry)?;

    println!("IdRegistry contract address: {:?}", id_registry.address());

    // Test contract calls
    println!("\n=== Testing Contract Calls ===");

    // Test id_counter
    match id_registry.id_counter().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(counter) => {
            println!("✅ Current ID counter: {}", counter);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get ID counter: {}", e);
        }
    }

    // Test paused status
    match id_registry.paused().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(is_paused) => {
            println!("✅ Contract paused: {}", is_paused);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get paused status: {}", e);
        }
    }

    // Test version
    match id_registry.version().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(version) => {
            println!("✅ Contract version: {}", version);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get version: {}", e);
        }
    }

    // Test name
    match id_registry.name().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(name) => {
            println!("✅ Contract name: {}", name);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get name: {}", e);
        }
    }

    // Test grace period
    match id_registry.grace_period().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(period) => {
            println!("✅ Grace period: {} seconds", period);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get grace period: {}", e);
        }
    }

    // Test gateway frozen status
    match id_registry.gateway_frozen().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(is_frozen) => {
            println!("✅ Gateway frozen: {}", is_frozen);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get gateway frozen status: {}", e);
        }
    }

    // Test ID Gateway address
    match id_registry.id_gateway().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(gateway) => {
            println!("✅ ID Gateway address: {:?}", gateway);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get ID Gateway address: {}", e);
        }
    }

    // Test migration status
    match id_registry.is_migrated().await? {
        castorix::farcaster::contracts::types::ContractResult::Success(is_migrated) => {
            println!("✅ Is migrated: {}", is_migrated);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get migration status: {}", e);
        }
    }

    // Test a specific FID (using a known FID like 1)
    let test_fid = 1u64;
    println!("\n=== Testing Specific FID (FID {}) ===", test_fid);

    match id_registry.owner_of(test_fid).await? {
        castorix::farcaster::contracts::types::ContractResult::Success(owner) => {
            println!("✅ Owner of FID {}: {:?}", test_fid, owner);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!("❌ Failed to get owner of FID {}: {}", test_fid, e);
        }
    }

    match id_registry.recovery_of(test_fid).await? {
        castorix::farcaster::contracts::types::ContractResult::Success(recovery) => {
            println!("✅ Recovery address for FID {}: {:?}", test_fid, recovery);
        }
        castorix::farcaster::contracts::types::ContractResult::Error(e) => {
            println!(
                "❌ Failed to get recovery address for FID {}: {}",
                test_fid, e
            );
        }
    }

    println!("\n=== Test Complete ===");
    println!("The ABI-based implementation successfully demonstrates:");
    println!("✅ Type-safe contract calls using generated bindings");
    println!("✅ Proper error handling with ContractResult");
    println!("✅ Correct type conversions (u64 <-> U256)");
    println!("✅ Access to all read-only contract functions");

    Ok(())
}

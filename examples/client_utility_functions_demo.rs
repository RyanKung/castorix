use anyhow::Result;
use castorix::farcaster::contracts::{
    FarcasterContractClient,
    types::ContractAddresses,
};
use ethers::types::Address;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 Farcaster Client Utility Functions Demo");
    println!("===========================================");

    // Initialize client with local Optimism fork
    let rpc_url = "http://localhost:8545".to_string();
    let addresses = ContractAddresses::default();
    let client = FarcasterContractClient::new(rpc_url, addresses)?;

    // Test with FID 1 (known to exist)
    let fid = 1u64;

    println!("\n📋 1. Getting detailed FID keys information...");
    match client.get_fid_keys_detailed(fid).await {
        Ok(keys_info) => {
            println!("✅ FID {} Keys Information:", keys_info.fid);
            println!("   Custody: {}", keys_info.custody);
            println!("   Recovery: {}", keys_info.recovery);
            println!("   Active Keys: {} ({} items)", keys_info.active_keys, keys_info.active_keys_list.len());
            println!("   Inactive Keys: {} ({} items)", keys_info.inactive_keys, keys_info.inactive_keys_list.len());
            println!("   Pending Keys: {} ({} items)", keys_info.pending_keys, keys_info.pending_keys_list.len());
            
            if !keys_info.inactive_keys_list.is_empty() {
                println!("   Sample Inactive Key: {}", keys_info.inactive_keys_list[0]);
            }
        }
        Err(e) => {
            println!("❌ Failed to get detailed FID keys: {}", e);
        }
    }

    println!("\n🔑 2. Generating unique signing key...");
    match client.generate_unique_signing_key(fid, 5).await {
        Ok(signing_key) => {
            let public_key = signing_key.verifying_key().to_bytes();
            println!("✅ Generated unique key: {}", hex::encode(public_key));
            
            // Test keypair verification
            println!("\n🔐 3. Verifying keypair...");
            let test_message = b"test message for keypair verification";
            match client.verify_keypair(&signing_key, test_message) {
                Ok(is_valid) => {
                    if is_valid {
                        println!("✅ Keypair verification successful");
                    } else {
                        println!("❌ Keypair verification failed");
                    }
                }
                Err(e) => {
                    println!("❌ Keypair verification error: {}", e);
                }
            }
            
            // Test signer registration verification
            println!("\n🔍 4. Verifying signer registration (should not be found)...");
            match client.verify_signer_registration(fid, public_key).await {
                Ok(verification_result) => {
                    println!("   Found: {}", verification_result.found);
                    println!("   Is Valid: {}", verification_result.is_valid);
                    println!("   Message: {}", verification_result.message);
                    
                    if verification_result.found {
                        println!("⚠️  Unexpected: Key found in registry");
                    } else {
                        println!("✅ Key not found in registry (as expected)");
                    }
                }
                Err(e) => {
                    println!("❌ Signer verification error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to generate unique signing key: {}", e);
        }
    }

    println!("\n🔒 5. Checking key management permissions...");
    let test_address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?;
    match client.can_manage_fid_keys(test_address, fid).await {
        Ok(can_manage) => {
            println!("   Address {} can manage FID {} keys: {}", test_address, fid, can_manage);
            if can_manage {
                println!("⚠️  Unexpected: Test address should not be able to manage FID {} keys", fid);
            } else {
                println!("✅ Correct: Test address cannot manage FID {} keys", fid);
            }
        }
        Err(e) => {
            println!("❌ Permission check error: {}", e);
        }
    }

    println!("\n🛡️ 6. Testing unauthorized key operations (security check)...");
    match client.test_unauthorized_key_operations(fid, test_address).await {
        Ok(result) => {
            println!("✅ Security test completed successfully");
            println!("   Target FID: {}", result.target_fid);
            println!("   Caller Address: {}", result.caller_address);
            println!("   Can Manage Keys: {}", result.can_manage_keys);
            println!("   Unauthorized Add Failed: {}", result.unauthorized_add_failed);
            println!("   Unauthorized Remove Failed: {}", result.unauthorized_remove_failed);
            println!("   Direct Remove Failed: {}", result.direct_remove_failed);
            println!("   Keys Unchanged: {}", result.keys_unchanged);
            
            if !result.error_messages.is_empty() {
                println!("   Error Messages:");
                for msg in &result.error_messages {
                    println!("     - {}", msg);
                }
            }
        }
        Err(e) => {
            println!("❌ Security test failed: {}", e);
        }
    }

    println!("\n🎉 Client utility functions demo completed successfully!");
    println!("These utility functions are now available in FarcasterContractClient:");
    println!("  - get_fid_keys_detailed() - Get comprehensive key information");
    println!("  - generate_unique_signing_key() - Generate unique Ed25519 keypairs");
    println!("  - verify_keypair() - Verify keypair validity");
    println!("  - verify_signer_registration() - Check signer registration status");
    println!("  - can_manage_fid_keys() - Check key management permissions");
    println!("  - test_unauthorized_key_operations() - Security testing");

    Ok(())
}

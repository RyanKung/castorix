use anyhow::Result;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use ethers::types::Address;

use crate::farcaster::contracts::{
    types::{Fid, ContractResult},
    contract_client::FarcasterContractClient,
    types::SecurityTestResult,
};

impl FarcasterContractClient {
    /// Test unauthorized key operations (security check)
    pub async fn test_unauthorized_key_operations(&self, target_fid: Fid, caller_address: Address) -> Result<SecurityTestResult> {
        let mut result = SecurityTestResult {
            target_fid,
            caller_address,
            can_manage_keys: false,
            unauthorized_add_failed: false,
            unauthorized_remove_failed: false,
            direct_remove_failed: false,
            keys_unchanged: false,
            error_messages: Vec::new(),
        };
        
        // Check if caller can manage keys
        result.can_manage_keys = self.can_manage_fid_keys(caller_address, target_fid).await?;
        
        if result.can_manage_keys {
            result.error_messages.push("Caller is authorized to manage keys - skipping unauthorized tests".to_string());
            return Ok(result);
        }
        
        // Test 1: Try unauthorized key addition
        let test_key = SigningKey::generate(&mut OsRng {}).verifying_key().to_bytes();
        let deadline = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)?.as_secs() + 3600;
        let simulated_sig = vec![0u8; 65];
        
        let target_fid_info = self.get_fid_info(target_fid).await?;
        let add_result = self.key_gateway.add_for(
            target_fid_info.custody,
            1,
            test_key.to_vec(),
            1,
            vec![],
            deadline.into(),
            simulated_sig.clone(),
        ).await?;
        
        match add_result {
            ContractResult::Success(_receipt) => {
                result.error_messages.push("SECURITY ISSUE: Unauthorized key addition succeeded!".to_string());
            }
            ContractResult::Error(_e) => {
                result.unauthorized_add_failed = true;
            }
        }
        
        // Test 2: Try unauthorized key removal
        let existing_keys = self.key_registry.keys_of(target_fid, 1).await?;
        if let ContractResult::Success(keys) = existing_keys {
            if !keys.is_empty() {
                let target_key = keys[0].clone();
                let remove_result = self.key_registry.remove_for(
                    target_fid_info.custody,
                    target_key,
                    deadline,
                    simulated_sig,
                ).await?;
                
                match remove_result {
                    ContractResult::Success(_receipt) => {
                        result.error_messages.push("SECURITY ISSUE: Unauthorized key removal succeeded!".to_string());
                    }
                    ContractResult::Error(_) => {
                        result.unauthorized_remove_failed = true;
                    }
                }
            }
        }
        
        // Test 3: Try direct key removal
        let remove_direct_result = self.key_registry.remove(test_key.to_vec()).await?;
        match remove_direct_result {
            ContractResult::Success(_) => {
                result.error_messages.push("SECURITY ISSUE: Direct unauthorized key removal succeeded!".to_string());
            }
            ContractResult::Error(_) => {
                result.direct_remove_failed = true;
            }
        }
        
        // Test 4: Verify keys are unchanged
        let final_fid_info = self.get_fid_info(target_fid).await?;
        if final_fid_info.active_keys == target_fid_info.active_keys &&
           final_fid_info.inactive_keys == target_fid_info.inactive_keys &&
           final_fid_info.pending_keys == target_fid_info.pending_keys {
            result.keys_unchanged = true;
        }
        
        Ok(result)
    }
}

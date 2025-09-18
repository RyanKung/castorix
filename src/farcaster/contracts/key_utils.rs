use anyhow::Result;
use ed25519_dalek::{Signer as Ed25519Signer, SigningKey, Verifier};
use hex;
use rand::rngs::OsRng;

use crate::farcaster::contracts::{
    contract_client::FarcasterContractClient,
    types::{ContractResult, Fid},
    types::{FidKeysInfo, SignerVerificationResult},
};

/// Generate a new Ed25519 key pair
pub fn generate_ed25519_keypair() -> SigningKey {
    let mut csprng = OsRng {};
    SigningKey::generate(&mut csprng)
}

impl FarcasterContractClient {
    /// Verify that a signer was successfully added to a FID
    pub async fn verify_signer_registration(
        &self,
        fid: Fid,
        expected_public_key: [u8; 32],
    ) -> Result<SignerVerificationResult> {
        // Check the key status in the registry
        let key_result = self
            .key_registry
            .keys(fid, expected_public_key.to_vec())
            .await?;

        match key_result {
            ContractResult::Success((state, key_type)) => {
                let is_active = state == 0;
                let is_correct_type = key_type == 1; // Ed25519
                let is_valid = is_active && is_correct_type;

                Ok(SignerVerificationResult {
                    found: true,
                    is_active,
                    is_correct_type,
                    is_valid,
                    state,
                    key_type,
                    message: if is_valid {
                        "Key is in Active state and correct type - registration successful!"
                            .to_string()
                    } else if state == 2 {
                        "Key is in Pending state - may need additional confirmation".to_string()
                    } else {
                        "Key is in Inactive state or incorrect type".to_string()
                    },
                })
            }
            ContractResult::Error(e) => Ok(SignerVerificationResult {
                found: false,
                is_active: false,
                is_correct_type: false,
                is_valid: false,
                state: 0,
                key_type: 0,
                message: format!("Key not found in registry: {}", e),
            }),
        }
    }

    /// Get detailed key information for a FID
    pub async fn get_fid_keys_detailed(&self, fid: Fid) -> Result<FidKeysInfo> {
        // Get key counts
        let fid_info = self.get_fid_info(fid).await?;

        let mut keys_info = FidKeysInfo {
            fid,
            custody: fid_info.custody,
            recovery: fid_info.recovery,
            active_keys: fid_info.active_keys,
            inactive_keys: fid_info.inactive_keys,
            pending_keys: fid_info.pending_keys,
            active_keys_list: Vec::new(),
            inactive_keys_list: Vec::new(),
            pending_keys_list: Vec::new(),
        };

        // Get detailed key information for each state
        for (state, _state_name) in [(0u8, "Active"), (1u8, "Inactive"), (2u8, "Pending")] {
            match self.key_registry.keys_of(fid, state).await {
                Ok(ContractResult::Success(keys)) => {
                    let keys_hex: Vec<String> = keys.iter().map(|k| hex::encode(k)).collect();
                    match state {
                        0 => keys_info.active_keys_list = keys_hex,
                        1 => keys_info.inactive_keys_list = keys_hex,
                        2 => keys_info.pending_keys_list = keys_hex,
                        _ => {}
                    }
                }
                Ok(ContractResult::Error(_e)) => {
                    // Handle error case - keys list will remain empty
                }
                Err(_) => {
                    // Handle error case - keys list will remain empty
                }
            }
        }

        Ok(keys_info)
    }

    /// Generate a unique Ed25519 keypair for a FID (ensures it doesn't already exist)
    pub async fn generate_unique_signing_key(
        &self,
        fid: Fid,
        max_attempts: u32,
    ) -> Result<SigningKey> {
        let mut attempts = 0;

        loop {
            let mut csprng = OsRng {};
            let signing_key = SigningKey::generate(&mut csprng);
            let public_key = signing_key.verifying_key().to_bytes();

            attempts += 1;

            // Check if this key already exists in the registry
            match self.key_registry.keys(fid, public_key.to_vec()).await? {
                ContractResult::Success((_state, _key_type)) => {
                    // Check if the key actually exists in the key lists
                    let existing_keys = self.key_registry.keys_of(fid, 1).await?;
                    let mut key_exists = false;
                    if let ContractResult::Success(keys) = existing_keys {
                        for existing_key in keys {
                            if existing_key == public_key.to_vec() {
                                key_exists = true;
                                break;
                            }
                        }
                    }

                    if key_exists {
                        if attempts >= max_attempts {
                            return Err(anyhow::anyhow!(
                                "Failed to generate unique key after {} attempts",
                                max_attempts
                            ));
                        }
                        continue;
                    } else {
                        // Key is unique (state=0, type=0 but not in key list means it doesn't exist)
                        return Ok(signing_key);
                    }
                }
                ContractResult::Error(_) => {
                    // Key doesn't exist, we can use it
                    return Ok(signing_key);
                }
            }
        }
    }

    /// Verify that a keypair is valid by testing signature/verification
    pub fn verify_keypair(&self, signing_key: &SigningKey, test_message: &[u8]) -> Result<bool> {
        let signature = Ed25519Signer::sign(signing_key, test_message);
        let is_valid = signing_key
            .verifying_key()
            .verify(test_message, &signature)
            .is_ok();
        Ok(is_valid)
    }

    /// Check if an address can perform key operations on a FID
    pub async fn can_manage_fid_keys(
        &self,
        address: ethers::types::Address,
        fid: Fid,
    ) -> Result<bool> {
        let fid_info = self.get_fid_info(fid).await?;
        Ok(address == fid_info.custody)
    }
}

use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};

/// Farcaster contract addresses on Optimism mainnet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub id_registry: Address,
    pub key_registry: Address,
    pub storage_registry: Address,
    pub id_gateway: Address,
    pub key_gateway: Address,
    pub bundler: Address,
    pub signed_key_request_validator: Address,
}

impl Default for ContractAddresses {
    fn default() -> Self {
        Self {
            // Farcaster contract addresses on Optimism mainnet (corrected from screenshot)
            id_registry: "0x00000000fc6c5f01fc30151999387bb99a9f489b"
                .parse()
                .unwrap(),
            key_registry: "0x00000000fc1237824fb747abde0ff18990e59b7e"
                .parse()
                .unwrap(),
            storage_registry: "0x00000000fcce7f938e7ae6d3c335bd6a1a7c593d"
                .parse()
                .unwrap(),
            id_gateway: "0x00000000fc25870c6ed6b6c7e41fb078b7656f69"
                .parse()
                .unwrap(),
            key_gateway: "0x00000000fc56947c7e7183f8ca4b62398caadf0b"
                .parse()
                .unwrap(),
            bundler: "0x00000000fc04c910a0b5fea33b03e0447ad0b0aa"
                .parse()
                .unwrap(),
            signed_key_request_validator: "0x00000000fc700472606ed4fa22623acf62c60553"
                .parse()
                .unwrap(),
        }
    }
}

/// Farcaster ID (FID) type
pub type Fid = u64;

/// Recovery address for Farcaster account
pub type RecoveryAddress = Address;

/// Key type for Farcaster keys
pub type KeyType = u32;

/// Key metadata for Farcaster keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub key_type: KeyType,
    pub key: Vec<u8>,
    pub metadata: Vec<u8>,
}

/// Storage units for Farcaster accounts
pub type StorageUnits = u32;

/// Storage price in wei
pub type StoragePrice = U256;

/// Result type for contract calls
#[derive(Debug, Clone)]
pub enum ContractResult<T> {
    Success(T),
    Error(String),
}

impl<T> ContractResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, ContractResult::Success(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ContractResult::Error(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            ContractResult::Success(value) => value,
            ContractResult::Error(msg) => panic!("Contract error: {msg}"),
        }
    }

    pub fn unwrap_or(self, default: T) -> T {
        match self {
            ContractResult::Success(value) => value,
            ContractResult::Error(_) => default,
        }
    }
}

/// Comprehensive FID information
#[derive(Debug, Clone)]
pub struct FidInfo {
    pub fid: Fid,
    pub custody: Address,
    pub recovery: Address,
    pub active_keys: u64,
    pub inactive_keys: u64,
    pub pending_keys: u64,
}

/// Network status information
#[derive(Debug, Clone)]
pub struct NetworkStatus {
    pub chain_id: u64,
    pub block_number: u64,
    pub id_gateway_paused: bool,
    pub key_gateway_paused: bool,
    pub storage_registry_paused: bool,
}

/// Result of signer verification
#[derive(Debug, Clone)]
pub struct SignerVerificationResult {
    pub found: bool,
    pub is_active: bool,
    pub is_correct_type: bool,
    pub is_valid: bool,
    pub state: u8,
    pub key_type: u32,
    pub message: String,
}

/// Detailed key information for a FID
#[derive(Debug, Clone)]
pub struct FidKeysInfo {
    pub fid: Fid,
    pub custody: Address,
    pub recovery: Address,
    pub active_keys: u64,
    pub inactive_keys: u64,
    pub pending_keys: u64,
    pub active_keys_list: Vec<String>,
    pub inactive_keys_list: Vec<String>,
    pub pending_keys_list: Vec<String>,
}

/// Result of security test for unauthorized key operations
#[derive(Debug, Clone)]
pub struct SecurityTestResult {
    pub target_fid: Fid,
    pub caller_address: Address,
    pub can_manage_keys: bool,
    pub unauthorized_add_failed: bool,
    pub unauthorized_remove_failed: bool,
    pub direct_remove_failed: bool,
    pub keys_unchanged: bool,
    pub error_messages: Vec<String>,
}

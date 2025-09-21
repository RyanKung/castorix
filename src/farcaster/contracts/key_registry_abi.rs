//! ABI-based KeyRegistry contract wrapper using generated bindings
//!
//! This module provides a type-safe wrapper around the KeyRegistry contract
//! using the automatically generated ABI bindings.

#![cfg(not(doctest))]

use crate::farcaster::contracts::{
    generated::keyregistry_bindings::KeyRegistry as KeyRegistryContract,
    types::{ContractResult, Fid},
};
use anyhow::Result;
use ethers::{
    providers::{Http, Provider},
    types::Address,
};

/// ABI-based KeyRegistry contract wrapper
#[derive(Clone)]
pub struct KeyRegistryAbi {
    contract: KeyRegistryContract<Provider<Http>>,
}

impl KeyRegistryAbi {
    /// Create a new KeyRegistryAbi instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        let contract = KeyRegistryContract::new(address, std::sync::Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the contract instance
    pub fn contract(&self) -> &KeyRegistryContract<Provider<Http>> {
        &self.contract
    }

    /// Get the total number of keys for a FID in a specific state
    pub async fn total_keys(&self, fid: Fid, state: u8) -> Result<ContractResult<u64>> {
        match self.contract.total_keys(fid.into(), state).call().await {
            Ok(total) => {
                let total_u64 = total.try_into().unwrap_or(0);
                Ok(ContractResult::Success(total_u64))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the key data for a specific key
    pub async fn key_data_of(&self, fid: Fid, key: Vec<u8>) -> Result<ContractResult<(u8, u32)>> {
        match self
            .contract
            .key_data_of(fid.into(), key.into())
            .call()
            .await
        {
            Ok(key_data) => {
                // Extract fields from the KeyData struct
                let state_u8 = key_data.state;
                let key_type_u32 = key_data.key_type;
                Ok(ContractResult::Success((state_u8, key_type_u32)))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the state and type of a specific key
    pub async fn keys(&self, fid: Fid, key: Vec<u8>) -> Result<ContractResult<(u8, u32)>> {
        match self.contract.keys(fid.into(), key.into()).call().await {
            Ok((state, key_type)) => {
                let state_u8 = state;
                let key_type_u32 = key_type;
                Ok(ContractResult::Success((state_u8, key_type_u32)))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the key at a specific index for a FID in a specific state
    pub async fn key_at(&self, fid: Fid, state: u8, index: u64) -> Result<ContractResult<Vec<u8>>> {
        match self
            .contract
            .key_at(fid.into(), state, index.into())
            .call()
            .await
        {
            Ok(key_bytes) => Ok(ContractResult::Success(key_bytes.to_vec())),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get all keys for a FID in a specific state
    pub async fn keys_of(&self, fid: Fid, state: u8) -> Result<ContractResult<Vec<Vec<u8>>>> {
        match self.contract.keys_of(fid.into(), state).call().await {
            Ok(keys) => {
                let keys_vec: Vec<Vec<u8>> = keys.into_iter().map(|k| k.to_vec()).collect();
                Ok(ContractResult::Success(keys_vec))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the ID Registry address
    pub async fn id_registry(&self) -> Result<ContractResult<Address>> {
        match self.contract.id_registry().call().await {
            Ok(address) => Ok(ContractResult::Success(address)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Check if the contract is paused
    pub async fn paused(&self) -> Result<ContractResult<bool>> {
        match self.contract.paused().call().await {
            Ok(is_paused) => Ok(ContractResult::Success(is_paused)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the grace period
    pub async fn grace_period(&self) -> Result<ContractResult<u64>> {
        match self.contract.grace_period().call().await {
            Ok(period) => {
                let period_u64: u64 = period.into();
                Ok(ContractResult::Success(period_u64))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Check if the gateway is frozen
    pub async fn gateway_frozen(&self) -> Result<ContractResult<bool>> {
        match self.contract.gateway_frozen().call().await {
            Ok(is_frozen) => Ok(ContractResult::Success(is_frozen)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the Key Gateway address
    pub async fn key_gateway(&self) -> Result<ContractResult<Address>> {
        match self.contract.key_gateway().call().await {
            Ok(gateway) => Ok(ContractResult::Success(gateway)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Remove a key (requires proper authorization)
    pub async fn remove(&self, key: Vec<u8>) -> Result<ContractResult<()>> {
        match self.contract.remove(key.into()).send().await {
            Ok(tx) => match tx.await {
                Ok(_receipt) => Ok(ContractResult::Success(())),
                Err(e) => Ok(ContractResult::Error(format!("Transaction failed: {e}"))),
            },
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Remove a key for another FID owner (requires authorization signature)
    pub async fn remove_for(
        &self,
        fid_owner: Address,
        key: Vec<u8>,
        deadline: u64,
        signature: Vec<u8>,
    ) -> Result<ContractResult<ethers::types::TransactionReceipt>> {
        match self
            .contract
            .remove_for(fid_owner, key.into(), deadline.into(), signature.into())
            .send()
            .await
        {
            Ok(pending_tx) => match pending_tx.await {
                Ok(Some(receipt)) => Ok(ContractResult::Success(receipt)),
                Ok(None) => Ok(ContractResult::Error(
                    "Transaction failed - no receipt received".to_string(),
                )),
                Err(e) => Ok(ContractResult::Error(format!("Transaction failed: {e}"))),
            },
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the migrator address
    pub async fn migrator(&self) -> Result<ContractResult<Address>> {
        match self.contract.migrator().call().await {
            Ok(migrator) => Ok(ContractResult::Success(migrator)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Check if the contract has been migrated
    pub async fn is_migrated(&self) -> Result<ContractResult<bool>> {
        match self.contract.is_migrated().call().await {
            Ok(is_migrated) => Ok(ContractResult::Success(is_migrated)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the migration timestamp
    pub async fn migrated_at(&self) -> Result<ContractResult<u64>> {
        match self.contract.migrated_at().call().await {
            Ok(timestamp) => {
                let timestamp_u64 = timestamp;
                Ok(ContractResult::Success(timestamp_u64))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the contract version
    pub async fn version(&self) -> Result<ContractResult<String>> {
        match self.contract.version().call().await {
            Ok(version) => Ok(ContractResult::Success(version)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }
}

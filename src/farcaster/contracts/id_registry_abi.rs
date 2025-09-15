//! ABI-based IdRegistry contract wrapper using generated bindings
//!
//! This module provides a type-safe wrapper around the IdRegistry contract
//! using the automatically generated ABI bindings.

#![cfg(not(doctest))]

use crate::farcaster::contracts::{
    generated::idregistry_bindings::IdRegistry as IdRegistryContract,
    types::{ContractResult, Fid, RecoveryAddress},
};
use anyhow::Result;
use ethers::{
    providers::{Http, Provider},
    types::Address,
};

/// ABI-based IdRegistry contract wrapper
#[derive(Clone)]
pub struct IdRegistryAbi {
    contract: IdRegistryContract<Provider<Http>>,
}

impl IdRegistryAbi {
    /// Create a new IdRegistryAbi instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        let contract = IdRegistryContract::new(address, std::sync::Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the contract instance
    pub fn contract(&self) -> &IdRegistryContract<Provider<Http>> {
        &self.contract
    }

    /// Get the custody address of a Farcaster ID (Farcaster's equivalent of ownerOf)
    pub async fn custody_of(&self, fid: Fid) -> Result<ContractResult<Address>> {
        match self.contract.custody_of(fid.into()).call().await {
            Ok(owner) => Ok(ContractResult::Success(owner)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the owner of a Farcaster ID (alias for custody_of for compatibility)
    pub async fn owner_of(&self, fid: Fid) -> Result<ContractResult<Address>> {
        self.custody_of(fid).await
    }

    /// Get the Farcaster ID of an owner
    pub async fn id_of(&self, owner: Address) -> Result<ContractResult<Fid>> {
        match self.contract.id_of(owner).call().await {
            Ok(fid) => {
                let fid_u64 = fid.try_into().unwrap_or(0);
                Ok(ContractResult::Success(fid_u64))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the recovery address for a Farcaster ID
    pub async fn recovery_of(&self, fid: Fid) -> Result<ContractResult<RecoveryAddress>> {
        match self.contract.recovery_of(fid.into()).call().await {
            Ok(recovery) => Ok(ContractResult::Success(recovery)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the current ID counter
    pub async fn id_counter(&self) -> Result<ContractResult<u64>> {
        match self.contract.id_counter().call().await {
            Ok(counter) => {
                let counter_u64 = counter.try_into().unwrap_or(0);
                Ok(ContractResult::Success(counter_u64))
            }
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

    /// Get the ID Gateway address
    pub async fn id_gateway(&self) -> Result<ContractResult<Address>> {
        match self.contract.id_gateway().call().await {
            Ok(gateway) => Ok(ContractResult::Success(gateway)),
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

    /// Get the contract name
    pub async fn name(&self) -> Result<ContractResult<String>> {
        match self.contract.name().call().await {
            Ok(name) => Ok(ContractResult::Success(name)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Verify a FID signature
    pub async fn verify_fid_signature(
        &self,
        custody_address: Address,
        fid: Fid,
        digest: [u8; 32],
        signature: Vec<u8>,
    ) -> Result<ContractResult<bool>> {
        let digest_bytes32 = digest;
        match self
            .contract
            .verify_fid_signature(
                custody_address,
                fid.into(),
                digest_bytes32,
                signature.into(),
            )
            .call()
            .await
        {
            Ok(is_valid) => Ok(ContractResult::Success(is_valid)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::providers::{Http, Provider};

    #[tokio::test]
    async fn test_id_registry_abi_creation() {
        // This test will fail without a valid RPC URL, but it demonstrates the API
        let provider = Provider::<Http>::try_from("https://www.optimism.io/").unwrap();
        let address = "0x00000000fc6c5f01fc30151999387bb99a9f489b"
            .parse::<Address>()
            .unwrap();

        let result = IdRegistryAbi::new(provider, address);
        assert!(result.is_ok());
    }
}

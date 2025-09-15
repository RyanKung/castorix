//! ABI-based StorageRegistry contract wrapper using generated bindings
//!
//! This module provides a type-safe wrapper around the StorageRegistry contract
//! using the automatically generated ABI bindings.

#![cfg(not(doctest))]

use crate::farcaster::contracts::{
    generated::storageregistry_bindings::StorageRegistry as StorageRegistryContract,
    types::ContractResult,
};
use anyhow::Result;
use ethers::{
    providers::{Http, Provider},
    types::{Address, U256},
};

/// Storage units type
pub type StorageUnits = u32;

/// ABI-based StorageRegistry contract wrapper
#[derive(Clone)]
pub struct StorageRegistryAbi {
    contract: StorageRegistryContract<Provider<Http>>,
}

impl StorageRegistryAbi {
    /// Create a new StorageRegistryAbi instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        let contract = StorageRegistryContract::new(address, std::sync::Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the contract instance
    pub fn contract(&self) -> &StorageRegistryContract<Provider<Http>> {
        &self.contract
    }

    /// Get the price for storage units
    pub async fn price(&self, units: StorageUnits) -> Result<ContractResult<U256>> {
        match self.contract.price(units.into()).call().await {
            Ok(price) => Ok(ContractResult::Success(price)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the unit price
    pub async fn unit_price(&self) -> Result<ContractResult<U256>> {
        match self.contract.unit_price().call().await {
            Ok(price) => Ok(ContractResult::Success(price)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the USD unit price
    pub async fn usd_unit_price(&self) -> Result<ContractResult<U256>> {
        match self.contract.usd_unit_price().call().await {
            Ok(price) => Ok(ContractResult::Success(price)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the maximum units
    pub async fn max_units(&self) -> Result<ContractResult<u64>> {
        match self.contract.max_units().call().await {
            Ok(units) => {
                let units_u64 = units.try_into().unwrap_or(0);
                Ok(ContractResult::Success(units_u64))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the rented units
    pub async fn rented_units(&self) -> Result<ContractResult<u64>> {
        match self.contract.rented_units().call().await {
            Ok(units) => {
                let units_u64 = units.try_into().unwrap_or(0);
                Ok(ContractResult::Success(units_u64))
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the ETH/USD price
    pub async fn eth_usd_price(&self) -> Result<ContractResult<U256>> {
        match self.contract.eth_usd_price().call().await {
            Ok(price) => Ok(ContractResult::Success(price)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the vault address
    pub async fn vault(&self) -> Result<ContractResult<Address>> {
        match self.contract.vault().call().await {
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


    /// Get the contract version
    pub async fn version(&self) -> Result<ContractResult<String>> {
        match self.contract.version().call().await {
            Ok(version) => Ok(ContractResult::Success(version)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }
}

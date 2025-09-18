//! ABI-based IdGateway contract wrapper using generated bindings
//!
//! This module provides a type-safe wrapper around the IdGateway contract
//! using the automatically generated ABI bindings.

#![cfg(not(doctest))]

use crate::farcaster::contracts::{
    generated::idgateway_bindings::IdGateway as IdGatewayContract, types::ContractResult,
};
use anyhow::Result;
use ethers::{
    providers::{Http, Provider},
    types::{Address, U256},
};

/// ABI-based IdGateway contract wrapper
#[derive(Clone)]
pub struct IdGatewayAbi {
    contract: IdGatewayContract<Provider<Http>>,
}

impl IdGatewayAbi {
    /// Create a new IdGatewayAbi instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        let contract = IdGatewayContract::new(address, std::sync::Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the contract instance
    pub fn contract(&self) -> &IdGatewayContract<Provider<Http>> {
        &self.contract
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

    /// Get the base price
    pub async fn price(&self) -> Result<ContractResult<U256>> {
        match self.contract.price().call().await {
            Ok(price) => Ok(ContractResult::Success(price)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the Storage Registry address
    pub async fn storage_registry(&self) -> Result<ContractResult<Address>> {
        match self.contract.storage_registry().call().await {
            Ok(address) => Ok(ContractResult::Success(address)),
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

    /// Register a new FID (requires wallet client for actual transaction)
    pub async fn register(&self, _recovery: Address) -> Result<ContractResult<(u64, U256)>> {
        // This would require a wallet client for actual transaction
        // For now, return an error indicating this is read-only
        Ok(ContractResult::Error(
            "Register requires wallet client - use wallet-based registration".to_string(),
        ))
    }

    /// Register a new FID with extra storage (requires wallet client for actual transaction)
    pub async fn register_with_storage(
        &self,
        _recovery: Address,
        _extra_storage: u64,
    ) -> Result<ContractResult<(u64, U256)>> {
        // This would require a wallet client for actual transaction
        // For now, return an error indicating this is read-only
        Ok(ContractResult::Error(
            "Register with storage requires wallet client - use wallet-based registration"
                .to_string(),
        ))
    }
}

//! ABI-based Bundler contract wrapper using generated bindings
//!
//! This module provides a type-safe wrapper around the Bundler contract
//! using the automatically generated ABI bindings.

#![cfg(not(doctest))]

use anyhow::Result;
use ethers::providers::Http;
use ethers::providers::Provider;
use ethers::types::Address;
use ethers::types::U256;

use crate::farcaster::contracts::generated::bundler_bindings::Bundler as BundlerContract;
use crate::farcaster::contracts::types::ContractResult;

/// ABI-based Bundler contract wrapper
#[derive(Clone)]
pub struct BundlerAbi {
    contract: BundlerContract<Provider<Http>>,
}

impl BundlerAbi {
    /// Create a new BundlerAbi instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        let contract = BundlerContract::new(address, std::sync::Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the price for extra storage
    pub async fn price(&self, extra_storage: u64) -> Result<ContractResult<U256>> {
        match self.contract.price(extra_storage.into()).call().await {
            Ok(price) => Ok(ContractResult::Success(price)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the ID Gateway address
    pub async fn id_gateway(&self) -> Result<ContractResult<Address>> {
        match self.contract.id_gateway().call().await {
            Ok(address) => Ok(ContractResult::Success(address)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the Key Gateway address
    pub async fn key_gateway(&self) -> Result<ContractResult<Address>> {
        match self.contract.key_gateway().call().await {
            Ok(address) => Ok(ContractResult::Success(address)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }
}

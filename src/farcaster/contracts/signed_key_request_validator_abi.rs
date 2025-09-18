use ethers::{providers::Middleware, types::Address};
use std::sync::Arc;

use crate::farcaster::contracts::generated::signedkeyrequestvalidator_bindings::SignedKeyRequestValidator as SignedKeyRequestValidatorContract;

/// ABI wrapper for SignedKeyRequestValidator contract
#[derive(Clone)]
pub struct SignedKeyRequestValidatorAbi<M> {
    pub contract: SignedKeyRequestValidatorContract<M>,
}

impl<M: Middleware + Clone> SignedKeyRequestValidatorAbi<M> {
    /// Create a new SignedKeyRequestValidatorAbi instance
    pub fn new(provider: M, address: Address) -> anyhow::Result<Self> {
        let contract = SignedKeyRequestValidatorContract::new(address, Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the contract instance
    pub fn contract(&self) -> &SignedKeyRequestValidatorContract<M> {
        &self.contract
    }
}

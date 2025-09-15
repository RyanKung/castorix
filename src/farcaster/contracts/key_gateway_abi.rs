//! ABI-based KeyGateway contract wrapper using generated bindings
//!
//! This module provides a type-safe wrapper around the KeyGateway contract
//! using the automatically generated ABI bindings.

#![cfg(not(doctest))]

use crate::farcaster::contracts::{
    generated::keygateway_bindings::KeyGateway as KeyGatewayContract,
    types::ContractResult,
};
use anyhow::Result;
use ethers::{
    providers::{Http, Provider},
    types::{Address, U256, Bytes},
    types::transaction::eip2718::TypedTransaction,
    middleware::Middleware,
};

/// ABI-based KeyGateway contract wrapper
#[derive(Clone)]
pub struct KeyGatewayAbi {
    contract: KeyGatewayContract<Provider<Http>>,
}

impl KeyGatewayAbi {
    /// Create a new KeyGatewayAbi instance
    pub fn new(provider: Provider<Http>, address: Address) -> Result<Self> {
        let contract = KeyGatewayContract::new(address, std::sync::Arc::new(provider));
        Ok(Self { contract })
    }

    /// Get the contract address
    pub fn address(&self) -> Address {
        self.contract.address()
    }

    /// Get the contract instance
    pub fn contract(&self) -> &KeyGatewayContract<Provider<Http>> {
        &self.contract
    }

    /// Get the Key Registry address
    pub async fn key_registry(&self) -> Result<ContractResult<Address>> {
        match self.contract.key_registry().call().await {
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

    /// Add a key for the caller (direct method)
    pub async fn add(
        &self,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        metadata: Vec<u8>,
    ) -> Result<ContractResult<()>> {
        match self.contract
            .add(key_type, key.into(), metadata_type, metadata.into())
            .call()
            .await
        {
            Ok(_) => Ok(ContractResult::Success(())),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Add a key for another address (authorized method)
    pub async fn add_for(
        &self,
        fid_owner: Address,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        metadata: Vec<u8>,
        deadline: U256,
        sig: Vec<u8>,
    ) -> Result<ContractResult<ethers::types::TransactionReceipt>> {
        match self.contract
            .add_for(
                fid_owner,
                key_type,
                key.into(),
                metadata_type,
                metadata.into(),
                deadline,
                sig.into(),
            )
            .send()
            .await
        {
            Ok(pending_tx) => {
                match pending_tx.await {
                    Ok(Some(receipt)) => Ok(ContractResult::Success(receipt)),
                    Ok(None) => Ok(ContractResult::Error("Transaction failed - no receipt received".to_string())),
                    Err(e) => Ok(ContractResult::Error(format!("Transaction failed: {e}"))),
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Add a key for another address using raw transaction (for third-party payment)
    pub async fn add_for_raw(
        &self,
        provider: &ethers::providers::Provider<ethers::providers::Http>,
        wallet: &ethers::signers::LocalWallet,
        fid_owner: Address,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        metadata: Vec<u8>,
        deadline: U256,
        sig: Vec<u8>,
    ) -> Result<ContractResult<ethers::types::TransactionReceipt>> {
        use ethers::signers::Signer;
        
        // Clone parameters to avoid move issues
        let key_clone = key.clone();
        let metadata_clone = metadata.clone();
        let sig_clone = sig.clone();
        
        // Build the transaction data
        let tx_data = self.contract
            .add_for(
                fid_owner,
                key_type,
                key.into(),
                metadata_type,
                metadata.into(),
                deadline,
                sig.into(),
            )
            .calldata()
            .ok_or_else(|| anyhow::anyhow!("Failed to get calldata"))?;

        // Get gas price and estimate gas
        let gas_price = provider.get_gas_price().await?;
        let gas_limit = self.contract
            .add_for(
                fid_owner,
                key_type,
                key_clone.into(),
                metadata_type,
                metadata_clone.into(),
                deadline,
                sig_clone.into(),
            )
            .estimate_gas()
            .await?;

        // Get nonce
        let nonce = provider.get_transaction_count(wallet.address(), None).await?;

        // Create transaction request
        let tx_request = ethers::types::TransactionRequest::new()
            .to(self.contract.address())
            .data(tx_data)
            .gas(gas_limit)
            .gas_price(gas_price)
            .nonce(nonce)
            .value(0u64);

        // Sign the transaction
        let chain_id = provider.get_chainid().await?;
        let wallet_with_chain_id = wallet.clone().with_chain_id(chain_id.as_u64());
        let typed_tx = TypedTransaction::Legacy(tx_request);
        let signature = wallet_with_chain_id.sign_transaction(&typed_tx).await?;
        
        // Create signed transaction bytes
        let signed_tx_bytes = typed_tx.rlp_signed(&signature);

        // Send raw transaction
        let tx_hash = provider.send_raw_transaction(signed_tx_bytes).await?;
        
        // Wait for receipt
        match tx_hash.await {
            Ok(Some(receipt)) => Ok(ContractResult::Success(receipt)),
            Ok(None) => Ok(ContractResult::Error("Transaction failed - no receipt received".to_string())),
            Err(e) => Ok(ContractResult::Error(format!("Transaction failed: {e}"))),
        }
    }

    /// Get nonce for an address
    pub async fn nonces(&self, owner: Address) -> Result<ContractResult<U256>> {
        match self.contract.nonces(owner).call().await {
            Ok(nonce) => Ok(ContractResult::Success(nonce)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Use nonce (increment and return new value)
    pub async fn use_nonce(&self) -> Result<ContractResult<U256>> {
        match self.contract.use_nonce().call().await {
            Ok(nonce) => Ok(ContractResult::Success(nonce)),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }

    /// Get the ADD_TYPEHASH
    pub async fn add_typehash(&self) -> Result<ContractResult<Bytes>> {
        match self.contract.add_typehash().call().await {
            Ok(typehash) => Ok(ContractResult::Success(typehash.into())),
            Err(e) => Ok(ContractResult::Error(format!("Contract call failed: {e}"))),
        }
    }
}

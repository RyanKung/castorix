use ethers::{
    providers::{Provider, Http, Middleware},
};
use anyhow::Result;

use crate::farcaster::contracts::{
    id_registry::IdRegistry,
    key_registry::KeyRegistry,
    storage_registry::StorageRegistry,
    id_gateway::IdGateway,
    key_gateway::KeyGateway,
    bundler::Bundler,
    types::ContractAddresses,
};

/// Main client for interacting with Farcaster contracts on Optimism
pub struct FarcasterContractClient {
    provider: Provider<Http>,
    addresses: ContractAddresses,
    id_registry: IdRegistry,
    key_registry: KeyRegistry,
    storage_registry: StorageRegistry,
    id_gateway: IdGateway,
    key_gateway: KeyGateway,
    bundler: Bundler,
}

impl FarcasterContractClient {
    /// Create a new FarcasterContractClient with custom addresses
    pub fn new(rpc_url: String, addresses: ContractAddresses) -> Result<Self> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        
        let id_registry = IdRegistry::new(provider.clone(), addresses.id_registry)?;
        let key_registry = KeyRegistry::new(provider.clone(), addresses.key_registry)?;
        let storage_registry = StorageRegistry::new(provider.clone(), addresses.storage_registry)?;
        let id_gateway = IdGateway::new(provider.clone(), addresses.id_gateway)?;
        let key_gateway = KeyGateway::new(provider.clone(), addresses.key_gateway)?;
        let bundler = Bundler::new(provider.clone(), addresses.bundler)?;
        
        Ok(Self {
            provider,
            addresses,
            id_registry,
            key_registry,
            storage_registry,
            id_gateway,
            key_gateway,
            bundler,
        })
    }
    
    /// Create a new FarcasterContractClient with default addresses
    pub fn new_with_default_addresses(rpc_url: String) -> Result<Self> {
        Self::new(rpc_url, ContractAddresses::default())
    }
    
    /// Create a new FarcasterContractClient from environment variables
    pub fn from_env() -> Result<Self> {
        let rpc_url = std::env::var("ETH_OP_RPC_URL")
            .map_err(|_| anyhow::anyhow!("ETH_OP_RPC_URL environment variable not set"))?;
        
        Self::new_with_default_addresses(rpc_url)
    }
    
    /// Get the provider
    pub fn provider(&self) -> &Provider<Http> {
        &self.provider
    }
    
    /// Get the contract addresses
    pub fn addresses(&self) -> &ContractAddresses {
        &self.addresses
    }
    
    /// Get the ID Registry contract
    pub fn id_registry(&self) -> &IdRegistry {
        &self.id_registry
    }
    
    /// Get the Key Registry contract
    pub fn key_registry(&self) -> &KeyRegistry {
        &self.key_registry
    }
    
    /// Get the Storage Registry contract
    pub fn storage_registry(&self) -> &StorageRegistry {
        &self.storage_registry
    }
    
    /// Get the ID Gateway contract
    pub fn id_gateway(&self) -> &IdGateway {
        &self.id_gateway
    }
    
    /// Get the Key Gateway contract
    pub fn key_gateway(&self) -> &KeyGateway {
        &self.key_gateway
    }
    
    /// Get the Bundler contract
    pub fn bundler(&self) -> &Bundler {
        &self.bundler
    }
    
    /// Get contract addresses as a map
    pub fn get_addresses_map(&self) -> std::collections::HashMap<String, String> {
        let mut addresses = std::collections::HashMap::new();
        addresses.insert("id_registry".to_string(), format!("{:?}", self.addresses.id_registry));
        addresses.insert("key_registry".to_string(), format!("{:?}", self.addresses.key_registry));
        addresses.insert("storage_registry".to_string(), format!("{:?}", self.addresses.storage_registry));
        addresses.insert("id_gateway".to_string(), format!("{:?}", self.addresses.id_gateway));
        addresses.insert("key_gateway".to_string(), format!("{:?}", self.addresses.key_gateway));
        addresses.insert("bundler".to_string(), format!("{:?}", self.addresses.bundler));
        addresses
    }
    
    /// Verify that all contracts are accessible
    pub async fn verify_contracts(&self) -> Result<ContractVerificationResult> {
        let mut results = ContractVerificationResult::new();
        
        // Test ID Registry - use a simple call instead
        match self.id_registry.owner_of(1).await {
            Ok(_) => results.id_registry = true,
            Err(e) => {
                results.id_registry = false;
                results.errors.push(format!("ID Registry error: {e}"));
            }
        }
        
        // Test Key Registry
        match self.key_registry.key_count_of(1).await {
            Ok(_) => results.key_registry = true,
            Err(e) => {
                results.key_registry = false;
                results.errors.push(format!("Key Registry error: {e}"));
            }
        }
        
        // Test Storage Registry
        match self.storage_registry.price_per_unit().await {
            Ok(_) => results.storage_registry = true,
            Err(e) => {
                results.storage_registry = false;
                results.errors.push(format!("Storage Registry error: {e}"));
            }
        }
        
        // Test ID Gateway
        match self.id_gateway.total_supply().await {
            Ok(_) => results.id_gateway = true,
            Err(e) => {
                results.id_gateway = false;
                results.errors.push(format!("ID Gateway error: {e}"));
            }
        }
        
        // Test Key Gateway
        match self.key_gateway.is_valid_key(1).await {
            Ok(_) => results.key_gateway = true,
            Err(e) => {
                results.key_gateway = false;
                results.errors.push(format!("Key Gateway error: {e}"));
            }
        }
        
        results.all_working = results.id_registry && results.key_registry && 
                             results.storage_registry && results.id_gateway && results.key_gateway;
        
        Ok(results)
    }
    
    /// Get network information
    pub async fn get_network_info(&self) -> Result<NetworkInfo> {
        let chain_id = self.provider.get_chainid().await?;
        let block_number = self.provider.get_block_number().await?;
        let gas_price = self.provider.get_gas_price().await?;
        
        Ok(NetworkInfo {
            chain_id: chain_id.as_u64(),
            block_number: block_number.as_u64(),
            gas_price,
        })
    }
}

/// Contract verification result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractVerificationResult {
    pub id_registry: bool,
    pub key_registry: bool,
    pub storage_registry: bool,
    pub id_gateway: bool,
    pub key_gateway: bool,
    pub all_working: bool,
    pub errors: Vec<String>,
}

impl ContractVerificationResult {
    fn new() -> Self {
        Self {
            id_registry: false,
            key_registry: false,
            storage_registry: false,
            id_gateway: false,
            key_gateway: false,
            all_working: false,
            errors: Vec::new(),
        }
    }
}

/// Network information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfo {
    pub chain_id: u64,
    pub block_number: u64,
    pub gas_price: ethers::types::U256,
}

/// Builder for FarcasterContractClient
pub struct FarcasterContractClientBuilder {
    rpc_url: Option<String>,
    addresses: Option<ContractAddresses>,
}

impl FarcasterContractClientBuilder {
    pub fn new() -> Self {
        Self {
            rpc_url: None,
            addresses: None,
        }
    }
    
    pub fn rpc_url(mut self, rpc_url: String) -> Self {
        self.rpc_url = Some(rpc_url);
        self
    }
    
    pub fn addresses(mut self, addresses: ContractAddresses) -> Self {
        self.addresses = Some(addresses);
        self
    }
    
    pub fn build(self) -> Result<FarcasterContractClient> {
        let rpc_url = self.rpc_url
            .ok_or_else(|| anyhow::anyhow!("RPC URL is required"))?;
        
        let addresses = self.addresses.unwrap_or_default();
        
        FarcasterContractClient::new(rpc_url, addresses)
    }
}

impl Default for FarcasterContractClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

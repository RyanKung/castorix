use anyhow::Result;
use ethers::{
    middleware::Middleware,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, H256, U256},
};
use hex;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::farcaster::contracts::{
    bundler_abi::BundlerAbi,
    id_gateway_abi::IdGatewayAbi,
    id_registry_abi::IdRegistryAbi,
    key_gateway_abi::KeyGatewayAbi,
    key_registry_abi::KeyRegistryAbi,
    nonce_manager::NonceRegistry,
    signed_key_request_validator_abi::SignedKeyRequestValidatorAbi,
    storage_registry_abi::StorageRegistryAbi,
    types::{ContractAddresses, ContractResult, Fid},
    types::{FidInfo, NetworkStatus},
};

// Global nonce registry shared across all FarcasterContractClient instances
static GLOBAL_NONCE_REGISTRY: OnceLock<Arc<tokio::sync::Mutex<NonceRegistry>>> = OnceLock::new();

/// Main client for interacting with Farcaster contracts on Optimism
#[derive(Clone)]
pub struct FarcasterContractClient {
    pub provider: Arc<Provider<Http>>,
    pub addresses: ContractAddresses,
    pub id_registry: IdRegistryAbi,
    pub key_registry: KeyRegistryAbi,
    pub storage_registry: StorageRegistryAbi,
    pub id_gateway: IdGatewayAbi,
    pub key_gateway: KeyGatewayAbi,
    pub bundler: BundlerAbi,
    pub signed_key_request_validator: SignedKeyRequestValidatorAbi<Provider<Http>>,
    pub wallet: Option<Arc<LocalWallet>>,
    pub nonce_registry: Arc<tokio::sync::Mutex<NonceRegistry>>,
}

impl FarcasterContractClient {
    /// Create a new FarcasterContractClient with custom addresses
    pub fn new(rpc_url: String, addresses: ContractAddresses) -> Result<Self> {
        let provider = Arc::new(Provider::<Http>::try_from(rpc_url)?);

        // Initialize global nonce registry if not already initialized
        let nonce_registry = GLOBAL_NONCE_REGISTRY.get_or_init(|| {
            Arc::new(tokio::sync::Mutex::new(NonceRegistry::new(
                (*provider).clone(),
            )))
        });

        let id_registry = IdRegistryAbi::new((*provider).clone(), addresses.id_registry)?;
        let key_registry = KeyRegistryAbi::new((*provider).clone(), addresses.key_registry)?;
        let storage_registry =
            StorageRegistryAbi::new((*provider).clone(), addresses.storage_registry)?;
        let id_gateway = IdGatewayAbi::new((*provider).clone(), addresses.id_gateway)?;
        let key_gateway = KeyGatewayAbi::new((*provider).clone(), addresses.key_gateway)?;
        let bundler = BundlerAbi::new((*provider).clone(), addresses.bundler)?;
        let signed_key_request_validator = SignedKeyRequestValidatorAbi::new(
            (*provider).clone(),
            addresses.signed_key_request_validator,
        )?;

        Ok(Self {
            provider: provider.clone(),
            addresses,
            id_registry,
            key_registry,
            storage_registry,
            id_gateway,
            key_gateway,
            bundler,
            signed_key_request_validator,
            wallet: None,
            nonce_registry: nonce_registry.clone(),
        })
    }

    /// Create a new FarcasterContractClient with wallet
    pub fn new_with_wallet(
        rpc_url: String,
        addresses: ContractAddresses,
        wallet: LocalWallet,
    ) -> Result<Self> {
        let mut client = Self::new(rpc_url, addresses)?;
        client.wallet = Some(Arc::new(wallet));
        Ok(client)
    }

    // ===== ACCESSOR METHODS =====

    /// Get the provider
    pub fn provider(&self) -> &Arc<Provider<Http>> {
        &self.provider
    }

    /// Get the wallet (if available)
    pub fn wallet(&self) -> Option<&Arc<LocalWallet>> {
        self.wallet.as_ref()
    }

    /// Check if client has a wallet
    pub fn has_wallet(&self) -> bool {
        self.wallet.is_some()
    }

    /// Get wallet address (if available)
    pub fn wallet_address(&self) -> Option<Address> {
        self.wallet.as_ref().map(|w| w.address())
    }

    /// Get contract addresses
    pub fn addresses(&self) -> &ContractAddresses {
        &self.addresses
    }

    // ===== HIGH-LEVEL FARCACTER FUNCTIONS =====

    /// Get comprehensive FID information
    pub async fn get_fid_info(&self, fid: Fid) -> Result<FidInfo> {
        // Get custody and recovery addresses
        let custody_result = self.id_registry.custody_of(fid).await?;
        let recovery_result = self.id_registry.recovery_of(fid).await?;

        let custody = match custody_result {
            ContractResult::Success(addr) => addr,
            ContractResult::Error(e) => {
                return Err(anyhow::anyhow!("Failed to get custody address: {}", e))
            }
        };

        let recovery = match recovery_result {
            ContractResult::Success(addr) => addr,
            ContractResult::Error(e) => {
                return Err(anyhow::anyhow!("Failed to get recovery address: {}", e))
            }
        };

        // Get key counts
        let active_keys = self.key_registry.total_keys(fid, 0).await?.unwrap_or(0);
        let inactive_keys = self.key_registry.total_keys(fid, 1).await?.unwrap_or(0);
        let pending_keys = self.key_registry.total_keys(fid, 2).await?.unwrap_or(0);

        Ok(FidInfo {
            fid,
            custody,
            recovery,
            active_keys,
            inactive_keys,
            pending_keys,
        })
    }

    /// Check if an address has an FID
    pub async fn address_has_fid(&self, address: Address) -> Result<Option<Fid>> {
        let result = self.id_registry.id_of(address).await?;
        match result {
            ContractResult::Success(fid) => {
                if fid == 0 {
                    Ok(None) // 0 means no FID
                } else {
                    Ok(Some(fid))
                }
            }
            ContractResult::Error(_) => Ok(None),
        }
    }

    /// Get registration price
    pub async fn get_registration_price(&self) -> Result<U256> {
        let result = self.id_gateway.price().await?;
        match result {
            ContractResult::Success(price) => Ok(price),
            ContractResult::Error(e) => {
                Err(anyhow::anyhow!("Failed to get registration price: {}", e))
            }
        }
    }

    /// Get storage rental price
    pub async fn get_storage_price(&self, units: u64) -> Result<U256> {
        let result = self.storage_registry.price(units as u32).await?;
        match result {
            ContractResult::Success(price) => Ok(price),
            ContractResult::Error(e) => Err(anyhow::anyhow!("Failed to get storage price: {}", e)),
        }
    }

    /// Get network status information
    pub async fn get_network_status(&self) -> Result<NetworkStatus> {
        let chain_id = self.provider.get_chainid().await?;
        let block_number = self.provider.get_block_number().await?;

        // Check if gateways are paused
        let id_gateway_paused = self.id_gateway.paused().await?.unwrap_or(false);
        let key_gateway_paused = self.key_gateway.paused().await?.unwrap_or(false);
        let storage_registry_paused = self.storage_registry.paused().await?.unwrap_or(false);

        Ok(NetworkStatus {
            chain_id: chain_id.as_u64(),
            block_number: block_number.as_u64(),
            id_gateway_paused,
            key_gateway_paused,
            storage_registry_paused,
        })
    }

    /// Register a new FID (requires wallet)
    pub async fn register_fid(&self, recovery: Address) -> Result<ContractResult<(u64, U256)>> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for FID registration"))?;

        // Get registration price
        let price = self.get_registration_price().await?;

        // Get chain ID and create signer middleware
        let chain_id = self.provider.get_chainid().await?;
        let wallet_with_chain_id = wallet.as_ref().clone().with_chain_id(chain_id.as_u64());

        // Get current nonce to avoid nonce conflicts
        let nonce = self.get_next_nonce(wallet.address()).await?;
        println!("   📝 Using nonce: {}", nonce);

        let signer_middleware = SignerMiddleware::new(self.provider.clone(), wallet_with_chain_id);

        // Create the contract instance with signer middleware
        let contract = self.id_gateway.contract().clone();
        let wallet_contract = contract.connect(Arc::new(signer_middleware));

        // Call register function using ethers call method with explicit nonce
        let call = wallet_contract.method::<_, (U256, U256)>("register", recovery)?;
        match call.value(price).nonce(nonce).send().await {
            Ok(tx) => {
                let receipt = tx.await?;
                match receipt {
                    Some(receipt) => {
                        // Try to extract FID from transaction receipt
                        match self.extract_fid_from_receipt(&receipt) {
                            Ok(fid) => {
                                let overpayment = U256::zero(); // For now, return 0 as overpayment
                                Ok(ContractResult::Success((fid, overpayment)))
                            }
                            Err(_) => {
                                // If event parsing fails, query the contract to get the FID
                                println!(
                                    "   🔍 Event parsing failed, querying contract for FID..."
                                );
                                match self.get_fid_by_address(recovery).await {
                                    Ok(fid) => {
                                        println!("   ✅ Found FID by querying contract: {}", fid);
                                        let overpayment = U256::zero();
                                        Ok(ContractResult::Success((fid, overpayment)))
                                    }
                                    Err(e) => Ok(ContractResult::Error(format!(
                                        "Could not determine FID: {}",
                                        e
                                    ))),
                                }
                            }
                        }
                    }
                    None => Ok(ContractResult::Error(
                        "Transaction failed - no receipt".to_string(),
                    )),
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Registration failed: {}", e))),
        }
    }

    /// Register a new FID with extra storage (requires wallet)
    pub async fn register_fid_with_storage(
        &self,
        recovery: Address,
        extra_storage: u64,
    ) -> Result<ContractResult<(u64, U256)>> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for FID registration"))?;

        // Get registration price with extra storage
        let price = self
            .storage_registry
            .price(1 + extra_storage as u32)
            .await?;
        let price = match price {
            ContractResult::Success(p) => p,
            ContractResult::Error(e) => {
                return Ok(ContractResult::Error(format!("Failed to get price: {}", e)))
            }
        };

        // Get chain ID and create signer middleware
        let chain_id = self.provider.get_chainid().await?;
        let wallet_with_chain_id = wallet.as_ref().clone().with_chain_id(chain_id.as_u64());

        // Get current nonce to avoid nonce conflicts
        let nonce = self.get_next_nonce(wallet.address()).await?;
        println!("   📝 Using nonce: {}", nonce);

        let signer_middleware = SignerMiddleware::new(self.provider.clone(), wallet_with_chain_id);

        // Create the contract instance with signer middleware
        let contract = self.id_gateway.contract().clone();
        let wallet_contract = contract.connect(Arc::new(signer_middleware));

        // Call register function with extra storage using ethers call method with explicit nonce
        let call = wallet_contract
            .method::<_, (U256, U256)>("register", (recovery, U256::from(extra_storage)))?;
        match call.value(price).nonce(nonce).send().await {
            Ok(tx) => {
                let receipt = tx.await?;
                match receipt {
                    Some(receipt) => {
                        // Parse the return values from the transaction receipt
                        let fid = self.extract_fid_from_receipt(&receipt)?;
                        let overpayment = U256::zero(); // For now, return 0 as overpayment
                        Ok(ContractResult::Success((fid, overpayment)))
                    }
                    None => Ok(ContractResult::Error(
                        "Transaction failed - no receipt".to_string(),
                    )),
                }
            }
            Err(e) => Ok(ContractResult::Error(format!("Registration failed: {}", e))),
        }
    }

    /// Rent storage for a FID (requires wallet)
    pub async fn rent_storage(&self, fid: Fid, units: u64) -> Result<ContractResult<U256>> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for storage rental"))?;

        // Get storage price
        let price = self.get_storage_price(units).await?;

        // Get chain ID and create signer middleware
        let chain_id = self.provider.get_chainid().await?;
        let wallet_with_chain_id = wallet.as_ref().clone().with_chain_id(chain_id.as_u64());

        // Get current nonce to avoid nonce conflicts
        let nonce = self.get_next_nonce(wallet.address()).await?;
        println!("   📝 Using nonce: {}", nonce);

        let signer_middleware = SignerMiddleware::new(self.provider.clone(), wallet_with_chain_id);

        // Create the contract instance with signer middleware
        let contract = self.storage_registry.contract().clone();
        let wallet_contract = contract.connect(Arc::new(signer_middleware));

        // Call rent function using ethers call method with explicit nonce
        let call = wallet_contract.method::<_, U256>("rent", (fid, units as u32))?;
        match call.value(price).nonce(nonce).send().await {
            Ok(tx) => {
                let receipt = tx.await?;
                match receipt {
                    Some(receipt) => {
                        // Parse the return values from the transaction receipt
                        let overpayment = self.extract_overpayment_from_receipt(&receipt)?;
                        Ok(ContractResult::Success(overpayment))
                    }
                    None => Ok(ContractResult::Error(
                        "Transaction failed - no receipt".to_string(),
                    )),
                }
            }
            Err(e) => Ok(ContractResult::Error(format!(
                "Storage rental failed: {}",
                e
            ))),
        }
    }

    /// Extract FID from transaction receipt
    fn extract_fid_from_receipt(&self, receipt: &ethers::types::TransactionReceipt) -> Result<u64> {
        println!(
            "🔍 Looking for Register event in {} logs...",
            receipt.logs.len()
        );

        // Try different possible event signatures
        let possible_signatures = vec![
            (
                "Register(uint256,address,address)",
                ethers::utils::keccak256("Register(uint256,address,address)"),
            ),
            (
                "Register(address,address,uint256)",
                ethers::utils::keccak256("Register(address,address,uint256)"),
            ),
            (
                "Register(address,address)",
                ethers::utils::keccak256("Register(address,address)"),
            ),
            (
                "IdRegistered(uint256,address,address)",
                ethers::utils::keccak256("IdRegistered(uint256,address,address)"),
            ),
            (
                "IdRegistered(address,address,uint256)",
                ethers::utils::keccak256("IdRegistered(address,address,uint256)"),
            ),
            (
                "Transfer(address,address,uint256)",
                ethers::utils::keccak256("Transfer(address,address,uint256)"),
            ),
        ];

        for (i, log) in receipt.logs.iter().enumerate() {
            println!(
                "   Log {}: topics={}, data={}",
                i,
                log.topics.len(),
                log.data.len()
            );
            if log.topics.len() > 0 {
                println!("   Topic 0: {}", log.topics[0]);
            }

            if log.topics.len() >= 2 {
                let topic0 = log.topics[0];

                // Check against all possible signatures
                for (sig_name, expected_hash) in &possible_signatures {
                    let expected_signature = ethers::types::H256::from(*expected_hash);
                    println!(
                        "   Checking {}: {} == {}",
                        sig_name, topic0, expected_signature
                    );

                    if topic0 == expected_signature {
                        println!("   ✅ Found matching event: {}!", sig_name);

                        // Try to extract FID from different positions
                        if sig_name.contains("uint256") {
                            // For events with uint256, try different topic positions
                            for topic_idx in 1..log.topics.len() {
                                let fid_bytes = &log.topics[topic_idx].as_bytes()[12..];
                                let fid = U256::from_big_endian(fid_bytes).as_u64();
                                if fid > 0 && fid < 10000000 {
                                    // Reasonable FID range
                                    println!(
                                        "   📝 Extracted FID from topic {}: {}",
                                        topic_idx, fid
                                    );
                                    return Ok(fid);
                                }
                            }
                        }
                    }
                }
            }
        }

        // If no event found, try to get FID from the actual registered address
        // by calling the contract to see what FID was assigned
        println!("   ⚠️  No matching event found, will query contract for FID...");

        // For now, we'll return an error and handle this in the calling code
        Err(anyhow::anyhow!(
            "Could not extract FID from transaction receipt - no matching events found"
        ))
    }

    /// Extract overpayment from transaction receipt
    fn extract_overpayment_from_receipt(
        &self,
        _receipt: &ethers::types::TransactionReceipt,
    ) -> Result<U256> {
        // For now, return 0 as overpayment
        // In a real implementation, we would parse the transaction logs to get the actual overpayment
        Ok(U256::zero())
    }

    /// Get FID by address by querying the IdRegistry contract
    async fn get_fid_by_address(&self, address: Address) -> Result<u64> {
        // Query the IdRegistry contract to get the FID for this address
        // The IdRegistry has a mapping from address to FID
        let call = self
            .id_registry
            .contract()
            .method::<_, U256>("idOf", address)?;
        match call.call().await {
            Ok(fid) => {
                let fid_u64 = fid.as_u64();
                if fid_u64 > 0 {
                    Ok(fid_u64)
                } else {
                    Err(anyhow::anyhow!("No FID found for address"))
                }
            }
            Err(e) => Err(anyhow::anyhow!("Failed to query FID: {}", e)),
        }
    }

    /// Get the next nonce for a wallet using NonceManager
    async fn get_next_nonce(&self, address: Address) -> Result<U256> {
        let mut registry = self.nonce_registry.lock().await;
        registry.get_next_nonce(address).await
    }

    /// Fund a wallet using NonceManager for nonce management
    pub async fn fund_wallet(&self, target_address: Address, amount: U256) -> Result<H256> {
        let funder_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"; // Account 0
        let funder_wallet = LocalWallet::from_str(funder_key)?;
        let funder_address = funder_wallet.address();
        let chain_id = self.provider.get_chainid().await?;
        let funder_with_chain_id = funder_wallet.with_chain_id(chain_id.as_u64());

        // Get next nonce for funder using NonceManager
        let nonce = self.get_next_nonce(funder_address).await?;
        println!("   🔧 Using nonce {} for funder {}", nonce, funder_address);

        let fund_tx = TransactionRequest::new()
            .to(target_address)
            .value(amount)
            .gas(21000)
            .nonce(nonce);

        let signer_middleware = SignerMiddleware::new(self.provider.clone(), funder_with_chain_id);
        let pending_tx = signer_middleware.send_transaction(fund_tx, None).await?;
        let receipt = pending_tx.await?;

        match receipt {
            Some(receipt) => {
                println!("   ✅ Wallet funded successfully!");
                println!("   Transaction Hash: {:?}", receipt.transaction_hash);
                Ok(receipt.transaction_hash)
            }
            None => Err(anyhow::anyhow!("Funding transaction failed")),
        }
    }

    /// Wait for transaction confirmation to ensure nonce is updated

    /// Register a signer key for the current wallet's FID (requires wallet)
    pub async fn register_signer_key(
        &self,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        _metadata: Vec<u8>,
    ) -> Result<ContractResult<()>> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for signer registration"))?;

        // Get the FID for the current wallet address
        let fid = match self.address_has_fid(wallet.address()).await? {
            Some(fid) => fid,
            None => {
                return Ok(ContractResult::Error(
                    "Wallet address does not have a FID".to_string(),
                ))
            }
        };

        // Create deadline (1 hour from now)
        let deadline = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs()
            + 3600;

        // Get the FID owner address (custody address)
        let fid_owner = match self.get_fid_custody(fid).await? {
            Some(owner) => {
                println!("   FID {} owner: {}", fid, owner);
                owner
            }
            None => return Ok(ContractResult::Error("FID not found".to_string())),
        };

        // Assert that the current wallet is the FID owner
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No wallet available"))?;
        assert_eq!(
            wallet.address(),
            fid_owner,
            "Current wallet address {} must be the FID owner {} to sign the request",
            wallet.address(),
            fid_owner
        );
        println!("   ✅ Wallet address matches FID owner - can proceed with signing");

        // Create EIP-712 signature for SignedKeyRequest using SignedKeyRequestValidator
        let signature = self
            .create_signed_key_request_signature(fid, fid_owner, &key, deadline)
            .await?;

        // Create SignedKeyRequestMetadata using the signature
        let signed_key_request_metadata = self
            .create_signed_key_request_metadata(fid, fid_owner, &key, deadline, signature)
            .await?;

        // Use addFor method to pass fidOwner correctly
        println!("   Calling key_gateway.addFor with:");
        println!("     fid_owner: {}", fid_owner);
        println!("     key_type: {}", key_type);
        println!("     key: {}", hex::encode(&key));
        println!("     metadata_type: {}", metadata_type);
        println!(
            "     metadata length: {}",
            signed_key_request_metadata.len()
        );
        println!("     deadline: {}", deadline);

        // Create EIP-712 signature for KeyGateway.addFor
        let add_for_signature = self
            .create_add_for_signature(
                fid_owner,
                key_type,
                &key,
                metadata_type,
                &signed_key_request_metadata,
                deadline,
            )
            .await?;

        let result = self
            .key_gateway
            .add_for(
                fid_owner,
                key_type,
                key,
                metadata_type,
                signed_key_request_metadata,
                deadline.into(),
                add_for_signature,
            )
            .await?;

        match result {
            ContractResult::Success(_receipt) => {
                println!("   ✅ Signer key registered successfully!");
                Ok(ContractResult::Success(()))
            }
            ContractResult::Error(e) => {
                println!("   ❌ Signer registration failed: {}", e);
                Ok(ContractResult::Error(format!(
                    "Signer registration failed: {}",
                    e
                )))
            }
        }
    }

    /// Register a signer key for a specific FID owner (third-party registration)
    /// This allows Wallet B to pay gas for Wallet A's signer registration
    pub async fn register_signer_key_for_fid_owner(
        &self,
        fid_owner_address: ethers::types::Address,
        fid: u64,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        _metadata: Vec<u8>,
    ) -> Result<ContractResult<()>> {
        println!(
            "🔑 Registering signer key for FID owner {} (FID: {})",
            fid_owner_address, fid
        );

        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for signer registration"))?;

        // Verify that the FID owner address has the specified FID
        let fid_owner_fid = match self.get_fid_for_address(fid_owner_address).await? {
            Some(owner_fid) => owner_fid,
            None => {
                return Ok(ContractResult::Error(
                    "FID owner address does not have a FID".to_string(),
                ))
            }
        };

        if fid_owner_fid != fid {
            return Ok(ContractResult::Error(format!(
                "FID owner address {} has FID {}, but expected FID {}",
                fid_owner_address, fid_owner_fid, fid
            )));
        }

        // Create deadline (1 hour from now)
        let deadline = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs()
            + 3600;

        println!("   FID {} owner: {}", fid, fid_owner_address);
        println!("   Current wallet (gas payer): {}", wallet.address());
        println!("   ✅ Third-party registration authorized - current wallet will pay gas");

        // Create EIP-712 signature for SignedKeyRequest using SignedKeyRequestValidator
        // Note: We need to create a temporary client with the FID owner's wallet to sign
        // For this test, we'll assume the FID owner has pre-signed the request
        // In a real implementation, this would be done off-chain by the FID owner

        // For now, let's create a mock signature (in real implementation, this would come from FID owner)
        let signature = vec![0u8; 65]; // Mock signature - in real implementation, this would be from FID owner

        // Create SignedKeyRequestMetadata using the signature
        let signed_key_request_metadata = self
            .create_signed_key_request_metadata(fid, fid_owner_address, &key, deadline, signature)
            .await?;

        // Use addFor method to pass fidOwner correctly
        println!("   Calling key_gateway.addFor with:");
        println!("     fid_owner: {}", fid_owner_address);
        println!("     key_type: {}", key_type);
        println!("     key: {}", hex::encode(&key));
        println!("     metadata_type: {}", metadata_type);
        println!(
            "     metadata length: {}",
            signed_key_request_metadata.len()
        );
        println!("     deadline: {}", deadline);

        // Create EIP-712 signature for KeyGateway.addFor using current wallet (gas payer)
        let add_for_signature = self
            .create_add_for_signature(
                fid_owner_address,
                key_type,
                &key,
                metadata_type,
                &signed_key_request_metadata,
                deadline,
            )
            .await?;

        let result = self
            .key_gateway
            .add_for(
                fid_owner_address,
                key_type,
                key,
                metadata_type,
                signed_key_request_metadata,
                deadline.into(),
                add_for_signature,
            )
            .await?;

        match result {
            ContractResult::Success(_receipt) => {
                println!("   ✅ Third-party signer key registered successfully!");
                Ok(ContractResult::Success(()))
            }
            ContractResult::Error(e) => {
                println!("   ❌ Third-party signer registration failed: {}", e);
                Ok(ContractResult::Error(format!(
                    "Third-party signer registration failed: {}",
                    e
                )))
            }
        }
    }

    /// Register a signer key with pre-generated metadata (for third-party registration)
    pub async fn register_signer_key_with_metadata(
        &self,
        fid_owner_address: ethers::types::Address,
        fid: u64,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        metadata: Vec<u8>,
        deadline: u64,
    ) -> Result<ContractResult<()>> {
        println!(
            "🔑 Registering signer key with pre-generated metadata for FID owner {} (FID: {})",
            fid_owner_address, fid
        );

        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for signer registration"))?;

        // Verify that the FID owner address has the specified FID
        let fid_owner_fid = match self.get_fid_for_address(fid_owner_address).await? {
            Some(owner_fid) => owner_fid,
            None => {
                return Ok(ContractResult::Error(
                    "FID owner address does not have a FID".to_string(),
                ))
            }
        };

        if fid_owner_fid != fid {
            return Ok(ContractResult::Error(format!(
                "FID owner address {} has FID {}, but expected FID {}",
                fid_owner_address, fid_owner_fid, fid
            )));
        }

        println!("   FID {} owner: {}", fid, fid_owner_address);
        println!("   Current wallet (gas payer): {}", wallet.address());
        println!("   ✅ Third-party registration authorized - current wallet will pay gas");

        // Use addFor method to pass fidOwner correctly
        println!("   Calling key_gateway.addFor with:");
        println!("     fid_owner: {}", fid_owner_address);
        println!("     key_type: {}", key_type);
        println!("     key: {}", hex::encode(&key));
        println!("     metadata_type: {}", metadata_type);
        println!("     metadata length: {}", metadata.len());
        println!("     deadline: {}", deadline);

        // Create EIP-712 signature for KeyGateway.addFor using current wallet (gas payer)
        let add_for_signature = self
            .create_add_for_signature(
                fid_owner_address,
                key_type,
                &key,
                metadata_type,
                &metadata,
                deadline,
            )
            .await?;

        let result = self
            .key_gateway
            .add_for(
                fid_owner_address,
                key_type,
                key,
                metadata_type,
                metadata,
                deadline.into(),
                add_for_signature,
            )
            .await?;

        match result {
            ContractResult::Success(_receipt) => {
                println!("   ✅ Third-party signer key registered successfully!");
                Ok(ContractResult::Success(()))
            }
            ContractResult::Error(e) => {
                println!("   ❌ Third-party signer registration failed: {}", e);
                Ok(ContractResult::Error(format!(
                    "Third-party signer registration failed: {}",
                    e
                )))
            }
        }
    }

    /// Submit signer registration with pre-generated signatures (for third-party gas payment)
    pub async fn submit_signer_registration_with_signatures(
        &self,
        fid_owner_address: ethers::types::Address,
        fid: u64,
        key_type: u32,
        key: Vec<u8>,
        metadata_type: u8,
        metadata: Vec<u8>,
        deadline: u64,
        add_for_signature: Vec<u8>,
    ) -> Result<ContractResult<()>> {
        println!("🔑 Submitting signer registration with pre-generated signatures for FID owner {} (FID: {})", fid_owner_address, fid);

        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wallet required for signer registration"))?;

        // Verify that the FID owner address has the specified FID
        let fid_owner_fid = match self.get_fid_for_address(fid_owner_address).await? {
            Some(owner_fid) => owner_fid,
            None => {
                return Ok(ContractResult::Error(
                    "FID owner address does not have a FID".to_string(),
                ))
            }
        };

        if fid_owner_fid != fid {
            return Ok(ContractResult::Error(format!(
                "FID owner address {} has FID {}, but expected FID {}",
                fid_owner_address, fid_owner_fid, fid
            )));
        }

        println!("   FID {} owner: {}", fid, fid_owner_address);
        println!("   Current wallet (gas payer): {}", wallet.address());
        println!("   ✅ Third-party registration authorized - current wallet will pay gas");

        // Use addFor method with pre-generated signature
        println!("   Calling key_gateway.addFor with pre-generated signature:");
        println!("     fid_owner: {}", fid_owner_address);
        println!("     key_type: {}", key_type);
        println!("     key: {}", hex::encode(&key));
        println!("     metadata_type: {}", metadata_type);
        println!("     metadata length: {}", metadata.len());
        println!("     deadline: {}", deadline);

        let result = self
            .key_gateway
            .add_for(
                fid_owner_address,
                key_type,
                key,
                metadata_type,
                metadata,
                deadline.into(),
                add_for_signature,
            )
            .await?;

        match result {
            ContractResult::Success(_receipt) => {
                println!("   ✅ Third-party signer key registered successfully!");
                Ok(ContractResult::Success(()))
            }
            ContractResult::Error(e) => {
                println!("   ❌ Third-party signer registration failed: {}", e);
                Ok(ContractResult::Error(format!(
                    "Third-party signer registration failed: {}",
                    e
                )))
            }
        }
    }

    /// Get the FID for a specific address
    async fn get_fid_for_address(&self, address: ethers::types::Address) -> Result<Option<u64>> {
        match self.id_registry.id_of(address).await? {
            ContractResult::Success(fid) => Ok(Some(fid)),
            ContractResult::Error(_) => Ok(None),
        }
    }

    /// Get the custody address for a FID
    async fn get_fid_custody(&self, fid: u64) -> Result<Option<Address>> {
        match self.id_registry.custody_of(fid.into()).await? {
            ContractResult::Success(custody) => Ok(Some(custody)),
            ContractResult::Error(_) => Ok(None),
        }
    }

    /// Create EIP-712 signature for KeyGateway.addFor
    pub async fn create_add_for_signature(
        &self,
        fid_owner: ethers::types::Address,
        key_type: u32,
        key: &[u8],
        metadata_type: u8,
        metadata: &[u8],
        deadline: u64,
    ) -> Result<Vec<u8>> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No wallet available"))?;

        // Get chain ID and contract address
        let chain_id = self.provider.get_chainid().await?.as_u64();
        let gateway_address = self.addresses.key_gateway;

        // Get current nonce for the fid_owner
        let nonce_result = self.key_gateway.nonces(fid_owner).await?;
        let nonce = match nonce_result {
            ContractResult::Success(nonce) => nonce.as_u64(),
            ContractResult::Error(e) => return Err(anyhow::anyhow!("Failed to get nonce: {}", e)),
        };

        // Create the EIP-712 typed data structure for Add
        let typed_data = self.create_add_typed_data(
            fid_owner,
            key_type,
            key,
            metadata_type,
            metadata,
            nonce,
            deadline,
            gateway_address,
            chain_id,
        )?;

        // Sign the typed data using EIP-712
        let signature = wallet.sign_typed_data(&typed_data).await?;

        // Return the signature as bytes
        Ok(signature.to_vec())
    }

    /// Create EIP-712 signature for SignedKeyRequest using SignedKeyRequestValidator
    pub async fn create_signed_key_request_signature(
        &self,
        fid: u64,
        _fid_owner: ethers::types::Address,
        public_key: &[u8],
        deadline: u64,
    ) -> Result<Vec<u8>> {
        let wallet = self
            .wallet
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No wallet available"))?;

        // Get chain ID and contract address
        let chain_id = self.provider.get_chainid().await?.as_u64();
        let validator_address = self.addresses.signed_key_request_validator;

        // Create the EIP-712 typed data structure for SignedKeyRequest
        let typed_data = self.create_signed_key_request_typed_data(
            fid,
            public_key,
            deadline,
            validator_address,
            chain_id,
        )?;

        // Sign the typed data using EIP-712
        let signature = wallet.sign_typed_data(&typed_data).await?;

        // Return the signature as bytes
        Ok(signature.to_vec())
    }

    /// Create SignedKeyRequestMetadata using the signature
    pub async fn create_signed_key_request_metadata(
        &self,
        fid: u64,
        fid_owner: ethers::types::Address,
        _public_key: &[u8],
        deadline: u64,
        signature: Vec<u8>,
    ) -> Result<Vec<u8>> {
        use crate::farcaster::contracts::generated::signedkeyrequestvalidator_bindings::SignedKeyRequestMetadata;
        use ethers::types::{Bytes, U256};

        // Create the metadata struct with the provided signature
        let metadata_struct = SignedKeyRequestMetadata {
            request_fid: U256::from(fid),
            request_signer: fid_owner,
            signature: Bytes::from(signature),
            deadline: U256::from(deadline),
        };

        // Encode the metadata using the contract's encodeMetadata function
        let encoded_metadata = self
            .signed_key_request_validator
            .contract()
            .encode_metadata(metadata_struct)
            .await?;

        Ok(encoded_metadata.to_vec())
    }

    /// Create EIP-712 typed data for SignedKeyRequest
    fn create_signed_key_request_typed_data(
        &self,
        fid: u64,
        public_key: &[u8],
        deadline: u64,
        validator_address: ethers::types::Address,
        chain_id: u64,
    ) -> Result<ethers::types::transaction::eip712::TypedData> {
        use ethers::types::transaction::eip712::{EIP712Domain, Eip712DomainType, TypedData};
        use std::collections::BTreeMap;

        // Create domain separator
        let domain = EIP712Domain {
            name: Some("Farcaster SignedKeyRequestValidator".to_string()),
            version: Some("1".to_string()),
            chain_id: Some(ethers::types::U256::from(chain_id)),
            verifying_contract: Some(validator_address),
            salt: None,
        };

        // Create type definitions
        let mut types = BTreeMap::new();

        // Add EIP712Domain type
        types.insert(
            "EIP712Domain".to_string(),
            vec![
                Eip712DomainType {
                    name: "name".to_string(),
                    r#type: "string".to_string(),
                },
                Eip712DomainType {
                    name: "version".to_string(),
                    r#type: "string".to_string(),
                },
                Eip712DomainType {
                    name: "chainId".to_string(),
                    r#type: "uint256".to_string(),
                },
                Eip712DomainType {
                    name: "verifyingContract".to_string(),
                    r#type: "address".to_string(),
                },
            ],
        );

        // Add SignedKeyRequest type - key should be bytes, not bytes32
        types.insert(
            "SignedKeyRequest".to_string(),
            vec![
                Eip712DomainType {
                    name: "requestFid".to_string(),
                    r#type: "uint256".to_string(),
                },
                Eip712DomainType {
                    name: "key".to_string(),
                    r#type: "bytes".to_string(),
                },
                Eip712DomainType {
                    name: "deadline".to_string(),
                    r#type: "uint256".to_string(),
                },
            ],
        );

        // Create message data - key should be the raw public key bytes
        let mut message = BTreeMap::new();
        message.insert(
            "requestFid".to_string(),
            serde_json::Value::String(fid.to_string()),
        );
        message.insert(
            "key".to_string(),
            serde_json::Value::String(format!("0x{}", hex::encode(public_key))),
        );
        message.insert(
            "deadline".to_string(),
            serde_json::Value::String(deadline.to_string()),
        );

        Ok(TypedData {
            domain,
            types,
            primary_type: "SignedKeyRequest".to_string(),
            message,
        })
    }

    /// Create EIP-712 typed data for Add operation
    fn create_add_typed_data(
        &self,
        fid_owner: ethers::types::Address,
        key_type: u32,
        key: &[u8],
        metadata_type: u8,
        metadata: &[u8],
        nonce: u64,
        deadline: u64,
        key_gateway_address: ethers::types::Address,
        chain_id: u64,
    ) -> Result<ethers::types::transaction::eip712::TypedData> {
        use ethers::types::transaction::eip712::{EIP712Domain, Eip712DomainType, TypedData};
        use std::collections::BTreeMap;

        // Domain separator for Farcaster KeyGateway
        let domain = EIP712Domain {
            name: Some("Farcaster KeyGateway".to_string()),
            version: Some("1".to_string()),
            chain_id: Some(ethers::types::U256::from(chain_id)),
            verifying_contract: Some(key_gateway_address),
            salt: None,
        };

        // Type definition for Add struct - must match the contract's ADD_TYPEHASH
        let mut types = BTreeMap::new();
        types.insert(
            "EIP712Domain".to_string(),
            vec![
                Eip712DomainType {
                    name: "name".to_string(),
                    r#type: "string".to_string(),
                },
                Eip712DomainType {
                    name: "version".to_string(),
                    r#type: "string".to_string(),
                },
                Eip712DomainType {
                    name: "chainId".to_string(),
                    r#type: "uint256".to_string(),
                },
                Eip712DomainType {
                    name: "verifyingContract".to_string(),
                    r#type: "address".to_string(),
                },
            ],
        );
        types.insert(
            "Add".to_string(),
            vec![
                Eip712DomainType {
                    name: "owner".to_string(),
                    r#type: "address".to_string(),
                },
                Eip712DomainType {
                    name: "keyType".to_string(),
                    r#type: "uint32".to_string(),
                },
                Eip712DomainType {
                    name: "key".to_string(),
                    r#type: "bytes".to_string(),
                },
                Eip712DomainType {
                    name: "metadataType".to_string(),
                    r#type: "uint8".to_string(),
                },
                Eip712DomainType {
                    name: "metadata".to_string(),
                    r#type: "bytes".to_string(),
                },
                Eip712DomainType {
                    name: "nonce".to_string(),
                    r#type: "uint256".to_string(),
                },
                Eip712DomainType {
                    name: "deadline".to_string(),
                    r#type: "uint256".to_string(),
                },
            ],
        );

        // Data for Add struct - use proper types for EIP-712
        let mut message = BTreeMap::new();
        message.insert(
            "owner".to_string(),
            serde_json::Value::String(format!("0x{:x}", fid_owner)),
        );
        message.insert(
            "keyType".to_string(),
            serde_json::Value::String(key_type.to_string()),
        );
        message.insert(
            "key".to_string(),
            serde_json::Value::String(format!("0x{}", hex::encode(key))),
        );
        message.insert(
            "metadataType".to_string(),
            serde_json::Value::String(metadata_type.to_string()),
        );
        message.insert(
            "metadata".to_string(),
            serde_json::Value::String(format!("0x{}", hex::encode(metadata))),
        );
        message.insert(
            "nonce".to_string(),
            serde_json::Value::String(nonce.to_string()),
        );
        message.insert(
            "deadline".to_string(),
            serde_json::Value::String(deadline.to_string()),
        );

        Ok(TypedData {
            domain,
            types,
            primary_type: "Add".to_string(),
            message,
        })
    }
}

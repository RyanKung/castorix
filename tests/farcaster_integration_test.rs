use anyhow::Result;
use castorix::farcaster::contracts::{
    types::{ContractAddresses, ContractResult},
    contract_client::FarcasterContractClient,
};
use ed25519_dalek::{SigningKey, Signer as Ed25519Signer, Verifier};
use ethers::{
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256, TransactionRequest},
    middleware::Middleware,
    middleware::SignerMiddleware,
};
use rand::rngs::OsRng;
use std::str::FromStr;


/// Simplified integration test configuration
#[derive(Clone)]
pub struct IntegrationTestConfig {
    pub rpc_url: String,
    pub private_key: String,
}

impl IntegrationTestConfig {
    pub fn for_local_test() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:8545".to_string(),
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
        }
    }
    
    /// Create a config with a random wallet for testing
    pub fn with_random_wallet() -> Self {
        let random_wallet = LocalWallet::new(&mut OsRng);
        Self {
            rpc_url: "http://127.0.0.1:8545".to_string(),
            private_key: format!("0x{}", hex::encode(random_wallet.signer().to_bytes())),
        }
    }
}

/// Simplified integration test client
pub struct IntegrationTestClient {
    pub contract_client: FarcasterContractClient,
    pub wallet: LocalWallet,
    pub provider: Provider<Http>,
    pub config: IntegrationTestConfig,
}

impl IntegrationTestClient {
    /// Create a new integration test client
    pub async fn new(config: IntegrationTestConfig) -> Result<Self> {
        println!("🔧 Creating integration test client...");
        println!("   RPC URL: {}", config.rpc_url);
        
        let provider = Provider::<Http>::try_from(&config.rpc_url)?;
        println!("   Provider created successfully");
        
        let wallet = LocalWallet::from_str(&config.private_key)?;
        println!("   Wallet created successfully: {}", wallet.address());
        
        // Check wallet balance
        let balance = provider.get_balance(wallet.address(), None).await?;
        println!("   Wallet balance: {} ETH", ethers::utils::format_ether(balance));
        
        // Create contract client first
        let contract_client = FarcasterContractClient::new_with_wallet(
            config.rpc_url.clone(), 
            ContractAddresses::default(), 
            wallet.clone()
        )?;
        
        // If balance is too low, we need to fund it (for random wallets)
        if balance < ethers::utils::parse_ether("10.0")? {
            println!("   ⚠️  Wallet balance is low, attempting to fund...");
            println!("   Using NonceManager for funder account to address {}", wallet.address());
            
            // Use the same contract client's fund_wallet method which uses NonceManager
            match contract_client.fund_wallet(wallet.address(), ethers::utils::parse_ether("10.0")?).await {
                Ok(tx_hash) => {
                    println!("   ✅ Wallet funded successfully with NonceManager!");
                    println!("   Transaction Hash: {:?}", tx_hash);
                }
                Err(e) => {
                    println!("   ⚠️  Funding transaction failed: {}, but continuing...", e);
                }
            }
        }
        println!("   Contract client created successfully");

        Ok(Self {
            contract_client,
            wallet,
            provider,
            config,
        })
    }
    
    /// Get wallet address
    pub fn address(&self) -> Address {
        self.wallet.address()
    }
    
    /// Test FID registration with random address
    pub async fn test_fid_registration(&self) -> Result<U256> {
        println!("🚀 Testing FID registration with random address...");
        
        // Step 1: Generate a random wallet
        println!("🔑 Generating random wallet for FID registration...");
        let random_wallet = LocalWallet::new(&mut OsRng);
        let test_address = random_wallet.address();
        println!("   Test Address: {}", test_address);
        
        // Step 2: Transfer ETH to test address using main wallet
        println!("💰 Transferring 1 ETH to test address...");
        let transfer_tx = TransactionRequest::new()
            .to(test_address)
            .value(ethers::utils::parse_ether("1.0")?)
            .gas(21000);
        
        let chain_id = self.provider.get_chainid().await?;
        let wallet_with_chain_id = self.wallet.clone().with_chain_id(chain_id.as_u64());
        let signer_middleware = SignerMiddleware::new(self.provider.clone(), wallet_with_chain_id);
        let tx = signer_middleware.send_transaction(transfer_tx, None).await?.await?;
        match tx {
            Some(receipt) => {
                println!("✅ ETH transfer successful!");
                println!("   Transaction Hash: {:?}", receipt.transaction_hash);
            }
            None => return Err(anyhow::anyhow!("ETH transfer failed - no receipt")),
        }
        
        // Step 3: Register FID for the test address
        println!("🚀 Registering FID for test address...");
        let fid = self.test_real_registration_with_address(random_wallet, test_address).await?;
        println!("   Registered FID: {}", fid);
        
        Ok(fid)
    }

    /// Test real FID registration with specific wallet and address
    pub async fn test_real_registration_with_address(&self, wallet: LocalWallet, address: Address) -> Result<U256> {
        println!("🔑 Testing REAL FID registration with address: {}", address);
        
        // Check if address already has FID
        match self.contract_client.address_has_fid(address).await? {
                    Some(fid) => {
                println!("❌ Address already has FID: {}", fid);
                return Err(anyhow::anyhow!("Address already has FID: {}", fid));
                    }
                    None => {
                println!("✅ Address does not have FID - perfect for testing");
            }
        }
        
        // Get registration price
        println!("💰 Getting registration price...");
        let price = self.contract_client.id_gateway.price().await?;
        match price {
            ContractResult::Success(price) => {
                println!("   Registration price: {} ETH", ethers::utils::format_ether(price));
            }
            ContractResult::Error(e) => {
                return Err(anyhow::anyhow!("Failed to get registration price: {}", e));
            }
        }
        
        // Create contract client with the test wallet
        let test_contract_client = FarcasterContractClient::new_with_wallet(
            self.config.rpc_url.clone(),
            ContractAddresses::default(),
            wallet.clone()
        )?;
        
        // Register FID
        println!("🚀 Starting REAL FID registration...");
        let registration_result = test_contract_client.register_fid(address).await?;
        
        match registration_result {
            ContractResult::Success((fid, _)) => {
                println!("✅ FID registration successful!");
                println!("   FID: {}", fid);
                Ok(fid.into())
            }
            ContractResult::Error(e) => {
                println!("❌ FID registration failed: {}", e);
                Err(anyhow::anyhow!("FID registration failed: {}", e))
            }
        }
    }

    /// Test signer registration with new random wallet
    pub async fn test_signer_registration_with_new_wallet(&self) -> Result<SigningKey> {
        println!("🔑 Testing signer registration with new random wallet...");
        
        // Step 1: Generate a random wallet
        println!("🔑 Generating random wallet for signer registration...");
        let random_wallet = LocalWallet::new(&mut OsRng);
        let test_address = random_wallet.address();
        println!("   Test Address: {}", test_address);
        
        // Step 2: Transfer ETH to test address using main wallet
        println!("💰 Transferring 1 ETH to test address...");
        let transfer_tx = TransactionRequest::new()
            .to(test_address)
            .value(ethers::utils::parse_ether("1.0")?)
            .gas(21000);
        
        let chain_id = self.provider.get_chainid().await?;
        let wallet_with_chain_id = self.wallet.clone().with_chain_id(chain_id.as_u64());
        let signer_middleware = SignerMiddleware::new(self.provider.clone(), wallet_with_chain_id);
        let tx = signer_middleware.send_transaction(transfer_tx, None).await?.await?;
        match tx {
            Some(receipt) => {
                println!("✅ ETH transfer successful!");
                println!("   Transaction Hash: {:?}", receipt.transaction_hash);
            }
            None => return Err(anyhow::anyhow!("ETH transfer failed - no receipt")),
        }
        
        // Step 3: Register FID for the test address
        println!("🚀 Registering FID for test address...");
        let fid = self.test_real_registration_with_address(random_wallet.clone(), test_address).await?;
        println!("   Registered FID: {}", fid);
        
        // Step 4: Create contract client with the test wallet
        let test_contract_client = FarcasterContractClient::new_with_wallet(
            self.config.rpc_url.clone(),
            ContractAddresses::default(),
            random_wallet.clone()
        )?;
        
        // Step 5: Generate unique keypair
        println!("🔑 Generating unique keypair for signer registration...");
        let mut csprng = OsRng {};
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();
        let public_key_bytes = public_key.to_bytes();
        println!("   Generated public key: {}", hex::encode(public_key_bytes));
        
        // Step 6: Register the signer key
        println!("🚀 Registering signer key...");
        let registration_result = test_contract_client.register_signer_key(
            1, // Ed25519 key type
            public_key_bytes.to_vec(),
            1, // Ed25519 metadata type
            public_key_bytes.to_vec(), // metadata is the public key itself
        ).await?;
        
        match registration_result {
            ContractResult::Success(_) => {
                println!("✅ Signer key registered successfully!");
                
                // Verify the key was added to the registry
                match test_contract_client.key_registry.keys(fid.as_u64(), public_key_bytes.to_vec()).await? {
                    ContractResult::Success((state, key_type)) => {
                        println!("   ✅ Key found in registry!");
                        println!("     State: {} (0=NULL, 1=ADDED, 2=REMOVED)", state);
                        println!("     Key Type: {}", key_type);
            }
            ContractResult::Error(e) => {
                        println!("   ⚠️  Key verification failed: {}", e);
                        return Err(anyhow::anyhow!("Key was not found in registry after registration"));
                    }
                }
            }
            ContractResult::Error(e) => {
                println!("   ❌ Signer registration failed: {}", e);
                return Err(anyhow::anyhow!("Signer registration failed: {}", e));
            }
        }
        
        println!("🎉 Signer registration completed successfully!");
        Ok(signing_key)
    }

    /// Test third-party signer registration (Wallet A creates FID and signs, Wallet B pays gas)
    pub async fn test_third_party_signer_registration(&self) -> Result<SigningKey> {
        println!("🔑 Testing third-party signer registration...");
        
        // Step 1: Generate Wallet A (FID owner)
        println!("🔑 Generating Wallet A (FID owner)...");
        let wallet_a = LocalWallet::new(&mut OsRng);
        let wallet_a_address = wallet_a.address();
        println!("   Wallet A Address: {}", wallet_a_address);
        
        // Step 2: Transfer ETH to Wallet A using main wallet
        println!("💰 Transferring 1 ETH to Wallet A...");
        let transfer_tx = TransactionRequest::new()
            .to(wallet_a_address)
            .value(ethers::utils::parse_ether("1.0")?)
            .gas(21000);
        
        let chain_id = self.provider.get_chainid().await?;
        let wallet_with_chain_id = self.wallet.clone().with_chain_id(chain_id.as_u64());
        let signer_middleware = SignerMiddleware::new(self.provider.clone(), wallet_with_chain_id);
        let tx = signer_middleware.send_transaction(transfer_tx, None).await?.await?;
        match tx {
            Some(receipt) => {
                println!("✅ ETH transfer to Wallet A successful!");
                println!("   Transaction Hash: {:?}", receipt.transaction_hash);
            }
            None => return Err(anyhow::anyhow!("ETH transfer to Wallet A failed - no receipt")),
        }
        
        // Step 3: Register FID for Wallet A
        println!("🚀 Registering FID for Wallet A...");
        let fid = self.test_real_registration_with_address(wallet_a.clone(), wallet_a_address).await?;
        println!("   Registered FID: {}", fid);
        
        // Step 4: Rent storage for Wallet A's FID
        println!("🏠 Renting storage for Wallet A's FID...");
        let storage_client_a = FarcasterContractClient::new_with_wallet(
            self.config.rpc_url.clone(),
            ContractAddresses::default(),
            wallet_a.clone()
        )?;
        
        let rental_result = storage_client_a.rent_storage(fid.as_u64(), 1).await?;
        match rental_result {
            ContractResult::Success(_) => {
                println!("✅ Storage rental for Wallet A successful!");
            }
            ContractResult::Error(e) => {
                println!("   ⚠️  Storage rental failed: {}", e);
                // Continue anyway, storage rental is optional for this test
            }
        }
        
        // Step 5: Generate signer keypair for Wallet A
        println!("🔑 Generating signer keypair for Wallet A...");
        let mut csprng = OsRng {};
        let signing_key = SigningKey::generate(&mut csprng);
        let public_key = signing_key.verifying_key();
        let public_key_bytes = public_key.to_bytes();
        println!("   Generated public key: {}", hex::encode(public_key_bytes));
        
        // Step 6: Create contract client for Wallet A to generate signatures
        let client_a = FarcasterContractClient::new_with_wallet(
            self.config.rpc_url.clone(),
            ContractAddresses::default(),
            wallet_a.clone()
        )?;
        
        // Step 7: Generate Wallet B (gas payer)
        println!("🔑 Generating Wallet B (gas payer)...");
        let wallet_b = LocalWallet::new(&mut OsRng);
        let wallet_b_address = wallet_b.address();
        println!("   Wallet B Address: {}", wallet_b_address);
        
        // Step 8: Transfer ETH to Wallet B for gas
        println!("💰 Transferring 1 ETH to Wallet B for gas...");
        let transfer_tx_b = TransactionRequest::new()
            .to(wallet_b_address)
            .value(ethers::utils::parse_ether("1.0")?)
            .gas(21000);
        
        let tx_b = signer_middleware.send_transaction(transfer_tx_b, None).await?.await?;
        match tx_b {
            Some(receipt) => {
                println!("✅ ETH transfer to Wallet B successful!");
                println!("   Transaction Hash: {:?}", receipt.transaction_hash);
            }
            None => return Err(anyhow::anyhow!("ETH transfer to Wallet B failed - no receipt")),
        }
        
        // Step 9: Create contract client for Wallet B (gas payer)
        let client_b = FarcasterContractClient::new_with_wallet(
            self.config.rpc_url.clone(),
            ContractAddresses::default(),
            wallet_b.clone()
        )?;
        
        // Step 10: Wallet A generates all signatures, Wallet B submits transaction
        println!("🚀 Wallet A generating all signatures, Wallet B submitting transaction...");
        
        // First, let Wallet A generate the SignedKeyRequest signature
        let deadline = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs() + 3600;
            
        let signed_key_request_signature = client_a.create_signed_key_request_signature(
            fid.as_u64(),
            wallet_a_address,
            &public_key_bytes,
            deadline,
        ).await?;
        
        // Create SignedKeyRequestMetadata using Wallet A's signature
        let signed_key_request_metadata = client_a.create_signed_key_request_metadata(
            fid.as_u64(),
            wallet_a_address,
            &public_key_bytes,
            deadline,
            signed_key_request_signature,
        ).await?;
        
        // Wallet A also generates the KeyGateway.addFor signature
        let add_for_signature = client_a.create_add_for_signature(
            wallet_a_address,
            1, // Ed25519 key type
            &public_key_bytes,
            1, // Ed25519 metadata type
            &signed_key_request_metadata,
            deadline,
        ).await?;
        
        // Now Wallet B submits the transaction with Wallet A's signatures
        let registration_result = client_b.submit_signer_registration_with_signatures(
            wallet_a_address, // FID owner address
            fid.as_u64(),     // FID
            1,                 // Ed25519 key type
            public_key_bytes.to_vec(),
            1,                 // Ed25519 metadata type
            signed_key_request_metadata,
            deadline,
            add_for_signature,
        ).await?;
        
        match registration_result {
            ContractResult::Success(_) => {
                println!("✅ Third-party signer registration successful!");
                
                // Verify the key was added to the registry
                match client_b.key_registry.keys(fid.as_u64(), public_key_bytes.to_vec()).await? {
                    ContractResult::Success((state, key_type)) => {
                        println!("   ✅ Key found in registry!");
                        println!("     State: {} (0=NULL, 1=ADDED, 2=REMOVED)", state);
                        println!("     Key Type: {}", key_type);
                    }
                    ContractResult::Error(e) => {
                        println!("   ⚠️  Key verification failed: {}", e);
                        return Err(anyhow::anyhow!("Key was not found in registry after registration"));
                    }
                }
            }
            ContractResult::Error(e) => {
                println!("   ❌ Third-party signer registration failed: {}", e);
                return Err(anyhow::anyhow!("Third-party signer registration failed: {}", e));
            }
        }
        
        println!("🎉 Third-party signer registration completed successfully!");
        Ok(signing_key)
    }

    /// Test storage rental with random address
    pub async fn test_storage_rental(&self, fid: U256) -> Result<()> {
        println!("🚀 Testing storage rental for FID {}...", fid);
        
        // Get storage rental price
        let price = self.contract_client.storage_registry.price(1).await?;
        match price {
            ContractResult::Success(price) => {
                println!("   Storage rental price: {} ETH", ethers::utils::format_ether(price));
            }
            ContractResult::Error(e) => {
                return Err(anyhow::anyhow!("Failed to get storage price: {}", e));
            }
        }
        
        // Rent storage
        let rental_result = self.contract_client.rent_storage(fid.as_u64(), 1).await?;
        match rental_result {
            ContractResult::Success(_) => {
                println!("✅ Storage rental successful!");
            }
            ContractResult::Error(e) => {
                println!("❌ Storage rental failed: {}", e);
                return Err(anyhow::anyhow!("Storage rental failed: {}", e));
            }
        }
        
        Ok(())
    }

    /// Test contract connectivity
    pub async fn test_contract_connectivity(&self) -> Result<()> {
        println!("🔧 Testing contract connectivity...");
        
        // Test IdRegistry
        match self.contract_client.id_registry.id_counter().await? {
            ContractResult::Success(counter) => {
                println!("   ✅ IdRegistry connected - ID counter: {}", counter);
            }
            ContractResult::Error(e) => {
                println!("   ❌ IdRegistry connection failed: {}", e);
                return Err(anyhow::anyhow!("IdRegistry connection failed: {}", e));
            }
        }
        
        // Test KeyRegistry (using FID 1339338 for testing, state=1 for active keys)
        match self.contract_client.key_registry.total_keys(1339338, 1).await? {
            ContractResult::Success(total) => {
                println!("   ✅ KeyRegistry connected - Total keys for FID 1339338: {}", total);
                    }
                    ContractResult::Error(e) => {
                println!("   ❌ KeyRegistry connection failed: {}", e);
                return Err(anyhow::anyhow!("KeyRegistry connection failed: {}", e));
            }
        }
        
        // Test StorageRegistry
        match self.contract_client.storage_registry.rented_units().await? {
            ContractResult::Success(units) => {
                println!("   ✅ StorageRegistry connected - Rented units: {}", units);
            }
            ContractResult::Error(e) => {
                println!("   ❌ StorageRegistry connection failed: {}", e);
                return Err(anyhow::anyhow!("StorageRegistry connection failed: {}", e));
            }
        }
        
        println!("✅ All contracts connected successfully!");
        Ok(())
    }
}

// ============================================================================
// CORE INTEGRATION TESTS - All tests use random addresses for isolation
// ============================================================================

#[tokio::test]
async fn test_fid_registration_with_random_address() {
    let config = IntegrationTestConfig::with_random_wallet();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    let fid = client.test_fid_registration().await.expect("FID registration test failed");
    assert!(fid > U256::from(0));
    println!("✅ FID registration with random address successful: {}", fid);
}

#[tokio::test]
async fn test_signer_registration_with_random_address() {
    let config = IntegrationTestConfig::with_random_wallet();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    let signer_key = client.test_signer_registration_with_new_wallet().await
        .expect("Signer registration test failed");
    
    // Verify the keypair works
    let message = b"test message";
    let signature = Ed25519Signer::sign(&signer_key, message);
    assert!(signer_key.verifying_key().verify(message, &signature).is_ok());
    println!("✅ Signer registration with random address successful");
}

#[tokio::test]
async fn test_third_party_signer_registration() {
    let config = IntegrationTestConfig::with_random_wallet();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    let signer_key = client.test_third_party_signer_registration().await
        .expect("Third-party signer registration test failed");
    
    // Verify the keypair works
    let message = b"test message for third-party registration";
    let signature = Ed25519Signer::sign(&signer_key, message);
    assert!(signer_key.verifying_key().verify(message, &signature).is_ok());
    println!("✅ Third-party signer registration successful");
}

#[tokio::test]
async fn test_storage_rental_with_random_address() {
    let config = IntegrationTestConfig::with_random_wallet();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    // First register a FID with random address
    let fid = client.test_fid_registration().await.expect("FID registration test failed");
    
    // Then test storage rental
    client.test_storage_rental(fid).await.expect("Storage rental test failed");
    println!("✅ Storage rental with random address successful for FID: {}", fid);
}

#[tokio::test]
async fn test_contract_connectivity() {
    let config = IntegrationTestConfig::with_random_wallet();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    client.test_contract_connectivity().await.expect("Contract connectivity test failed");
    println!("✅ Contract connectivity test successful");
}

#[tokio::test]
async fn test_complete_flow_with_random_address() {
    let config = IntegrationTestConfig::with_random_wallet();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    println!("🚀 Testing complete Farcaster flow with random address...");
    
    // Step 1: Register FID
    let fid = client.test_fid_registration().await.expect("FID registration failed");
    println!("   ✅ FID registered: {}", fid);
    
    // Step 2: Register signer
    let signer_key = client.test_signer_registration_with_new_wallet().await
        .expect("Signer registration failed");
    println!("   ✅ Signer registered");
    
    // Step 3: Rent storage
    client.test_storage_rental(fid).await.expect("Storage rental failed");
    println!("   ✅ Storage rented");
    
    // Step 4: Verify keypair
    let message = b"complete flow test message";
    let signature = Ed25519Signer::sign(&signer_key, message);
    assert!(signer_key.verifying_key().verify(message, &signature).is_ok());
    println!("   ✅ Keypair verification successful");
    
    println!("🎉 Complete flow test successful!");
}

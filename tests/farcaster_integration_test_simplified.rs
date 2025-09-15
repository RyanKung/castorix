use anyhow::Result;
use castorix::farcaster::contracts::{
    types::{ContractAddresses, ContractResult},
    contract_client::FarcasterContractClient,
};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer as Ed25519Signer};
use ethers::{
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256, TransactionRequest},
    middleware::Middleware,
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
        
        let contract_client = FarcasterContractClient::new_with_wallet(
            config.rpc_url.clone(), 
            ContractAddresses::default(), 
            wallet.clone()
        )?;
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
        
        // Step 2: Transfer ETH to test address
        println!("💰 Transferring 1 ETH to test address...");
        let transfer_tx = TransactionRequest::new()
            .to(test_address)
            .value(ethers::utils::parse_ether("1.0")?)
            .gas(21000);
        
        let tx = self.wallet.send_transaction(transfer_tx, None).await?.await?;
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
        let price = self.contract_client.id_registry.price().await?;
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
            ContractResult::Success(fid) => {
                println!("✅ FID registration successful!");
                println!("   FID: {}", fid);
                Ok(fid)
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
        
        // Step 2: Transfer ETH to test address
        println!("💰 Transferring 1 ETH to test address...");
        let transfer_tx = TransactionRequest::new()
            .to(test_address)
            .value(ethers::utils::parse_ether("1.0")?)
            .gas(21000);
        
        let tx = self.wallet.send_transaction(transfer_tx, None).await?.await?;
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
                match test_contract_client.key_registry.keys(fid, public_key_bytes.to_vec()).await? {
                    ContractResult::Success((state, key_type)) => {
                        println!("   ✅ Key found in registry!");
                        println!("     State: {} (0=Active, 1=Inactive, 2=Pending)", state);
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
        let rental_result = self.contract_client.rent_storage(fid, 1).await?;
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
        
        // Test KeyRegistry - use a valid FID (we know FID 1339338 exists from previous tests, state=1 for active keys)
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
    let config = IntegrationTestConfig::for_local_test();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    let fid = client.test_fid_registration().await.expect("FID registration test failed");
    assert!(fid > U256::from(0));
    println!("✅ FID registration with random address successful: {}", fid);
}

#[tokio::test]
async fn test_signer_registration_with_random_address() {
    let config = IntegrationTestConfig::for_local_test();
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
async fn test_storage_rental_with_random_address() {
    let config = IntegrationTestConfig::for_local_test();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    // First register a FID with random address
    let fid = client.test_fid_registration().await.expect("FID registration test failed");
    
    // Then test storage rental
    client.test_storage_rental(fid).await.expect("Storage rental test failed");
    println!("✅ Storage rental with random address successful for FID: {}", fid);
}

#[tokio::test]
async fn test_contract_connectivity() {
    let config = IntegrationTestConfig::for_local_test();
    let client = IntegrationTestClient::new(config).await.expect("Failed to create test client");
    
    client.test_contract_connectivity().await.expect("Contract connectivity test failed");
    println!("✅ Contract connectivity test successful");
}

#[tokio::test]
async fn test_complete_flow_with_random_address() {
    let config = IntegrationTestConfig::for_local_test();
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

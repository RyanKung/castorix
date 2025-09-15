use anyhow::Result;
use ed25519_dalek::{Signer as Ed25519Signer, SigningKey, Verifier as Ed25519Verifier};
use ethers::{
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{transaction::eip2718::TypedTransaction, Address, TransactionRequest, U256},
    utils::parse_ether,
};
use rand::rngs::OsRng;

/// Simple local transaction test configuration
#[derive(Debug, Clone)]
pub struct SimpleTestConfig {
    pub rpc_url: String,
    pub private_key: String,
}

impl SimpleTestConfig {
    pub fn for_local_test() -> Self {
        Self {
            rpc_url: "http://127.0.0.1:8545".to_string(),
            private_key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
                .to_string(),
        }
    }
}

/// Simple wallet client for local testing
pub struct SimpleWalletClient {
    pub wallet: LocalWallet,
    pub provider: Provider<Http>,
    pub address: Address,
}

impl SimpleWalletClient {
    pub async fn new(config: SimpleTestConfig) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.rpc_url)?;
        let wallet = config.private_key.parse::<LocalWallet>()?;
        let address = wallet.address();

        Ok(Self {
            wallet,
            provider,
            address,
        })
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub async fn balance(&self) -> Result<U256> {
        Ok(self.provider.get_balance(self.address, None).await?)
    }

    pub async fn send_simple_transaction(&self) -> Result<()> {
        println!("🚀 Testing simple transaction on local network...");

        // Create a simple transaction to ourselves
        let tx_request = TransactionRequest::new()
            .to(self.address)
            .value(parse_ether("0.001")?)
            .gas(21000)
            .gas_price(1000000000u64);

        println!("📋 Transaction details:");
        println!("   From: {}", self.address);
        println!("   To: {}", self.address);
        println!("   Value: 0.001 ETH");
        println!("   Gas: 21000");
        println!("   Gas price: 1 Gwei");

        // Convert to TypedTransaction
        let typed_tx = TypedTransaction::Legacy(tx_request);

        // Send the transaction
        println!("\n📤 Sending transaction...");
        let pending_tx = self.provider.send_transaction(typed_tx, None).await?;
        println!("   Transaction hash: {}", pending_tx.tx_hash());

        // Wait for confirmation
        println!("   Waiting for confirmation...");
        let receipt = pending_tx.await?;

        match receipt {
            Some(receipt) => {
                println!("✅ Transaction confirmed!");
                println!("   Block number: {}", receipt.block_number.unwrap());
                println!("   Gas used: {}", receipt.gas_used.unwrap());
                println!(
                    "   Status: {}",
                    if receipt.status.unwrap() == 1u64.into() {
                        "Success"
                    } else {
                        "Failed"
                    }
                );
            }
            None => {
                println!("❌ Transaction failed - no receipt");
            }
        }

        Ok(())
    }

    pub async fn test_ed25519_signing(&self) -> Result<()> {
        println!("\n🔐 Testing Ed25519 signing...");

        // Generate Ed25519 key pair
        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let public_key = signing_key.verifying_key();

        println!("   Generated Ed25519 key pair");
        println!("   Public key: {}", hex::encode(public_key.as_bytes()));

        // Test message signing
        let test_message = b"Hello, Farcaster!";
        let signature = signing_key.sign(test_message);

        println!("   Message: {}", String::from_utf8_lossy(test_message));
        println!("   Signature: {}", hex::encode(signature.to_bytes()));

        // Verify signature
        match public_key.verify(test_message, &signature) {
            Ok(_) => {
                println!("✅ Ed25519 signature verification successful");
            }
            Err(e) => {
                println!("❌ Ed25519 signature verification failed: {}", e);
            }
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_simple_local_transaction() -> Result<()> {
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("🚀 Testing simple local transaction...");

    let config = SimpleTestConfig::for_local_test();
    let client = SimpleWalletClient::new(config).await?;

    println!("📋 Client Information:");
    println!("   Address: {}", client.address());
    let balance = client.balance().await?;
    println!("   Balance: {} ETH", ethers::utils::format_ether(balance));

    // Test simple transaction
    client.send_simple_transaction().await?;

    // Test Ed25519 signing
    client.test_ed25519_signing().await?;

    println!("\n🎉 Simple local transaction test completed!");

    Ok(())
}

#[tokio::test]
async fn test_network_connectivity() -> Result<()> {
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("🌐 Testing network connectivity...");

    let config = SimpleTestConfig::for_local_test();
    let client = SimpleWalletClient::new(config).await?;

    // Test network info
    let chain_id = client.provider.get_chainid().await?;
    let block_number = client.provider.get_block_number().await?;

    println!("📋 Network Information:");
    println!("   Chain ID: {}", chain_id);
    println!("   Block number: {}", block_number);
    println!("   RPC URL: {}", "http://127.0.0.1:8545");

    // Test balance
    let balance = client.balance().await?;
    println!("   Balance: {} ETH", ethers::utils::format_ether(balance));

    println!("✅ Network connectivity test completed!");

    Ok(())
}

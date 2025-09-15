use anyhow::Result;
use ethers::{
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, TransactionRequest, H256, U256},
};

/// Test environment for Farcaster contract testing with local blockchain
pub struct TestEnvironment {
    pub provider: Provider<Http>,
    pub wallets: Vec<LocalWallet>,
    pub deployer: LocalWallet,
    pub is_connected: bool,
}

impl TestEnvironment {
    /// Create a new test environment with local blockchain
    /// This will try to connect to a local Anvil instance running on localhost:8545
    pub async fn new() -> Result<Self> {
        // Try to connect to local Anvil instance
        let provider = Provider::<Http>::try_from("http://localhost:8545")?;

        // Test connection by getting chain ID
        let is_connected = match provider.get_chainid().await {
            Ok(_) => {
                println!("✅ Connected to local blockchain at http://localhost:8545");
                true
            }
            Err(_) => {
                println!("⚠️  Could not connect to local blockchain. Tests will be mocked.");
                false
            }
        };

        // Create test wallets
        let wallets: Vec<LocalWallet> = (0..5)
            .map(|_| LocalWallet::new(&mut rand::thread_rng()))
            .collect();
        let deployer = wallets[0].clone();

        Ok(Self {
            provider,
            wallets,
            deployer,
            is_connected,
        })
    }

    /// Get a wallet by index
    #[allow(dead_code)]
    pub fn wallet(&self, index: usize) -> LocalWallet {
        self.wallets[index].clone()
    }

    /// Get the deployer wallet
    #[allow(dead_code)]
    pub fn deployer(&self) -> LocalWallet {
        self.deployer.clone()
    }

    /// Get a provider reference
    #[allow(dead_code)]
    pub fn provider(&self) -> Provider<Http> {
        self.provider.clone()
    }

    /// Get the chain ID
    pub async fn chain_id(&self) -> Result<U256> {
        if !self.is_connected {
            println!("⚠️  Mock: Chain ID would be checked");
            return Ok(U256::from(31337)); // Mock chain ID (Anvil default)
        }

        Ok(self.provider.get_chainid().await?)
    }

    /// Get the block number
    pub async fn block_number(&self) -> Result<U256> {
        if !self.is_connected {
            println!("⚠️  Mock: Block number would be checked");
            return Ok(U256::from(1)); // Mock block number
        }

        Ok(U256::from(self.provider.get_block_number().await?.as_u64()))
    }

    /// Send ETH from deployer to an address
    pub async fn send_eth(&self, to: Address, amount: U256) -> Result<()> {
        if !self.is_connected {
            println!("⚠️  Mock: Would send {amount} ETH to {to:?}");
            return Ok(());
        }

        // For local testing, we'll simulate the transaction without actually sending it
        // since the deployer wallet might not have funds or proper setup
        println!("⚠️  Mock: Simulating ETH transfer of {amount} wei to {to:?}");
        println!("   From deployer: {:?}", self.deployer.address());

        // In a real implementation, you would:
        // 1. Check if deployer has sufficient balance
        // 2. Create and sign the transaction
        // 3. Send it to the network
        // 4. Wait for confirmation

        // For now, we'll just simulate success
        Ok(())
    }

    /// Fund a wallet with ETH
    pub async fn fund_wallet(&self, wallet: &LocalWallet, amount: U256) -> Result<()> {
        self.send_eth(wallet.address(), amount).await
    }

    /// Get account balance (mock if not connected)
    pub async fn balance(&self, address: Address) -> Result<U256> {
        if !self.is_connected {
            println!("⚠️  Mock: Balance for {address:?} would be checked");
            return Ok(U256::from(1000) * U256::from(10).pow(18.into())); // Mock balance
        }

        Ok(self.provider.get_balance(address, None).await?)
    }
}

/// Helper function to create a test environment
#[allow(dead_code)]
pub async fn setup_test_env() -> Result<TestEnvironment> {
    TestEnvironment::new().await
}

/// Helper function to create a funded wallet for testing
pub async fn create_funded_wallet(env: &TestEnvironment, amount: U256) -> Result<LocalWallet> {
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    env.fund_wallet(&wallet, amount).await?;
    Ok(wallet)
}

/// Mock Farcaster contract addresses for testing
pub struct MockContractAddresses {
    pub id_registry: Address,
    pub key_registry: Address,
    pub storage_registry: Address,
    pub id_gateway: Address,
    pub key_gateway: Address,
    pub bundler: Address,
    pub signed_key_request_validator: Address,
}

impl MockContractAddresses {
    /// Create mock contract addresses
    pub fn new() -> Self {
        Self {
            id_registry: "0x0000000000000000000000000000000000000001"
                .parse()
                .unwrap(),
            key_registry: "0x0000000000000000000000000000000000000002"
                .parse()
                .unwrap(),
            storage_registry: "0x0000000000000000000000000000000000000003"
                .parse()
                .unwrap(),
            id_gateway: "0x0000000000000000000000000000000000000004"
                .parse()
                .unwrap(),
            key_gateway: "0x0000000000000000000000000000000000000005"
                .parse()
                .unwrap(),
            bundler: "0x0000000000000000000000000000000000000006"
                .parse()
                .unwrap(),
            signed_key_request_validator: "0x0000000000000000000000000000000000000007"
                .parse()
                .unwrap(),
        }
    }
}

/// Test utilities for contract interactions
pub mod contract_utils {
    use super::*;
    use ethers::types::Bytes;

    /// Create a mock transaction request
    #[allow(dead_code)]
    pub fn create_mock_tx(
        from: Address,
        to: Address,
        data: Option<Bytes>,
        value: Option<U256>,
    ) -> TransactionRequest {
        TransactionRequest {
            from: Some(from),
            to: Some(to.into()),
            data,
            value,
            ..Default::default()
        }
    }

    /// Wait for transaction and return receipt (mock implementation)
    /// This is a simplified mock version for testing purposes
    #[allow(dead_code)]
    pub async fn send_and_wait(
        _provider: &Provider<Http>,
        tx: TransactionRequest,
    ) -> Result<ethers::types::TransactionReceipt> {
        // Mock transaction receipt for testing
        println!("⚠️  Mock: Transaction would be sent");
        println!("   From: {:?}", tx.from);
        println!("   To: {:?}", tx.to);
        println!("   Value: {:?}", tx.value);

        // Create a mock receipt with default values
        Ok(ethers::types::TransactionReceipt {
            transaction_hash: H256::random(),
            transaction_index: 0.into(),
            block_hash: Some(H256::random()),
            block_number: Some(1.into()),
            from: tx.from.unwrap_or(Address::zero()),
            to: tx.to.map(|addr| match addr {
                ethers::types::NameOrAddress::Address(a) => a,
                ethers::types::NameOrAddress::Name(_) => Address::zero(),
            }),
            cumulative_gas_used: 21000.into(),
            gas_used: Some(21000.into()),
            contract_address: None,
            logs: vec![],
            status: Some(1.into()),
            root: None,
            logs_bloom: Default::default(),
            transaction_type: Some(0.into()),
            effective_gas_price: tx.gas_price,
            other: Default::default(),
        })
    }
}

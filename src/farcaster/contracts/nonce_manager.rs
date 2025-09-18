//! Nonce management for concurrent transaction handling
//!
//! This module provides a thread-safe nonce manager that ensures
//! proper nonce sequencing for concurrent transactions.

use anyhow::Result;
use ethers::{
    middleware::Middleware,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Thread-safe nonce manager for a specific address
#[derive(Debug, Clone)]
pub struct NonceManager {
    address: Address,
    current_nonce: Arc<AtomicU64>,
    provider: Provider<Http>,
}

impl NonceManager {
    /// Create a new NonceManager for the given address
    pub async fn new(address: Address, provider: Provider<Http>) -> Result<Self> {
        // Get the initial nonce from the blockchain
        let initial_nonce = provider.get_transaction_count(address, None).await?;
        let nonce_u64 = initial_nonce.as_u64();

        println!(
            "   🔧 Initializing NonceManager for {} with nonce: {}",
            address, nonce_u64
        );

        Ok(Self {
            address,
            current_nonce: Arc::new(AtomicU64::new(nonce_u64)),
            provider,
        })
    }

    /// Get the next nonce atomically
    pub fn get_next_nonce(&self) -> U256 {
        let nonce = self.current_nonce.fetch_add(1, Ordering::SeqCst);
        println!("   📝 Getting next nonce for {}: {}", self.address, nonce);
        U256::from(nonce)
    }

    /// Get the current nonce without incrementing
    pub fn get_current_nonce(&self) -> U256 {
        let nonce = self.current_nonce.load(Ordering::SeqCst);
        U256::from(nonce)
    }

    /// Sync with blockchain nonce (useful for recovery)
    pub async fn sync_with_blockchain(&self) -> Result<()> {
        let blockchain_nonce = self
            .provider
            .get_transaction_count(self.address, None)
            .await?;
        let blockchain_nonce_u64 = blockchain_nonce.as_u64();

        // Update our nonce to match blockchain if it's higher
        let current = self.current_nonce.load(Ordering::SeqCst);
        if blockchain_nonce_u64 > current {
            self.current_nonce
                .store(blockchain_nonce_u64, Ordering::SeqCst);
            println!(
                "   🔄 Synced nonce for {}: {} -> {}",
                self.address, current, blockchain_nonce_u64
            );
        }

        Ok(())
    }

    /// Wait for a specific nonce to be confirmed on blockchain
    pub async fn wait_for_nonce_confirmation(&self, target_nonce: U256) -> Result<()> {
        let target = target_nonce.as_u64();
        println!(
            "   ⏳ Waiting for nonce {} to be confirmed for {}",
            target, self.address
        );

        for attempt in 1..=10 {
            sleep(Duration::from_millis(500 * attempt)).await;

            let blockchain_nonce = self
                .provider
                .get_transaction_count(self.address, None)
                .await?;
            let blockchain_nonce_u64 = blockchain_nonce.as_u64();

            if blockchain_nonce_u64 > target {
                println!("   ✅ Nonce {} confirmed for {}", target, self.address);
                return Ok(());
            }

            println!(
                "   ⏳ Attempt {}: blockchain nonce {} < target {}",
                attempt, blockchain_nonce_u64, target
            );
        }

        Err(anyhow::anyhow!(
            "Timeout waiting for nonce {} confirmation",
            target
        ))
    }

    /// Reset nonce to blockchain value (emergency recovery)
    pub async fn reset_to_blockchain(&self) -> Result<()> {
        let blockchain_nonce = self
            .provider
            .get_transaction_count(self.address, None)
            .await?;
        let blockchain_nonce_u64 = blockchain_nonce.as_u64();

        self.current_nonce
            .store(blockchain_nonce_u64, Ordering::SeqCst);
        println!(
            "   🔄 Reset nonce for {} to blockchain value: {}",
            self.address, blockchain_nonce_u64
        );

        Ok(())
    }

    /// Get address this manager is for
    pub fn address(&self) -> Address {
        self.address
    }
}

/// Global nonce manager registry for managing multiple addresses
#[derive(Debug)]
pub struct NonceRegistry {
    managers: std::collections::HashMap<Address, NonceManager>,
    provider: Provider<Http>,
}

impl NonceRegistry {
    /// Create a new nonce registry
    pub fn new(provider: Provider<Http>) -> Self {
        Self {
            managers: std::collections::HashMap::new(),
            provider,
        }
    }

    /// Get or create a nonce manager for the given address
    pub async fn get_manager(&mut self, address: Address) -> Result<&NonceManager> {
        if !self.managers.contains_key(&address) {
            let manager = NonceManager::new(address, self.provider.clone()).await?;
            self.managers.insert(address, manager);
        }

        Ok(self.managers.get(&address).unwrap())
    }

    /// Get next nonce for an address
    pub async fn get_next_nonce(&mut self, address: Address) -> Result<U256> {
        let manager = self.get_manager(address).await?;
        Ok(manager.get_next_nonce())
    }

    /// Sync all managers with blockchain
    pub async fn sync_all(&mut self) -> Result<()> {
        for (address, manager) in &self.managers {
            if let Err(e) = manager.sync_with_blockchain().await {
                println!("   ⚠️  Failed to sync nonce for {}: {}", address, e);
            }
        }
        Ok(())
    }
}

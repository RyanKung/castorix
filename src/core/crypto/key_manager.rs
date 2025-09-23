use anyhow::Context;
use anyhow::Result;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use ethers::signers::Signer;

/// Private key management system that loads keys from environment variables
#[derive(Clone)]
pub struct KeyManager {
    wallet: LocalWallet,
    address: Address,
}

impl KeyManager {
    /// Create a new KeyManager by loading private key from environment variable
    ///
    /// # Arguments
    /// * `env_key` - The environment variable name containing the private key
    ///
    /// # Returns
    /// * `Result<Self>` - The KeyManager instance or an error
    ///
    /// # Example
    /// ```rust
    /// use castorix::key_manager::KeyManager;
    /// // This will return an error if the environment variable is not set
    /// let result = KeyManager::from_env("PRIVATE_KEY");
    /// match result {
    ///     Ok(key_manager) => println!("KeyManager created successfully"),
    ///     Err(e) => println!("Failed to create KeyManager: {}", e),
    /// }
    /// ```
    pub fn from_env(_env_key: &str) -> Result<Self> {
        Err(anyhow::anyhow!("Environment variable access is not allowed. Use KeyManager::from_private_key() or encrypted key loading instead."))
    }

    /// Create a new KeyManager from a private key string
    ///
    /// # Arguments
    /// * `private_key` - The private key as a hex string (with or without 0x prefix)
    ///
    /// # Returns
    /// * `Result<Self>` - The KeyManager instance or an error
    pub fn from_private_key(private_key: &str) -> Result<Self> {
        // Remove 0x prefix if present
        let clean_key = if let Some(stripped) = private_key.strip_prefix("0x") {
            stripped
        } else {
            private_key
        };

        // Parse the private key
        let signing_key = SigningKey::from_slice(
            &hex::decode(clean_key).with_context(|| "Failed to decode private key from hex")?,
        )?;

        let wallet = LocalWallet::from(signing_key);
        let address = wallet.address();

        Ok(Self { wallet, address })
    }

    /// Get the wallet instance
    pub fn wallet(&self) -> &LocalWallet {
        &self.wallet
    }

    /// Get the wallet address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Sign a message with the private key
    ///
    /// # Arguments
    /// * `message` - The message to sign
    ///
    /// # Returns
    /// * `Result<Signature>` - The signature or an error
    pub async fn sign_message(&self, message: &str) -> Result<Signature> {
        self.wallet
            .sign_message(message)
            .await
            .with_context(|| "Failed to sign message")
    }

    /// Sign arbitrary data with the private key
    ///
    /// # Arguments
    /// * `data` - The data to sign
    ///
    /// # Returns
    /// * `Result<Signature>` - The signature or an error
    pub async fn sign_data(&self, data: &[u8]) -> Result<Signature> {
        self.wallet
            .sign_message(data)
            .await
            .with_context(|| "Failed to sign data")
    }

    /// Get the public key
    pub fn public_key(&self) -> Result<Vec<u8>> {
        let public_key = self.wallet.signer().verifying_key();
        Ok(public_key.to_sec1_bytes().to_vec())
    }

    /// Verify a signature
    ///
    /// # Arguments
    /// * `message` - The original message
    /// * `signature` - The signature to verify
    ///
    /// # Returns
    /// * `Result<bool>` - True if signature is valid, false otherwise
    pub async fn verify_signature(&self, message: &str, signature: &Signature) -> Result<bool> {
        // Recover the address from the signature
        let recovered_address = signature.recover(message)?;
        Ok(recovered_address == self.address)
    }
}

/// Initialize the environment variables from .env file
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn init_env() -> Result<()> {
    // Try to load .env file, but don't fail if it doesn't exist
    match dotenv::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenv::Error::Io(_)) => {
            // .env file doesn't exist, which is fine
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to load .env file: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_manager_creation() {
        // Test with a sample private key (DO NOT use in production)
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

        let key_manager = KeyManager::from_private_key(test_key);
        assert!(key_manager.is_ok());

        let key_manager = key_manager.unwrap();
        let address = key_manager.address();
        assert!(!address.is_zero());
    }

    #[tokio::test]
    async fn test_message_signing() {
        let test_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_manager = KeyManager::from_private_key(test_key).unwrap();

        let message = "Hello, World!";
        let signature = key_manager.sign_message(message).await;
        assert!(signature.is_ok());

        let signature = signature.unwrap();
        let is_valid = key_manager.verify_signature(message, &signature).await;
        assert!(is_valid.is_ok());
        assert!(is_valid.unwrap());
    }
}

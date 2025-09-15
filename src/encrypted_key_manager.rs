use crate::key_manager::KeyManager;
use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{Context, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use base64::{engine::general_purpose, Engine as _};
use ethers::{
    core::k256::ecdsa::SigningKey,
    prelude::*,
    signers::{LocalWallet, Signer},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Encrypted key storage structure
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedKeyData {
    /// Encrypted private key (base64 encoded)
    encrypted_key: String,
    /// Salt used for key derivation (base64 encoded)
    salt: String,
    /// Nonce used for encryption (base64 encoded)
    nonce: String,
    /// Wallet address for verification
    address: String,
    /// Key alias/display name
    alias: String,
    /// Creation timestamp
    created_at: u64,
}

/// Key information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    /// Key name (filename)
    pub name: String,
    /// Display alias
    pub alias: String,
    /// Wallet address
    pub address: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// Encrypted key manager that stores private keys encrypted on disk
pub struct EncryptedKeyManager {
    key_manager: Option<KeyManager>,
    storage_path: String,
}

impl EncryptedKeyManager {
    /// Create a new encrypted key manager
    ///
    /// # Arguments
    /// * `storage_path` - Path to store encrypted keys
    ///
    /// # Returns
    /// * `Self` - The EncryptedKeyManager instance
    pub fn new(storage_path: &str) -> Self {
        Self {
            key_manager: None,
            storage_path: storage_path.to_string(),
        }
    }

    /// Create encrypted key manager with default storage path
    ///
    /// # Returns
    /// * `Self` - The EncryptedKeyManager instance
    pub fn default_config() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| std::env::current_dir().unwrap());
        let storage_path = home_dir
            .join(".castorix")
            .join("keys")
            .to_string_lossy()
            .to_string();
        Self::new(&storage_path)
    }

    /// Import an existing private key and encrypt it with password
    ///
    /// # Arguments
    /// * `private_key_hex` - The private key as hex string (with or without 0x prefix)
    /// * `password` - Password to encrypt the private key
    /// * `key_name` - Name for the key (used as filename)
    /// * `alias` - Display alias for the key
    ///
    /// # Returns
    /// * `Result<String>` - The wallet address
    pub async fn import_and_encrypt(
        &mut self,
        private_key_hex: &str,
        password: &str,
        key_name: &str,
        alias: &str,
    ) -> Result<String> {
        // Parse the private key
        let clean_key = if let Some(stripped) = private_key_hex.strip_prefix("0x") {
            stripped
        } else {
            private_key_hex
        };

        let private_key_bytes =
            hex::decode(clean_key).with_context(|| "Failed to decode private key from hex")?;

        // Validate private key length
        if private_key_bytes.len() != 32 {
            return Err(anyhow::anyhow!(
                "Invalid private key length. Expected 32 bytes, got {}",
                private_key_bytes.len()
            ));
        }

        // Create wallet from private key
        let signing_key = SigningKey::from_slice(&private_key_bytes)
            .with_context(|| "Invalid private key format")?;
        let wallet = LocalWallet::from(signing_key);
        let address = wallet.address();

        // Encrypt the private key
        let mut encrypted_data = self.encrypt_private_key(&private_key_bytes, password, alias)?;
        encrypted_data.address = format!("{address:?}");

        // Save to file
        self.save_encrypted_key(key_name, &encrypted_data)?;

        // Create KeyManager for immediate use
        self.key_manager = Some(KeyManager::from_private_key(&hex::encode(
            private_key_bytes,
        ))?);

        Ok(format!("{address:?}"))
    }

    /// Generate a new private key (without encryption)
    ///
    /// # Returns
    /// * `Result<SigningKey>` - The generated private key or an error
    pub fn generate_private_key(&self) -> Result<SigningKey> {
        let signing_key = SigningKey::random(&mut OsRng);
        Ok(signing_key)
    }

    /// Generate a new private key and encrypt it with password
    ///
    /// # Arguments
    /// * `password` - Password to encrypt the private key
    /// * `key_name` - Name for the key (used as filename)
    /// * `alias` - Display alias for the key
    ///
    /// # Returns
    /// * `Result<String>` - The wallet address
    pub async fn generate_and_encrypt(
        &mut self,
        password: &str,
        key_name: &str,
        alias: &str,
    ) -> Result<String> {
        // Generate new private key
        let signing_key = SigningKey::random(&mut OsRng);
        let private_key_bytes = signing_key.to_bytes();
        let wallet = LocalWallet::from(signing_key);
        let address = wallet.address();

        // Encrypt the private key
        let mut encrypted_data = self.encrypt_private_key(&private_key_bytes, password, alias)?;
        encrypted_data.address = format!("{address:?}");

        // Save to file
        self.save_encrypted_key(key_name, &encrypted_data)?;

        // Create KeyManager for immediate use
        self.key_manager = Some(KeyManager::from_private_key(&hex::encode(
            private_key_bytes,
        ))?);

        Ok(format!("{address:?}"))
    }

    /// Load and decrypt a private key from storage
    ///
    /// # Arguments
    /// * `password` - Password to decrypt the private key
    /// * `key_name` - Name of the key to load
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn load_and_decrypt(&mut self, password: &str, key_name: &str) -> Result<()> {
        let encrypted_data = self.load_encrypted_key(key_name)?;

        // Decrypt the private key
        let private_key_bytes = self.decrypt_private_key(&encrypted_data, password)?;

        // Verify the address matches
        let signing_key = SigningKey::from_slice(&private_key_bytes)?;
        let wallet = LocalWallet::from(signing_key);
        let address = wallet.address();
        let expected_address: Address = encrypted_data
            .address
            .parse()
            .with_context(|| "Invalid address format in encrypted data")?;

        if address != expected_address {
            return Err(anyhow::anyhow!("Address mismatch - key may be corrupted"));
        }

        // Create KeyManager
        self.key_manager = Some(KeyManager::from_private_key(&hex::encode(
            private_key_bytes,
        ))?);

        Ok(())
    }

    /// Check if a key exists in storage
    ///
    /// # Arguments
    /// * `key_name` - Name of the key to check
    ///
    /// # Returns
    /// * `bool` - True if key exists
    pub fn key_exists(&self, key_name: &str) -> bool {
        let key_path = self.get_key_path(key_name);
        Path::new(&key_path).exists()
    }

    /// List all available keys
    ///
    /// # Returns
    /// * `Result<Vec<String>>` - List of key names
    pub fn list_keys(&self) -> Result<Vec<String>> {
        let storage_dir = Path::new(&self.storage_path);
        if !storage_dir.exists() {
            return Ok(vec![]);
        }

        let mut keys = Vec::new();
        for entry in fs::read_dir(storage_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    keys.push(stem.to_string_lossy().to_string());
                }
            }
        }

        Ok(keys)
    }

    /// Get the underlying KeyManager (if loaded)
    ///
    /// # Returns
    /// * `Option<&KeyManager>` - The KeyManager instance or None
    pub fn key_manager(&self) -> Option<&KeyManager> {
        self.key_manager.as_ref()
    }

    /// Get the wallet address (if loaded)
    ///
    /// # Returns
    /// * `Option<Address>` - The wallet address or None
    pub fn address(&self) -> Option<Address> {
        self.key_manager.as_ref().map(|km| km.address())
    }

    /// Sign a message with the loaded private key
    ///
    /// # Arguments
    /// * `message` - The message to sign
    ///
    /// # Returns
    /// * `Result<Signature>` - The signature or an error
    pub async fn sign_message(&self, message: &str) -> Result<Signature> {
        match &self.key_manager {
            Some(km) => km.sign_message(message).await,
            None => Err(anyhow::anyhow!("No key loaded. Please load a key first.")),
        }
    }

    /// Verify a signature
    ///
    /// # Arguments
    /// * `message` - The original message
    /// * `signature` - The signature to verify
    ///
    /// # Returns
    /// * `Result<bool>` - True if signature is valid
    pub async fn verify_signature(&self, message: &str, signature: &Signature) -> Result<bool> {
        match &self.key_manager {
            Some(km) => km.verify_signature(message, signature).await,
            None => Err(anyhow::anyhow!("No key loaded. Please load a key first.")),
        }
    }

    /// Encrypt private key with password
    fn encrypt_private_key(
        &self,
        private_key: &[u8],
        password: &str,
        alias: &str,
    ) -> Result<EncryptedKeyData> {
        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Derive key from password using Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash = password_hash.hash.unwrap();
        let key_bytes = hash.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);

        // Generate nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the private key
        let cipher = Aes256Gcm::new(key);
        let encrypted_key = cipher
            .encrypt(&nonce, private_key)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt private key: {}", e))?;

        Ok(EncryptedKeyData {
            encrypted_key: general_purpose::STANDARD.encode(&encrypted_key),
            salt: salt.to_string(),
            nonce: general_purpose::STANDARD.encode(nonce),
            address: String::new(), // Will be set by caller
            alias: alias.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Decrypt private key with password
    fn decrypt_private_key(
        &self,
        encrypted_data: &EncryptedKeyData,
        password: &str,
    ) -> Result<Vec<u8>> {
        // Parse salt
        let salt = SaltString::from_b64(&encrypted_data.salt)
            .map_err(|e| anyhow::anyhow!("Invalid salt format: {}", e))?;

        // Derive key from password using Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash = password_hash.hash.unwrap();
        let key_bytes = hash.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);

        // Decode nonce and encrypted data
        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted_data.nonce)
            .with_context(|| "Failed to decode nonce")?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted_key = general_purpose::STANDARD
            .decode(&encrypted_data.encrypted_key)
            .with_context(|| "Failed to decode encrypted key")?;

        // Decrypt the private key
        let cipher = Aes256Gcm::new(key);
        let decrypted_key = cipher.decrypt(nonce, encrypted_key.as_ref()).map_err(|e| {
            anyhow::anyhow!("Failed to decrypt private key - wrong password?: {}", e)
        })?;

        Ok(decrypted_key)
    }

    /// Save encrypted key to file
    fn save_encrypted_key(&self, key_name: &str, encrypted_data: &EncryptedKeyData) -> Result<()> {
        // Ensure storage directory exists
        let storage_dir = Path::new(&self.storage_path);
        fs::create_dir_all(storage_dir).with_context(|| "Failed to create storage directory")?;

        // Save to file
        let key_path = self.get_key_path(key_name);
        let json = serde_json::to_string_pretty(encrypted_data)
            .with_context(|| "Failed to serialize encrypted data")?;
        fs::write(&key_path, json).with_context(|| "Failed to write encrypted key file")?;

        Ok(())
    }

    /// Load encrypted key from file
    fn load_encrypted_key(&self, key_name: &str) -> Result<EncryptedKeyData> {
        let key_path = self.get_key_path(key_name);
        let json =
            fs::read_to_string(&key_path).with_context(|| "Failed to read encrypted key file")?;
        let encrypted_data: EncryptedKeyData =
            serde_json::from_str(&json).with_context(|| "Failed to parse encrypted key file")?;
        Ok(encrypted_data)
    }

    /// Rename an encrypted key
    ///
    /// # Arguments
    /// * `old_name` - Current key name
    /// * `new_name` - New key name
    /// * `password` - Password to verify ownership
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn rename_key(
        &mut self,
        old_name: &str,
        new_name: &str,
        password: &str,
    ) -> Result<()> {
        // Verify password by trying to load the key
        let mut temp_manager = EncryptedKeyManager::new(&self.storage_path);
        match temp_manager.load_and_decrypt(password, old_name).await {
            Ok(_) => {
                // Password is correct, proceed with renaming
                let old_path = self.get_key_path(old_name);
                let new_path = self.get_key_path(new_name);

                // Check if new name already exists
                if std::path::Path::new(&new_path).exists() {
                    return Err(anyhow::anyhow!("Key '{}' already exists", new_name));
                }

                // Rename the file
                std::fs::rename(&old_path, &new_path)
                    .with_context(|| "Failed to rename key file")?;

                Ok(())
            }
            Err(_) => Err(anyhow::anyhow!("Wrong password! Key not renamed.")),
        }
    }

    /// Update key alias
    ///
    /// # Arguments
    /// * `key_name` - Key name
    /// * `new_alias` - New alias
    /// * `password` - Password to verify ownership
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn update_alias(
        &mut self,
        key_name: &str,
        new_alias: &str,
        password: &str,
    ) -> Result<()> {
        // Load the encrypted key
        let mut encrypted_data = self.load_encrypted_key(key_name)?;

        // Verify password by trying to decrypt
        self.decrypt_private_key(&encrypted_data, password)?;

        // Update alias
        encrypted_data.alias = new_alias.to_string();

        // Save updated data
        self.save_encrypted_key(key_name, &encrypted_data)?;

        Ok(())
    }

    /// Get key information
    ///
    /// # Arguments
    /// * `key_name` - Key name
    ///
    /// # Returns
    /// * `Result<KeyInfo>` - Key information
    pub fn get_key_info(&self, key_name: &str) -> Result<KeyInfo> {
        let encrypted_data = self.load_encrypted_key(key_name)?;
        Ok(KeyInfo {
            name: key_name.to_string(),
            alias: encrypted_data.alias,
            address: encrypted_data.address,
            created_at: encrypted_data.created_at,
        })
    }

    /// List all keys with their information
    ///
    /// # Returns
    /// * `Result<Vec<KeyInfo>>` - List of key information
    pub fn list_keys_with_info(&self) -> Result<Vec<KeyInfo>> {
        let key_names = self.list_keys()?;
        let mut key_infos = Vec::new();

        for key_name in key_names {
            match self.get_key_info(&key_name) {
                Ok(info) => key_infos.push(info),
                Err(e) => {
                    eprintln!("Warning: Failed to get info for key '{key_name}': {e}");
                }
            }
        }

        Ok(key_infos)
    }

    /// Get the file path for a key
    fn get_key_path(&self, key_name: &str) -> String {
        format!("{}/{}.json", self.storage_path, key_name)
    }
}

/// Prompt user for password securely
pub fn prompt_password(prompt: &str) -> Result<String> {
    use rpassword::read_password;
    print!("{prompt}");
    std::io::Write::flush(&mut std::io::stdout()).with_context(|| "Failed to flush stdout")?;

    read_password().with_context(|| "Failed to read password")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_encrypted_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_string_lossy().to_string();
        let mut manager = EncryptedKeyManager::new(&storage_path);

        let password = "test_password_123";
        let key_name = "test_key";

        // Generate and encrypt key
        let address = manager
            .generate_and_encrypt(password, key_name, "test_alias")
            .await
            .unwrap();
        assert!(!address.is_empty());

        // Verify key exists
        assert!(manager.key_exists(key_name));

        // Verify address matches
        assert_eq!(format!("{:?}", manager.address().unwrap()), address);
    }

    #[tokio::test]
    async fn test_encrypted_key_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_string_lossy().to_string();
        let mut manager = EncryptedKeyManager::new(&storage_path);

        let password = "test_password_123";
        let key_name = "test_key";

        // Generate and encrypt key
        let address = manager
            .generate_and_encrypt(password, key_name, "test_alias")
            .await
            .unwrap();

        // Create new manager and load the key
        let mut manager2 = EncryptedKeyManager::new(&storage_path);
        manager2.load_and_decrypt(password, key_name).await.unwrap();

        // Verify addresses match
        assert_eq!(format!("{:?}", manager2.address().unwrap()), address);
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().to_string_lossy().to_string();
        let mut manager = EncryptedKeyManager::new(&storage_path);

        let password = "test_password_123";
        let wrong_password = "wrong_password";
        let key_name = "test_key";

        // Generate and encrypt key
        manager
            .generate_and_encrypt(password, key_name, "test_alias")
            .await
            .unwrap();

        // Try to load with wrong password
        let mut manager2 = EncryptedKeyManager::new(&storage_path);
        let result = manager2.load_and_decrypt(wrong_password, key_name).await;
        assert!(result.is_err());
    }
}

//! Encrypted key storage functionality
//!
//! This module provides encrypted storage for keys

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use aes_gcm::aead::Aead;
use aes_gcm::aead::AeadCore;
use aes_gcm::aead::KeyInit;
use aes_gcm::Aes256Gcm;
use aes_gcm::Key;
use aes_gcm::Nonce;
use anyhow::Context;
use anyhow::Result as AnyhowResult;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use base64::engine::general_purpose;
use base64::Engine as _;
use bs58;
use chrono;
use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;
use ethers::prelude::*;
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use hex;
use serde::Deserialize;
use serde::Serialize;

// Define CryptoError if it doesn't exist
#[derive(Debug)]
pub enum CryptoError {
    KeyNotFound(String),
    EncryptionError(String),
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    Other(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyNotFound(msg) => write!(f, "Key not found: {}", msg),
            CryptoError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::IoError(err) => write!(f, "IO error: {}", err),
            CryptoError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            CryptoError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

impl From<std::io::Error> for CryptoError {
    fn from(err: std::io::Error) -> Self {
        CryptoError::IoError(err)
    }
}

impl From<serde_json::Error> for CryptoError {
    fn from(err: serde_json::Error) -> Self {
        CryptoError::SerializationError(err)
    }
}

impl From<anyhow::Error> for CryptoError {
    fn from(err: anyhow::Error) -> Self {
        CryptoError::Other(err.to_string())
    }
}

/// Encrypted Ethereum key data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedEthKeyData {
    /// Encrypted Ethereum private key
    encrypted_private_key: String,
    /// Ethereum address (not encrypted, as it's not secret)
    address: String,
    /// The FID associated with this key
    fid: u64,
    /// Salt used for encryption
    salt: String,
    /// Nonce used for encryption
    nonce: String,
    /// Creation timestamp
    created_at: u64,
}

/// Ethereum key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthKeyInfo {
    /// FID associated with this key
    pub fid: u64,
    /// Ethereum address
    pub address: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// Encrypted Ed25519 key data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedEd25519KeyData {
    /// Encrypted Ed25519 signing key
    encrypted_signing_key: String,
    /// Public key (not encrypted, as it's not secret)
    public_key: String,
    /// The FID associated with this key
    fid: u64,
    /// Salt used for encryption
    salt: String,
    /// Nonce used for encryption
    nonce: String,
    /// Creation timestamp
    created_at: u64,
}

/// Ed25519 key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ed25519KeyInfo {
    /// FID associated with this key
    pub fid: u64,
    /// Public key
    pub public_key: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// Internal implementation of EncryptedEd25519KeyManager
struct EncryptedEd25519KeyManagerImpl {
    encrypted_keys: HashMap<u64, EncryptedEd25519KeyData>,
}

/// Internal implementation of EncryptedEthKeyManager
struct EncryptedEthKeyManagerImpl {
    encrypted_keys: HashMap<u64, EncryptedEthKeyData>,
}

/// Encrypted key manager trait
pub trait EncryptedKeyManager {
    /// Get the default keys file path
    fn default_keys_file() -> Result<String, CryptoError>;

    /// Load keys from file
    fn load_from_file(file_path: &str) -> Result<Self, CryptoError>
    where
        Self: Sized;

    /// Check if key exists for FID
    fn has_key(&self, fid: u64) -> bool;

    /// Get verifying key for FID
    fn get_verifying_key(
        &self,
        fid: u64,
        _password: &str,
    ) -> Result<ed25519_dalek::VerifyingKey, CryptoError>;
}

/// Encrypted Ed25519 key manager
pub struct EncryptedEd25519KeyManager {
    inner: EncryptedEd25519KeyManagerImpl,
}

/// Encrypted Ethereum key manager
pub struct EncryptedEthKeyManager {
    inner: EncryptedEthKeyManagerImpl,
}

impl EncryptedEd25519KeyManager {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            inner: EncryptedEd25519KeyManagerImpl::new(),
        }
    }

    /// Get default keys file path
    pub fn default_keys_file() -> Result<String, CryptoError> {
        EncryptedEd25519KeyManagerImpl::default_keys_file()
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Load from file
    pub fn load_from_file(file_path: &str) -> Result<Self, CryptoError> {
        let inner = EncryptedEd25519KeyManagerImpl::load_from_file(file_path)
            .map_err(|e| CryptoError::Other(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Check if key exists for FID
    pub fn has_key(&self, fid: u64) -> bool {
        self.inner.has_key(fid)
    }

    /// Get public key for FID
    pub fn get_public_key(&self, fid: u64) -> Result<String, CryptoError> {
        self.inner
            .get_public_key(fid)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// List all keys
    pub fn list_keys(&self) -> Vec<Ed25519KeyInfo> {
        self.inner.list_keys()
    }

    /// List keys with detailed info (alias for list_keys)
    pub fn list_keys_with_info(&self, _password: &str) -> Result<Vec<Ed25519KeyInfo>, CryptoError> {
        Ok(self.list_keys())
    }

    /// Generate and encrypt a new key
    pub async fn generate_and_encrypt(
        &mut self,
        fid: u64,
        password: &str,
    ) -> Result<(), CryptoError> {
        self.inner
            .generate_and_encrypt(fid, password)
            .await
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Import and encrypt an existing private key
    pub async fn import_and_encrypt(
        &mut self,
        fid: u64,
        private_key_hex: &str,
        password: &str,
    ) -> Result<(), CryptoError> {
        self.inner
            .import_and_encrypt(fid, private_key_hex, password)
            .await
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Remove key for FID
    pub fn remove_key(&mut self, fid: u64) -> Result<(), CryptoError> {
        self.inner
            .remove_key(fid)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Save to file
    pub fn save_to_file(&self, file_path: &str) -> Result<(), CryptoError> {
        self.inner
            .save_to_file(file_path)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Get signing key for FID
    pub fn get_signing_key(&self, fid: u64, password: &str) -> Result<SigningKey, CryptoError> {
        self.inner
            .get_signing_key(fid, password)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Get verifying key for FID
    pub fn get_verifying_key(&self, fid: u64, password: &str) -> Result<VerifyingKey, CryptoError> {
        self.inner
            .get_verifying_key(fid, password)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }
}

impl EncryptedKeyManager for EncryptedEd25519KeyManager {
    fn default_keys_file() -> Result<String, CryptoError> {
        EncryptedEd25519KeyManager::default_keys_file()
    }

    fn load_from_file(file_path: &str) -> Result<Self, CryptoError> {
        EncryptedEd25519KeyManager::load_from_file(file_path)
    }

    fn has_key(&self, fid: u64) -> bool {
        self.has_key(fid)
    }

    fn get_verifying_key(
        &self,
        fid: u64,
        password: &str,
    ) -> Result<ed25519_dalek::VerifyingKey, CryptoError> {
        self.get_verifying_key(fid, password)
    }
}

impl EncryptedEthKeyManager {
    /// Create a new instance
    pub fn new() -> Self {
        Self {
            inner: EncryptedEthKeyManagerImpl::new(),
        }
    }

    /// Get default keys file path
    pub fn default_keys_file() -> Result<String, CryptoError> {
        EncryptedEthKeyManagerImpl::default_keys_file()
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Load from file
    pub fn load_from_file(file_path: &str) -> Result<Self, CryptoError> {
        let inner = EncryptedEthKeyManagerImpl::load_from_file(file_path)
            .map_err(|e| CryptoError::Other(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Get custody key file
    pub fn custody_key_file(fid: u64) -> Result<String, CryptoError> {
        EncryptedEthKeyManagerImpl::custody_key_file(fid)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Check if key exists for FID
    pub fn has_key(&self, fid: u64) -> bool {
        self.inner.has_key(fid)
    }

    /// Get address for FID
    pub fn get_address(&self, fid: u64) -> Result<String, CryptoError> {
        self.inner
            .get_address(fid)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// List all keys
    pub fn list_keys(&self) -> Vec<(u64, String, u64)> {
        self.inner
            .list_keys()
            .into_iter()
            .map(|info| (info.fid, info.address, info.created_at))
            .collect()
    }

    /// Generate and encrypt a new key
    pub async fn generate_and_encrypt(
        &mut self,
        fid: u64,
        password: &str,
    ) -> Result<(), CryptoError> {
        self.inner
            .generate_and_encrypt(fid, password)
            .await
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Generate from recovery phrase
    pub async fn generate_from_recovery_phrase(
        &mut self,
        fid: u64,
        recovery_phrase: &str,
        password: &str,
    ) -> Result<(), CryptoError> {
        self.inner
            .generate_from_recovery_phrase(fid, recovery_phrase, password)
            .await
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Import and encrypt an existing private key
    pub async fn import_and_encrypt(
        &mut self,
        fid: u64,
        private_key_hex: &str,
        password: &str,
    ) -> Result<(), CryptoError> {
        self.inner
            .import_and_encrypt(fid, private_key_hex, password)
            .await
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Remove key for FID
    pub fn remove_key(&mut self, fid: u64) -> Result<(), CryptoError> {
        self.inner
            .remove_key(fid)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Save to file
    pub fn save_to_file(&self, file_path: &str) -> Result<(), CryptoError> {
        self.inner
            .save_to_file(file_path)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Decrypt and get wallet for FID
    pub fn decrypt_wallet(
        &self,
        fid: u64,
        password: &str,
    ) -> Result<ethers::signers::LocalWallet, CryptoError> {
        self.inner
            .decrypt_wallet(fid, password)
            .map_err(|e| CryptoError::Other(e.to_string()))
    }

    /// Get wallet for FID (alias for decrypt_wallet)
    pub fn get_wallet(
        &self,
        fid: u64,
        password: &str,
    ) -> Result<ethers::signers::LocalWallet, CryptoError> {
        self.decrypt_wallet(fid, password)
    }

    /// List keys with detailed info
    pub fn list_keys_with_info(&self, _password: &str) -> Result<Vec<EthKeyInfo>, CryptoError> {
        Ok(self.inner.list_keys())
    }
}

/// Prompt for password
pub fn prompt_password(prompt: &str) -> Result<String, anyhow::Error> {
    use rpassword::prompt_password;
    Ok(prompt_password(prompt)?)
}

/// Type alias for backward compatibility
pub type EncryptedKeyManagerType = EncryptedEd25519KeyManager;

impl EncryptedEd25519KeyManagerImpl {
    /// Create a new encrypted Ed25519 key manager
    fn new() -> Self {
        Self {
            encrypted_keys: HashMap::new(),
        }
    }

    /// Get the default keys file path
    fn default_keys_file() -> AnyhowResult<String> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let keys_dir = home_dir.join(".castorix").join("keys");
        std::fs::create_dir_all(&keys_dir)?;
        Ok(keys_dir
            .join("ed25519_keys.json")
            .to_string_lossy()
            .to_string())
    }

    /// Load keys from file
    fn load_from_file(file_path: &str) -> AnyhowResult<Self> {
        if !Path::new(file_path).exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read keys file: {file_path}"))?;

        let encrypted_keys: HashMap<u64, EncryptedEd25519KeyData> =
            serde_json::from_str(&content).with_context(|| "Failed to parse keys file")?;

        Ok(Self { encrypted_keys })
    }

    /// Save keys to file
    fn save_to_file(&self, file_path: &str) -> AnyhowResult<()> {
        let content = serde_json::to_string_pretty(&self.encrypted_keys)
            .with_context(|| "Failed to serialize keys")?;

        fs::write(file_path, content)
            .with_context(|| format!("Failed to write keys file: {file_path}"))?;

        Ok(())
    }

    /// Generate a new Ed25519 key pair and encrypt it
    async fn generate_and_encrypt(&mut self, fid: u64, password: &str) -> AnyhowResult<()> {
        // Check if key already exists for this FID
        if self.encrypted_keys.contains_key(&fid) {
            anyhow::bail!("Ed25519 key for FID {} already exists", fid);
        }

        // Generate new Ed25519 key pair
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();

        // Encrypt only the signing key (private key)
        let (encrypted_signing_key, salt, nonce) =
            self.encrypt_key(&signing_key.to_bytes(), password)?;

        // Store public key unencrypted (it's not secret)
        let public_key = hex::encode(verifying_key.to_bytes());

        let key_data = EncryptedEd25519KeyData {
            encrypted_signing_key,
            public_key,
            fid,
            salt,
            nonce,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        self.encrypted_keys.insert(fid, key_data);
        Ok(())
    }

    /// Import an existing Ed25519 private key and encrypt it
    async fn import_and_encrypt(
        &mut self,
        fid: u64,
        private_key_str: &str,
        password: &str,
    ) -> AnyhowResult<()> {
        // Check if key already exists for this FID
        if self.encrypted_keys.contains_key(&fid) {
            anyhow::bail!("Ed25519 key for FID {} already exists", fid);
        }

        // Try to decode as hex first, then base58
        let private_key_bytes = if let Some(stripped) = private_key_str.strip_prefix("0x") {
            // Remove 0x prefix and decode as hex
            hex::decode(stripped).context("Failed to decode private key hex")?
        } else if private_key_str.chars().all(|c| c.is_ascii_hexdigit()) {
            // Pure hex string
            hex::decode(private_key_str).context("Failed to decode private key hex")?
        } else {
            // Try base58 decoding (Solana format)
            bs58::decode(private_key_str)
                .into_vec()
                .map_err(|e| anyhow::anyhow!("Failed to decode private key as base58: {}", e))?
        };

        let signing_key = match private_key_bytes.len() {
            32 => {
                // Raw Ed25519 private key (32 bytes)
                SigningKey::from_bytes(
                    &private_key_bytes[..32]
                        .try_into()
                        .context("Invalid private key format")?,
                )
            }
            64 => {
                // Solana format: first 32 bytes are private key, last 32 bytes are public key
                SigningKey::from_bytes(
                    &private_key_bytes[..32]
                        .try_into()
                        .context("Invalid Solana private key format")?,
                )
            }
            _ => {
                anyhow::bail!("Invalid private key length: {} bytes. Expected 32 bytes (raw Ed25519) or 64 bytes (Solana format)", private_key_bytes.len());
            }
        };

        let verifying_key = signing_key.verifying_key();

        // Encrypt only the signing key (private key)
        let (encrypted_signing_key, salt, nonce) =
            self.encrypt_key(&signing_key.to_bytes(), password)?;

        // Store public key unencrypted (it's not secret)
        let public_key = hex::encode(verifying_key.to_bytes());

        let key_data = EncryptedEd25519KeyData {
            encrypted_signing_key,
            public_key,
            fid,
            salt,
            nonce,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        self.encrypted_keys.insert(fid, key_data);
        Ok(())
    }

    /// Get public key for a FID
    fn get_public_key(&self, fid: u64) -> AnyhowResult<String> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("No Ed25519 key found for FID: {}", fid))?;

        Ok(key_data.public_key.clone())
    }

    /// Get decrypted signing key for a FID
    fn get_signing_key(&self, fid: u64, password: &str) -> AnyhowResult<SigningKey> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("No Ed25519 key found for FID: {}", fid))?;

        let signing_key_bytes = self.decrypt_key(
            &key_data.encrypted_signing_key,
            &key_data.salt,
            &key_data.nonce,
            password,
        )?;

        Ok(SigningKey::from_bytes(
            &signing_key_bytes[..32]
                .try_into()
                .map_err(|e| anyhow::anyhow!("Invalid signing key format: {}", e))?,
        ))
    }

    /// Get verifying key for a FID
    fn get_verifying_key(&self, fid: u64, password: &str) -> AnyhowResult<VerifyingKey> {
        let signing_key = self.get_signing_key(fid, password)?;
        Ok(signing_key.verifying_key())
    }

    /// Check if key exists for FID
    fn has_key(&self, fid: u64) -> bool {
        self.encrypted_keys.contains_key(&fid)
    }

    /// List all keys
    fn list_keys(&self) -> Vec<Ed25519KeyInfo> {
        self.encrypted_keys
            .iter()
            .map(|(fid, key_data)| Ed25519KeyInfo {
                fid: *fid,
                public_key: key_data.public_key.clone(),
                created_at: key_data.created_at,
            })
            .collect()
    }

    /// Remove a key
    fn remove_key(&mut self, fid: u64) -> AnyhowResult<()> {
        self.encrypted_keys
            .remove(&fid)
            .ok_or_else(|| anyhow::anyhow!("No key found for FID: {}", fid))?;
        Ok(())
    }

    /// Encrypt a key with password
    fn encrypt_key(
        &self,
        key_bytes: &[u8],
        password: &str,
    ) -> AnyhowResult<(String, String, String)> {
        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = password_hash.hash.unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);

        // Generate nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt key: {}", e))?;

        Ok((
            general_purpose::STANDARD.encode(&ciphertext),
            general_purpose::STANDARD.encode(salt.as_str().as_bytes()),
            general_purpose::STANDARD.encode(nonce),
        ))
    }

    /// Decrypt a key with password
    fn decrypt_key(
        &self,
        encrypted_key: &str,
        salt: &str,
        nonce: &str,
        password: &str,
    ) -> AnyhowResult<Vec<u8>> {
        // Decode base64
        let ciphertext = general_purpose::STANDARD
            .decode(encrypted_key)
            .map_err(|e| anyhow::anyhow!("Failed to decode encrypted key: {}", e))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce)
            .map_err(|e| anyhow::anyhow!("Failed to decode nonce: {}", e))?;

        // Decode salt and recreate SaltString
        let salt_bytes = general_purpose::STANDARD
            .decode(salt)
            .map_err(|e| anyhow::anyhow!("Failed to decode salt: {}", e))?;

        let salt_str = String::from_utf8(salt_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to convert salt to string: {}", e))?;

        let salt = SaltString::from_b64(&salt_str)
            .map_err(|e| anyhow::anyhow!("Failed to recreate salt: {}", e))?;

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = password_hash.hash.unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);

        // Decrypt
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to decrypt key: {}", e))?;

        Ok(plaintext)
    }
}

impl EncryptedEthKeyManagerImpl {
    /// Create a new encrypted Ethereum key manager
    fn new() -> Self {
        Self {
            encrypted_keys: HashMap::new(),
        }
    }

    /// Get the default keys file path
    fn default_keys_file() -> AnyhowResult<String> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let keys_dir = home_dir.join(".castorix").join("custody");
        std::fs::create_dir_all(&keys_dir)?;
        Ok(keys_dir
            .join("custody_keys.json")
            .to_string_lossy()
            .to_string())
    }

    /// Get the custody key file path for a specific FID
    fn custody_key_file(fid: u64) -> AnyhowResult<String> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let keys_dir = home_dir.join(".castorix").join("custody");
        std::fs::create_dir_all(&keys_dir)?;
        Ok(keys_dir
            .join(format!("fid-{}-custody.json", fid))
            .to_string_lossy()
            .to_string())
    }

    /// Load keys from file
    fn load_from_file(file_path: &str) -> AnyhowResult<Self> {
        if !Path::new(file_path).exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read keys file: {file_path}"))?;

        let encrypted_keys: HashMap<u64, EncryptedEthKeyData> =
            serde_json::from_str(&content).with_context(|| "Failed to parse keys file")?;

        Ok(Self { encrypted_keys })
    }

    /// Save keys to file
    fn save_to_file(&self, file_path: &str) -> AnyhowResult<()> {
        let content = serde_json::to_string_pretty(&self.encrypted_keys)
            .with_context(|| "Failed to serialize keys")?;

        fs::write(file_path, content)
            .with_context(|| format!("Failed to write keys file: {file_path}"))?;

        Ok(())
    }

    /// Generate Ethereum key from recovery phrase and encrypt it
    async fn generate_from_recovery_phrase(
        &mut self,
        fid: u64,
        recovery_phrase: &str,
        password: &str,
    ) -> AnyhowResult<()> {
        // Check if key already exists for this FID
        if self.encrypted_keys.contains_key(&fid) {
            anyhow::bail!("Ethereum key for FID {} already exists", fid);
        }

        // Clean and normalize the recovery phrase
        let cleaned_phrase = recovery_phrase
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        // Use BIP44 standard path for Ethereum: m/44'/60'/0'/0/0
        let derivation_path = "m/44'/60'/0'/0/0";

        // Use ethers' built-in BIP44 derivation
        let wallet =
            ethers::signers::MnemonicBuilder::<ethers::signers::coins_bip39::English>::default()
                .phrase(&*cleaned_phrase)
                .derivation_path(derivation_path)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to create wallet from mnemonic with derivation path {}: {}",
                        derivation_path,
                        e
                    )
                })?
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build wallet: {}", e))?;

        let address = format!("{:?}", wallet.address());
        let private_key_bytes = wallet.signer().to_bytes();

        // Encrypt the private key
        let (encrypted_private_key, salt, nonce) =
            self.encrypt_key(&private_key_bytes, password)?;

        let key_data = EncryptedEthKeyData {
            encrypted_private_key,
            address,
            fid,
            salt,
            nonce,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        self.encrypted_keys.insert(fid, key_data);
        Ok(())
    }

    /// Generate a new Ethereum key pair and encrypt it
    async fn generate_and_encrypt(&mut self, fid: u64, password: &str) -> AnyhowResult<()> {
        // Check if key already exists for this FID
        if self.encrypted_keys.contains_key(&fid) {
            anyhow::bail!("Ethereum key for FID {} already exists", fid);
        }

        // Generate new Ethereum wallet
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let address = format!("{:?}", wallet.address());
        let private_key_bytes = wallet.signer().to_bytes();

        // Encrypt the private key
        let (encrypted_private_key, salt, nonce) =
            self.encrypt_key(&private_key_bytes, password)?;

        let key_data = EncryptedEthKeyData {
            encrypted_private_key,
            address,
            fid,
            salt,
            nonce,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        self.encrypted_keys.insert(fid, key_data);
        Ok(())
    }

    /// Import an existing Ethereum private key and encrypt it
    async fn import_and_encrypt(
        &mut self,
        fid: u64,
        private_key_hex: &str,
        password: &str,
    ) -> AnyhowResult<()> {
        // Check if key already exists for this FID
        if self.encrypted_keys.contains_key(&fid) {
            anyhow::bail!("Ethereum key for FID {} already exists", fid);
        }

        // Clean the private key hex
        let clean_key = if let Some(stripped) = private_key_hex.strip_prefix("0x") {
            stripped
        } else {
            private_key_hex
        };

        // Parse the private key
        let private_key_bytes = hex::decode(clean_key)
            .map_err(|e| anyhow::anyhow!("Invalid private key hex format: {}", e))?;

        // Validate private key length
        if private_key_bytes.len() != 32 {
            anyhow::bail!(
                "Invalid private key length. Expected 32 bytes, got {}",
                private_key_bytes.len()
            );
        }

        // Create wallet from private key
        let wallet = LocalWallet::from_bytes(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid private key format: {}", e))?;

        let address = format!("{:?}", wallet.address());

        // Encrypt the private key
        let (encrypted_private_key, salt, nonce) =
            self.encrypt_key(&private_key_bytes, password)?;

        let key_data = EncryptedEthKeyData {
            encrypted_private_key,
            address,
            fid,
            salt,
            nonce,
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        self.encrypted_keys.insert(fid, key_data);
        Ok(())
    }

    /// Get Ethereum address for a FID
    fn get_address(&self, fid: u64) -> AnyhowResult<String> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("No Ethereum key found for FID: {}", fid))?;

        Ok(key_data.address.clone())
    }

    /// Get decrypted private key for a FID
    fn get_private_key(&self, fid: u64, password: &str) -> AnyhowResult<Vec<u8>> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("No Ethereum key found for FID: {}", fid))?;

        self.decrypt_key(
            &key_data.encrypted_private_key,
            &key_data.salt,
            &key_data.nonce,
            password,
        )
    }

    /// Get wallet for a FID
    fn get_wallet(&self, fid: u64, password: &str) -> AnyhowResult<LocalWallet> {
        let private_key_bytes = self.get_private_key(fid, password)?;
        LocalWallet::from_bytes(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create wallet from private key: {}", e))
    }

    /// Check if key exists for FID
    fn has_key(&self, fid: u64) -> bool {
        self.encrypted_keys.contains_key(&fid)
    }

    /// List all keys
    fn list_keys(&self) -> Vec<EthKeyInfo> {
        self.encrypted_keys
            .iter()
            .map(|(fid, key_data)| EthKeyInfo {
                fid: *fid,
                address: key_data.address.clone(),
                created_at: key_data.created_at,
            })
            .collect()
    }

    /// Remove a key
    fn remove_key(&mut self, fid: u64) -> AnyhowResult<()> {
        self.encrypted_keys
            .remove(&fid)
            .ok_or_else(|| anyhow::anyhow!("No key found for FID: {}", fid))?;
        Ok(())
    }

    /// Decrypt and get wallet for FID
    fn decrypt_wallet(&self, fid: u64, password: &str) -> AnyhowResult<LocalWallet> {
        self.get_wallet(fid, password)
    }

    /// Encrypt a key with password
    fn encrypt_key(
        &self,
        key_bytes: &[u8],
        password: &str,
    ) -> AnyhowResult<(String, String, String)> {
        // Generate salt
        let salt = SaltString::generate(&mut OsRng);

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = password_hash.hash.unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);

        // Generate nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt
        let ciphertext = cipher
            .encrypt(&nonce, key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt key: {}", e))?;

        Ok((
            general_purpose::STANDARD.encode(&ciphertext),
            general_purpose::STANDARD.encode(salt.as_str().as_bytes()),
            general_purpose::STANDARD.encode(nonce),
        ))
    }

    /// Decrypt a key with password
    fn decrypt_key(
        &self,
        encrypted_key: &str,
        salt: &str,
        nonce: &str,
        password: &str,
    ) -> AnyhowResult<Vec<u8>> {
        // Decode base64
        let ciphertext = general_purpose::STANDARD
            .decode(encrypted_key)
            .map_err(|e| anyhow::anyhow!("Failed to decode encrypted key: {}", e))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce)
            .map_err(|e| anyhow::anyhow!("Failed to decode nonce: {}", e))?;

        // Decode salt and recreate SaltString
        let salt_bytes = general_purpose::STANDARD
            .decode(salt)
            .map_err(|e| anyhow::anyhow!("Failed to decode salt: {}", e))?;

        let salt_str = String::from_utf8(salt_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to convert salt to string: {}", e))?;

        let salt = SaltString::from_b64(&salt_str)
            .map_err(|e| anyhow::anyhow!("Failed to recreate salt: {}", e))?;

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = password_hash.hash.unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);

        // Decrypt
        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to decrypt key: {}", e))?;

        Ok(plaintext)
    }
}

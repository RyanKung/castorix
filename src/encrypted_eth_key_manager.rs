use aes_gcm::aead::{Aead, AeadCore, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{Context, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use base64::{engine::general_purpose, Engine as _};
use ethers::{
    prelude::*,
    signers::{LocalWallet, Signer},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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

/// Encrypted Ethereum key manager for Farcaster
pub struct EncryptedEthKeyManager {
    encrypted_keys: HashMap<u64, EncryptedEthKeyData>,
}

impl Default for EncryptedEthKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EncryptedEthKeyManager {
    /// Create a new encrypted Ethereum key manager
    pub fn new() -> Self {
        Self {
            encrypted_keys: HashMap::new(),
        }
    }

    /// Get the default keys file path
    pub fn default_keys_file() -> Result<String> {
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
    pub fn custody_key_file(fid: u64) -> Result<String> {
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
    pub fn load_from_file(file_path: &str) -> Result<Self> {
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
    pub fn save_to_file(&self, file_path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.encrypted_keys)
            .with_context(|| "Failed to serialize keys")?;

        fs::write(file_path, content)
            .with_context(|| format!("Failed to write keys file: {file_path}"))?;

        Ok(())
    }

    /// Generate Ethereum key from recovery phrase and encrypt it
    ///
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// * `recovery_phrase` - The recovery phrase (mnemonic)
    /// * `password` - Password for encryption
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn generate_from_recovery_phrase(
        &mut self,
        fid: u64,
        recovery_phrase: &str,
        password: &str,
    ) -> Result<()> {
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
        // This matches the approach used by MetaMask, Rabby, and other standard wallets
        let derivation_path = "m/44'/60'/0'/0/0";
        
        // Use ethers' built-in BIP44 derivation
        let wallet = ethers::signers::MnemonicBuilder::<ethers::signers::coins_bip39::English>::default()
            .phrase(&*cleaned_phrase)
            .derivation_path(&derivation_path)
            .map_err(|e| anyhow::anyhow!("Failed to create wallet from mnemonic with derivation path {}: {}", derivation_path, e))?
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
    pub async fn generate_and_encrypt(&mut self, fid: u64, password: &str) -> Result<()> {
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
    ///
    /// # Arguments
    /// * `fid` - The Farcaster ID
    /// * `private_key_hex` - The private key in hex format
    /// * `password` - Password for encryption
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn import_and_encrypt(
        &mut self,
        fid: u64,
        private_key_hex: &str,
        password: &str,
    ) -> Result<()> {
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
    pub fn get_address(&self, fid: u64) -> Result<String> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("No Ethereum key found for FID: {}", fid))?;

        Ok(key_data.address.clone())
    }

    /// Get decrypted private key for a FID
    pub fn get_private_key(&self, fid: u64, password: &str) -> Result<Vec<u8>> {
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
    pub fn get_wallet(&self, fid: u64, password: &str) -> Result<LocalWallet> {
        let private_key_bytes = self.get_private_key(fid, password)?;
        LocalWallet::from_bytes(&private_key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to create wallet from private key: {}", e))
    }

    /// Check if key exists for FID
    pub fn has_key(&self, fid: u64) -> bool {
        self.encrypted_keys.contains_key(&fid)
    }

    /// List all keys
    pub fn list_keys(&self) -> Vec<(u64, String, u64)> {
        self.encrypted_keys
            .iter()
            .map(|(fid, key_data)| (*fid, key_data.address.clone(), key_data.created_at))
            .collect()
    }

    /// List keys with detailed info
    pub fn list_keys_with_info(&self, _password: &str) -> Result<Vec<EthKeyInfo>> {
        let mut key_infos = Vec::new();

        for (fid, key_data) in &self.encrypted_keys {
            key_infos.push(EthKeyInfo {
                fid: *fid,
                address: key_data.address.clone(),
                created_at: key_data.created_at,
            });
        }

        Ok(key_infos)
    }

    /// Remove a key
    pub fn remove_key(&mut self, fid: u64) -> Result<()> {
        self.encrypted_keys
            .remove(&fid)
            .ok_or_else(|| anyhow::anyhow!("No key found for FID: {}", fid))?;
        Ok(())
    }

    /// Encrypt a key with password
    fn encrypt_key(&self, key_bytes: &[u8], password: &str) -> Result<(String, String, String)> {
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
    ) -> Result<Vec<u8>> {
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

/// Prompt for password
pub fn prompt_password(prompt: &str) -> Result<String> {
    use rpassword::read_password;
    print!("{prompt}");
    std::io::Write::flush(&mut std::io::stdout())?;
    read_password().map_err(|e| anyhow::anyhow!("Failed to read password: {}", e))
}

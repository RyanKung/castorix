use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::{Context, Result};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHasher};
use base64::{engine::general_purpose, Engine as _};
use bs58;
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Encrypted Ed25519 key manager for secure storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEd25519KeyManager {
    /// Map of FIDs to encrypted Ed25519 key data
    encrypted_keys: HashMap<u64, EncryptedEd25519KeyData>,
}

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

impl EncryptedEd25519KeyManager {
    /// Create a new encrypted Ed25519 key manager
    pub fn new() -> Self {
        Self {
            encrypted_keys: HashMap::new(),
        }
    }

    /// Generate a new Ed25519 key pair and encrypt it
    pub async fn generate_and_encrypt(&mut self, fid: u64, password: &str) -> Result<()> {
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
    /// Supports both 32-byte raw Ed25519 keys and 64-byte Solana format keys
    /// Supports both hex and base58 encoding
    pub async fn import_and_encrypt(
        &mut self,
        fid: u64,
        private_key_str: &str,
        password: &str,
    ) -> Result<()> {
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

    /// Decrypt and get a signing key by FID
    pub fn get_signing_key(&self, fid: u64, password: &str) -> Result<SigningKey> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("Ed25519 key for FID {} not found", fid))?;

        let decrypted_bytes = self.decrypt_key(
            &key_data.encrypted_signing_key,
            &key_data.salt,
            &key_data.nonce,
            password,
        )?;
        Ok(SigningKey::from_bytes(&decrypted_bytes[..32].try_into()?))
    }

    /// Get a verifying key by FID (no password needed as public key is stored unencrypted)
    pub fn get_verifying_key(&self, fid: u64, _password: &str) -> Result<VerifyingKey> {
        let key_data = self
            .encrypted_keys
            .get(&fid)
            .ok_or_else(|| anyhow::anyhow!("Ed25519 key for FID {} not found", fid))?;

        let public_key_bytes =
            hex::decode(&key_data.public_key).context("Failed to decode public key")?;
        Ok(VerifyingKey::from_bytes(
            &public_key_bytes[..32].try_into()?,
        )?)
    }

    /// Check if a key exists for a FID
    pub fn has_key(&self, fid: u64) -> bool {
        self.encrypted_keys.contains_key(&fid)
    }

    /// Update the FID for an existing key (move key to new FID)
    pub fn update_fid(&mut self, old_fid: u64, new_fid: u64) -> Result<()> {
        if let Some(key_data) = self.encrypted_keys.remove(&old_fid) {
            let mut updated_key_data = key_data;
            updated_key_data.fid = new_fid;
            self.encrypted_keys.insert(new_fid, updated_key_data);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Ed25519 key for FID {} not found", old_fid))
        }
    }

    /// List all available keys (without decrypting)
    pub fn list_keys(&self) -> Vec<(u64, String, u64)> {
        self.encrypted_keys
            .iter()
            .map(|(fid, key_data)| {
                let created_at = chrono::DateTime::from_timestamp(key_data.created_at as i64, 0)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string();
                (*fid, created_at, key_data.created_at)
            })
            .collect()
    }

    /// List keys with public key information (no password needed)
    pub fn list_keys_with_info(&self, _password: &str) -> Result<Vec<Ed25519KeyInfo>> {
        let mut key_infos = Vec::new();

        for (fid, key_data) in &self.encrypted_keys {
            key_infos.push(Ed25519KeyInfo {
                fid: *fid,
                public_key: key_data.public_key.clone(),
                created_at: key_data.created_at,
            });
        }

        Ok(key_infos)
    }

    /// Remove a key by FID
    pub fn remove_key(&mut self, fid: u64) -> Result<()> {
        self.encrypted_keys
            .remove(&fid)
            .ok_or_else(|| anyhow::anyhow!("Ed25519 key for FID {} not found", fid))?;
        Ok(())
    }

    /// Encrypt a key using AES-GCM with existing salt and nonce
    #[allow(dead_code)]
    fn encrypt_key_with_salt_nonce(
        &self,
        key_bytes: &[u8],
        password: &str,
        salt_str: &str,
        nonce_str: &str,
    ) -> Result<String> {
        // Decode nonce
        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_str)
            .context("Failed to decode nonce")?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Recreate salt
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| anyhow::anyhow!("Failed to recreate salt: {}", e))?;

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = password_hash.hash.unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);

        // Encrypt the key
        let ciphertext = cipher
            .encrypt(nonce, key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt key: {}", e))?;

        Ok(general_purpose::STANDARD.encode(&ciphertext))
    }

    /// Encrypt a key using AES-GCM
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
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the key
        let ciphertext = cipher
            .encrypt(nonce, key_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt key: {}", e))?;

        Ok((
            general_purpose::STANDARD.encode(&ciphertext),
            salt.as_str().to_string(),
            general_purpose::STANDARD.encode(nonce_bytes),
        ))
    }

    /// Decrypt a key using AES-GCM
    fn decrypt_key(
        &self,
        encrypted_key: &str,
        salt_str: &str,
        nonce_str: &str,
        password: &str,
    ) -> Result<Vec<u8>> {
        // Decode base64
        let ciphertext = general_purpose::STANDARD
            .decode(encrypted_key)
            .context("Failed to decode encrypted key")?;
        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_str)
            .context("Failed to decode nonce")?;

        // Recreate salt and nonce
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| anyhow::anyhow!("Failed to recreate salt: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Derive key from password
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let hash_bytes = password_hash.hash.unwrap();
        let key = Key::<Aes256Gcm>::from_slice(&hash_bytes.as_bytes()[..32]);
        let cipher = Aes256Gcm::new(key);

        // Decrypt the key
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Failed to decrypt key: {}", e))?;

        Ok(plaintext)
    }

    /// Save encrypted keys to file
    pub fn save_to_file(&self, file_path: &Path) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).context("Failed to create directory")?;
        }

        let json =
            serde_json::to_string_pretty(self).context("Failed to serialize encrypted keys")?;
        fs::write(file_path, json).context("Failed to write encrypted keys file")?;
        Ok(())
    }

    /// Load encrypted keys from file
    pub fn load_from_file(file_path: &Path) -> Result<Self> {
        if !file_path.exists() {
            return Ok(Self::new());
        }

        let json = fs::read_to_string(file_path).context("Failed to read encrypted keys file")?;
        let manager: Self =
            serde_json::from_str(&json).context("Failed to deserialize encrypted keys")?;
        Ok(manager)
    }

    /// Get the default encrypted keys file path
    pub fn default_keys_file() -> Result<std::path::PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir
            .join(".castorix")
            .join("encrypted_ed25519_keys.json"))
    }
}

#[derive(Debug, Clone)]
pub struct Ed25519KeyInfo {
    pub fid: u64,
    pub public_key: String,
    pub created_at: u64,
}

impl Default for EncryptedEd25519KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Prompt for password input
pub fn prompt_password(prompt: &str) -> Result<String> {
    use rpassword::read_password;
    use std::io::{self, Write};

    print!("{prompt}");
    io::stdout().flush()?;
    read_password().context("Failed to read password")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_and_encrypt() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 123;
        manager
            .generate_and_encrypt(fid, "test_password")
            .await
            .unwrap();

        let signing_key = manager.get_signing_key(fid, "test_password").unwrap();
        let verifying_key = manager.get_verifying_key(fid, "test_password").unwrap();

        assert!(manager.has_key(fid));
        assert_eq!(signing_key.verifying_key(), verifying_key);
    }

    #[tokio::test]
    async fn test_import_and_encrypt() {
        let mut manager = EncryptedEd25519KeyManager::new();

        // Generate a key first
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let private_key_hex = hex::encode(signing_key.to_bytes());
        let fid = 456;

        manager
            .import_and_encrypt(fid, &private_key_hex, "test_password")
            .await
            .unwrap();

        let imported_signing_key = manager.get_signing_key(fid, "test_password").unwrap();

        assert!(manager.has_key(fid));
        assert_eq!(signing_key.to_bytes(), imported_signing_key.to_bytes());
    }

    #[tokio::test]
    async fn test_encryption_decryption_roundtrip() {
        use ed25519_dalek::{Signer, Verifier};

        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 789;
        let password = "secure_password_123";

        // Generate a new key
        manager.generate_and_encrypt(fid, password).await.unwrap();

        // Verify we can decrypt and get the same key
        let signing_key = manager.get_signing_key(fid, password).unwrap();
        let verifying_key = manager.get_verifying_key(fid, password).unwrap();

        // Verify the key pair is valid
        assert_eq!(signing_key.verifying_key(), verifying_key);

        // Test signing and verification
        let message = b"Hello, Farcaster!";
        let signature = signing_key.sign(message);
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[tokio::test]
    async fn test_wrong_password_fails() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 999;
        let correct_password = "correct_password";
        let wrong_password = "wrong_password";

        // Generate and encrypt with correct password
        manager
            .generate_and_encrypt(fid, correct_password)
            .await
            .unwrap();

        // Try to decrypt with wrong password - should fail
        let result = manager.get_signing_key(fid, wrong_password);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_fid_fails() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 111;
        let password = "test_password";

        // First key should succeed
        manager.generate_and_encrypt(fid, password).await.unwrap();
        assert!(manager.has_key(fid));

        // Second key with same FID should fail
        let result = manager.generate_and_encrypt(fid, password).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[tokio::test]
    async fn test_import_hex_formats() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 222;
        let password = "test_password";

        // Generate a test key
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let private_key_bytes = signing_key.to_bytes();
        let private_key_hex = hex::encode(&private_key_bytes);

        // Test different hex formats
        let formats = vec![
            private_key_hex.clone(),          // Pure hex
            format!("0x{}", private_key_hex), // With 0x prefix
        ];

        for (i, format) in formats.iter().enumerate() {
            let test_fid = fid + i as u64;
            manager
                .import_and_encrypt(test_fid, format, password)
                .await
                .unwrap();

            let imported_key = manager.get_signing_key(test_fid, password).unwrap();
            assert_eq!(imported_key.to_bytes(), private_key_bytes);
        }
    }

    #[tokio::test]
    async fn test_file_save_load() {
        use ed25519_dalek::{Signer, Verifier};
        use tempfile::tempdir;

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_keys.json");

        let mut manager = EncryptedEd25519KeyManager::new();
        let fid1 = 333;
        let fid2 = 444;
        let password = "file_test_password";

        // Add some keys
        manager.generate_and_encrypt(fid1, password).await.unwrap();
        manager.generate_and_encrypt(fid2, password).await.unwrap();

        // Save to file
        manager.save_to_file(&file_path).unwrap();

        // Load from file
        let loaded_manager = EncryptedEd25519KeyManager::load_from_file(&file_path).unwrap();

        // Verify keys are still there and work
        assert!(loaded_manager.has_key(fid1));
        assert!(loaded_manager.has_key(fid2));

        let key1 = loaded_manager.get_signing_key(fid1, password).unwrap();
        let key2 = loaded_manager.get_signing_key(fid2, password).unwrap();

        // Verify they are different keys
        assert_ne!(key1.to_bytes(), key2.to_bytes());

        // Test signing with loaded keys
        let message = b"Test message";
        let sig1 = key1.sign(message);
        let sig2 = key2.sign(message);

        assert_ne!(sig1.to_bytes(), sig2.to_bytes());
        assert!(key1.verifying_key().verify(message, &sig1).is_ok());
        assert!(key2.verifying_key().verify(message, &sig2).is_ok());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let password = "list_test_password";

        // Add multiple keys
        let fids = vec![555, 666, 777];
        for fid in &fids {
            manager.generate_and_encrypt(*fid, password).await.unwrap();
        }

        // Test list_keys
        let keys = manager.list_keys();
        assert_eq!(keys.len(), 3);

        // Test list_keys_with_info
        let key_infos = manager.list_keys_with_info(password).unwrap();
        assert_eq!(key_infos.len(), 3);

        for key_info in key_infos {
            assert!(fids.contains(&key_info.fid));
            assert!(!key_info.public_key.is_empty());
            assert!(key_info.created_at > 0);
        }
    }

    #[tokio::test]
    async fn test_remove_key() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 888;
        let password = "remove_test_password";

        // Add a key
        manager.generate_and_encrypt(fid, password).await.unwrap();
        assert!(manager.has_key(fid));

        // Remove the key
        manager.remove_key(fid).unwrap();
        assert!(!manager.has_key(fid));

        // Try to get the removed key - should fail
        let result = manager.get_signing_key(fid, password);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_private_key_formats() {
        let mut manager = EncryptedEd25519KeyManager::new();
        let fid = 999;
        let password = "test_password";

        // Test invalid hex string
        let result = manager
            .import_and_encrypt(fid, "invalid_hex", password)
            .await;
        assert!(result.is_err());

        // Test wrong length hex (not 32 or 64 bytes)
        let short_hex = "1234567890abcdef"; // 16 bytes
        let result = manager.import_and_encrypt(fid, &short_hex, password).await;
        assert!(result.is_err());
    }
}

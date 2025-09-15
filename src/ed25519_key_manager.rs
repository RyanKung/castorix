use anyhow::{Context, Result};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Ed25519 key manager for Farcaster message signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ed25519KeyManager {
    /// Map of key names to Ed25519 signing keys
    keys: HashMap<String, Ed25519KeyPair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ed25519KeyPair {
    /// The Ed25519 signing key (private key)
    signing_key: Vec<u8>,
    /// The Ed25519 verifying key (public key)
    verifying_key: Vec<u8>,
    /// The FID associated with this key
    fid: u64,
}

impl Ed25519KeyManager {
    /// Create a new Ed25519 key manager
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Generate a new Ed25519 key pair for a given FID
    pub fn generate_key(&mut self, key_name: String, fid: u64) -> Result<()> {
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let verifying_key = signing_key.verifying_key();

        let key_pair = Ed25519KeyPair {
            signing_key: signing_key.to_bytes().to_vec(),
            verifying_key: verifying_key.to_bytes().to_vec(),
            fid,
        };

        self.keys.insert(key_name, key_pair);
        Ok(())
    }

    /// Import an existing Ed25519 private key
    pub fn import_key(&mut self, key_name: String, private_key_hex: &str, fid: u64) -> Result<()> {
        let private_key_bytes =
            hex::decode(private_key_hex).context("Failed to decode private key hex")?;

        let signing_key = SigningKey::from_bytes(
            &private_key_bytes[..32]
                .try_into()
                .context("Invalid private key length")?,
        );
        let verifying_key = signing_key.verifying_key();

        let key_pair = Ed25519KeyPair {
            signing_key: signing_key.to_bytes().to_vec(),
            verifying_key: verifying_key.to_bytes().to_vec(),
            fid,
        };

        self.keys.insert(key_name, key_pair);
        Ok(())
    }

    /// Get a signing key by name
    pub fn get_signing_key(&self, key_name: &str) -> Result<SigningKey> {
        let key_pair = self
            .keys
            .get(key_name)
            .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", key_name))?;

        Ok(SigningKey::from_bytes(
            &key_pair.signing_key[..32].try_into()?,
        ))
    }

    /// Get a verifying key by name
    pub fn get_verifying_key(&self, key_name: &str) -> Result<VerifyingKey> {
        let key_pair = self
            .keys
            .get(key_name)
            .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", key_name))?;

        Ok(VerifyingKey::from_bytes(
            &key_pair.verifying_key[..32].try_into()?,
        )?)
    }

    /// Get the FID associated with a key
    pub fn get_fid(&self, key_name: &str) -> Result<u64> {
        let key_pair = self
            .keys
            .get(key_name)
            .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", key_name))?;

        Ok(key_pair.fid)
    }

    /// List all available keys
    pub fn list_keys(&self) -> Vec<(String, u64, String)> {
        self.keys
            .iter()
            .map(|(name, key_pair)| {
                let public_key_hex = hex::encode(&key_pair.verifying_key);
                (name.clone(), key_pair.fid, public_key_hex)
            })
            .collect()
    }

    /// Remove a key
    pub fn remove_key(&mut self, key_name: &str) -> Result<()> {
        self.keys
            .remove(key_name)
            .ok_or_else(|| anyhow::anyhow!("Key '{}' not found", key_name))?;
        Ok(())
    }

    /// Update the FID for an existing key
    pub fn update_fid(&mut self, key_name: &str, fid: u64) -> Result<()> {
        if let Some(key_pair) = self.keys.get_mut(key_name) {
            key_pair.fid = fid;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Key '{}' not found", key_name))
        }
    }

    /// Save keys to file
    pub fn save_to_file(&self, file_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("Failed to serialize keys")?;
        fs::write(file_path, json).context("Failed to write keys file")?;
        Ok(())
    }

    /// Load keys from file
    pub fn load_from_file(file_path: &Path) -> Result<Self> {
        if !file_path.exists() {
            return Ok(Self::new());
        }

        let json = fs::read_to_string(file_path).context("Failed to read keys file")?;
        let manager: Self = serde_json::from_str(&json).context("Failed to deserialize keys")?;
        Ok(manager)
    }

    /// Get the default keys file path
    pub fn default_keys_file() -> Result<std::path::PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home_dir.join(".castorix").join("ed25519_keys.json"))
    }
}

impl Default for Ed25519KeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let mut manager = Ed25519KeyManager::new();
        manager.generate_key("test_key".to_string(), 123).unwrap();

        let signing_key = manager.get_signing_key("test_key").unwrap();
        let verifying_key = manager.get_verifying_key("test_key").unwrap();
        let fid = manager.get_fid("test_key").unwrap();

        assert_eq!(fid, 123);
        assert_eq!(signing_key.verifying_key(), verifying_key);
    }

    #[test]
    fn test_import_key() {
        let mut manager = Ed25519KeyManager::new();

        // Generate a key first
        let signing_key = SigningKey::generate(&mut rand::thread_rng());
        let private_key_hex = hex::encode(signing_key.to_bytes());

        manager
            .import_key("imported_key".to_string(), &private_key_hex, 456)
            .unwrap();

        let imported_signing_key = manager.get_signing_key("imported_key").unwrap();
        let fid = manager.get_fid("imported_key").unwrap();

        assert_eq!(fid, 456);
        assert_eq!(signing_key.to_bytes(), imported_signing_key.to_bytes());
    }
}

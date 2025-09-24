//! Cryptographic utilities and key management
//!
//! Provides secure key storage, signing, and encryption

pub mod encrypted_storage;
pub mod key_manager;

pub use encrypted_storage::CryptoError;
pub use encrypted_storage::Ed25519KeyInfo;
pub use encrypted_storage::EncryptedEd25519KeyManager;
pub use encrypted_storage::EncryptedEthKeyManager;
pub use encrypted_storage::EthKeyInfo;
pub use key_manager::KeyManager;

//! Core functionality for Castorix library
//!
//! This module contains the essential components for interacting with Farcaster protocol:
//! - Client: Farcaster Hub API client
//! - Crypto: Key management and cryptographic utilities  
//! - Protocol: Message types and protocol implementation
//! - Types: Common data structures
//! - Utils: Utility functions
//! - Contracts: Smart contract interactions

pub mod client;
pub mod contracts;
pub mod crypto;
pub mod protocol;
pub mod types;
pub mod utils;

// Re-exports for convenience
pub use client::hub_client::FarcasterClient;
pub use crypto::key_manager::KeyManager;
pub use protocol::message::Message;
pub use protocol::message::MessageData;
pub use protocol::message::MessageType;
pub use protocol::spam_checker::SpamChecker;
pub use protocol::username_proof::UserNameProof;
pub use protocol::username_proof::UserNameType;

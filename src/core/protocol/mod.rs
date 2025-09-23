//! Farcaster protocol implementation
//!
//! Message types, username proofs, and protocol utilities

pub mod message;
pub mod username_proof;
pub mod spam_checker;

pub use message::{Message, MessageData, MessageType};
pub use username_proof::{UserNameProof, UserNameType};
pub use spam_checker::SpamChecker;

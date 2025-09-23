use crate::cli::types::{
    CustodyCommands, EnsCommands, FidCommands, HubCommands, KeyCommands, SignersCommands,
    StorageCommands,
};
use clap::{Parser, Subcommand};

/// Castorix - Farcaster ENS Domain Proof Tool
/// A comprehensive tool for managing private keys, creating ENS domain proofs, and interacting with Farcaster Hub
#[derive(Parser)]
#[command(name = "castorix")]
#[command(version = "0.1.0")]
#[command(about = "🔐 Castorix - Secure Farcaster ENS Integration Tool")]
#[command(long_about = r#"
🔐 Castorix - Secure Farcaster ENS Integration Tool

A comprehensive command-line tool for managing encrypted private keys, creating ENS domain proofs, 
and interacting with Farcaster Hub. Castorix provides secure key storage with password-based 
encryption, making it safe to store private keys locally while maintaining easy access for 
Farcaster operations.

Key Features:
  • 🔒 Encrypted Private Key Management - Store keys securely with password protection
  • 🌐 ENS Domain Proof Creation - Generate proofs for your ENS domains
  • 📡 Farcaster Hub Integration - Submit proofs and interact with Farcaster
  • 🏷️  Key Aliases - Organize keys with friendly names and descriptions
  • 🔄 Key Management - Rename, update, and manage multiple keys
  • 📁 Custom Storage Path - Specify custom directory for storing encrypted keys

Examples:
  # Generate a new encrypted key
  castorix key generate-encrypted my-wallet "My Main Wallet"
  
  # Import an existing private key
  castorix key import existing-key "Backup Wallet" 0x1234...
  
  # List all your keys
  castorix key list
  
  # Generate an ENS proof (using default key)
  castorix ens proof vitalik.eth 12345
  
  # Generate an ENS proof (using specific encrypted wallet)
  castorix ens proof ryankung.base.eth 460432 --wallet-name my-wallet
  
  # Use custom storage path
  castorix --path /custom/path key generate-encrypted my-wallet "My Wallet"
  
For more information, visit: https://github.com/your-repo/castorix
"#)]
pub struct Cli {
    /// Custom path for storing encrypted keys and configuration files
    /// If not specified, uses the default system directory (~/.castorix/)
    #[arg(long, global = true, value_name = "PATH")]
    pub path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 🔑 Key management operations
    ///
    /// Manage encrypted private keys with secure password protection.
    /// Create, import, list, and organize your wallet keys with friendly aliases.
    Key {
        #[command(subcommand)]
        action: KeyCommands,
    },
    /// 🌐 ENS domain proof operations
    ///
    /// Generate and verify ENS domain proofs for Farcaster integration.
    /// Link your ENS domains to your Farcaster identity.
    ///
    /// Use --wallet-name to select specific encrypted wallets for signing.
    Ens {
        #[command(subcommand)]
        action: EnsCommands,
    },
    /// 📡 Farcaster Hub operations
    ///
    /// Interact with Farcaster Hub to submit proofs and retrieve data.
    /// Submit casts, proofs, and get user information.
    Hub {
        #[command(subcommand)]
        action: HubCommands,
    },
    /// 🔑 ECDSA custody key management
    ///
    /// Manage ECDSA keys used for Farcaster account management.
    /// These keys are bound to specific FIDs and used for custody operations.
    Custody {
        #[command(subcommand)]
        action: CustodyCommands,
    },
    /// 🔐 Signer management for Farcaster
    ///
    /// Manage and query signers (account keys) associated with Farcaster IDs.
    /// These are Ed25519 public keys that are authorized to sign messages for FIDs.
    Signers {
        #[command(subcommand)]
        action: SignersCommands,
    },
    /// 🆔 FID registration and management
    ///
    /// Register new Farcaster IDs (FIDs) and manage existing ones.
    /// This includes checking registration prices and listing owned FIDs.
    Fid {
        #[command(subcommand)]
        action: FidCommands,
    },
    /// 🏠 Storage rental and management
    ///
    /// Rent and manage storage units for Farcaster IDs.
    /// This allows FIDs to store more messages, casts, and other data.
    Storage {
        #[command(subcommand)]
        action: StorageCommands,
    },
}

impl Cli {
    /// Parse command line arguments
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

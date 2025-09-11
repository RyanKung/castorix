use clap::{Parser, Subcommand};
use crate::cli::types::*;

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

Examples:
  # Generate a new encrypted key
  castorix key generate-encrypted my-wallet "My Main Wallet"
  
  # Import an existing private key
  castorix key import existing-key "Backup Wallet" 0x1234...
  
  # List all your keys
  castorix key list
  
  # Create an ENS proof (using default key)
  castorix ens create vitalik.eth 12345
  
  # Create an ENS proof (using specific encrypted wallet)
  castorix ens create ryankung.base.eth 460432 --wallet-name my-wallet
  
  # Run a comprehensive demo
  castorix demo

For more information, visit: https://github.com/your-repo/castorix
"#)]
pub struct Cli {
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
    /// Create and verify ENS domain proofs for Farcaster integration.
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
    /// 🚀 Demo all functionality
    /// 
    /// Run a comprehensive demonstration of all Castorix features.
    /// Perfect for first-time users to see what the tool can do.
    Demo,
}

impl Cli {
    /// Parse command line arguments
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }
}

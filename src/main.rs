// Castorix CLI - Command line interface for Farcaster protocol interaction
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA

use castorix::{
    cli::{Cli, CliHandler, commands::Commands, types::{KeyCommands, HubCommands}},
    key_manager::{KeyManager, init_env},
    ens_proof::EnsProof,
    farcaster_client::FarcasterClient
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment variables from .env file
    init_env()?;

    // Parse command line arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::Key { action } => {
            // For encrypted key commands, we don't need to load from env
            match action {
                KeyCommands::Info |
                KeyCommands::GenerateEncrypted |
                KeyCommands::Import |
                KeyCommands::Load { .. } |
                KeyCommands::List |
                KeyCommands::Delete { .. } => {
                    // These commands handle their own key management
                    CliHandler::handle_key_command(action, &KeyManager::from_private_key("0000000000000000000000000000000000000000000000000000000000000001")?).await?;
                }
                _ => {
                    // For other commands, try to load from environment
                    match KeyManager::from_env("PRIVATE_KEY") {
                        Ok(key_manager) => {
                            CliHandler::handle_key_command(action, &key_manager).await?;
                        }
                        Err(_) => {
                            println!("❌ No private key found in environment variables.");
                            println!("💡 Use 'castorix key generate-encrypted <name>' to create an encrypted key, or");
                            println!("   set PRIVATE_KEY environment variable for legacy mode.");
                        }
                    }
                }
            }
        }
        Commands::Hub { action } => {
            let hub_url = std::env::var("FARCASTER_HUB_URL").unwrap_or_else(|_| "https://hub-api.neynar.com".to_string());
            
            // For read-only operations, we don't need a key manager
            match action {
                HubCommands::User { .. } |
                HubCommands::EthAddresses { .. } |
                HubCommands::EnsDomains { .. } |
                HubCommands::CustodyAddress { .. } |
                HubCommands::Signers { .. } |
                HubCommands::Key { .. } => {
                    let hub_client = FarcasterClient::read_only(hub_url);
                    CliHandler::handle_hub_command(action, &hub_client).await?;
                }
                HubCommands::SubmitProof { .. } |
                HubCommands::SubmitProofEip712 { .. } => {
                    // SubmitProof and SubmitProofEip712 handle their own key management
                    let hub_client = FarcasterClient::read_only(hub_url);
                    CliHandler::handle_hub_command(action, &hub_client).await?;
                }
                _ => {
                    // For write operations, we need a key manager
                    match KeyManager::from_env("PRIVATE_KEY") {
                        Ok(key_manager) => {
                            let hub_client = FarcasterClient::with_key_manager(hub_url, key_manager);
                            CliHandler::handle_hub_command(action, &hub_client).await?;
                        }
                        Err(_) => {
                            println!("❌ No private key found in environment variables.");
                            println!("💡 Please either:");
                            println!("   1. Set PRIVATE_KEY environment variable for legacy mode, or");
                            println!("   2. Use 'castorix key load <key-name>' to load an encrypted key first");
                        }
                    }
                }
            }
        }
        Commands::Ens { action } => {
            let rpc_url = std::env::var("ETH_RPC_URL")
                .unwrap_or_else(|_| "https://eth-mainnet.g.alchemy.com/v2/demo".to_string());
            
            // Create a dummy key manager for ENS operations
            let dummy_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
            if let Ok(key_manager) = KeyManager::from_private_key(dummy_key) {
                let ens_proof = EnsProof::new(key_manager, rpc_url);
                CliHandler::handle_ens_command(action, &ens_proof).await?;
            } else {
                println!("❌ Failed to create key manager for ENS operations");
            }
        }
        Commands::Demo => {
            run_demo().await?;
        }
    }

    Ok(())
}

/// Run a comprehensive demo of all functionality
async fn run_demo() -> Result<()> {
    println!("🚀 Castorix Demo - Comprehensive Farcaster ENS Integration");
    println!("{}", "=".repeat(60));

    // Initialize environment variables
    castorix::key_manager::init_env()?;

    // Key Management Demo
    println!("\n🔑 Key Management Demo:");
    let key_manager = KeyManager::from_env("PRIVATE_KEY")?;
    println!("   Wallet Address: {:?}", key_manager.address());
    
    let message = "Hello, Castorix!";
    let signature = key_manager.sign_message(message).await?;
    println!("   Message: {message}");
    println!("   Signature: {signature:?}");
    
    let is_valid = key_manager.verify_signature(message, &signature).await?;
    println!("   Signature valid: {is_valid}");

    // ENS Proof Demo
    println!("\n🌐 ENS Domain Proof Demo:");
    let ens_proof = EnsProof::from_env()?;
    let domain = "example.eth";
    let fid = 12345;
    
    println!("   Attempting to create proof for domain: {domain}");
    match ens_proof.create_ens_proof(domain, fid).await {
        Ok(proof) => {
            println!("   ✅ Successfully created ENS proof!");
            if let Ok(json) = ens_proof.serialize_proof(&proof) {
                println!("   📄 Proof JSON:");
                println!("   {json}");
            }
        }
        Err(e) => {
            println!("   ❌ Failed to create ENS proof: {e}");
            println!("   💡 This is expected if you don't own the domain '{domain}'");
        }
    }

    // Farcaster Hub Demo
    println!("\n📡 Farcaster Hub Integration Demo:");
    let farcaster_client = FarcasterClient::from_env()?;
    println!("   Connected to Farcaster Hub: {}", farcaster_client.hub_url());
    
    let fid = 12345;
    println!("   Fetching user information for FID: {fid}");
    match farcaster_client.get_user(fid).await {
        Ok(user_data) => {
            println!("   ✅ User data retrieved:");
            println!("   {}", serde_json::to_string_pretty(&user_data)?);
        }
        Err(e) => {
            println!("   ❌ Failed to get user data: {e}");
            println!("   💡 This is expected if the FID doesn't exist or hub is unreachable");
        }
    }

    println!("\n🎉 Demo completed! Use 'castorix --help' to see all available commands.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use ethers::signers::{LocalWallet, Signer};

    #[test]
    fn test_ecdsa_to_ed25519_conversion() {
        println!("🔑 Testing: ECDSA and Ed25519 private key length verification");
        println!("{}", "=".repeat(60));
        
        // Generate an ECDSA private key
        let ecdsa_wallet = LocalWallet::new(&mut rand::thread_rng());
        let ecdsa_private_key = ecdsa_wallet.signer().to_bytes();
        
        println!("📊 ECDSA Private Key Information:");
        println!("   Private Key (hex): {}", hex::encode(ecdsa_private_key));
        println!("   Private Key Length: {} bytes", ecdsa_private_key.len());
        println!("   Private Key Bits: {} bits", ecdsa_private_key.len() * 8);
        println!("   Address: {:?}", ecdsa_wallet.address());
        
        // Assert that the private key is exactly 32 bytes (256 bits)
        assert_eq!(ecdsa_private_key.len(), 32, "ECDSA private key must be exactly 32 bytes (256 bits)");
        assert_eq!(ecdsa_private_key.len() * 8, 256, "ECDSA private key must be exactly 256 bits");
        println!("✅ ECDSA private key length verification passed: {} bytes ({} bits)", 
                ecdsa_private_key.len(), ecdsa_private_key.len() * 8);
        
        // Use the same 256-bit private key for both algorithms
        let ed25519_key = SigningKey::from_bytes(&ecdsa_private_key[..32].try_into().unwrap());
        let ed25519_public = ed25519_key.verifying_key();
        
        println!("\n📊 Ed25519 Private Key Information:");
        println!("   Public Key (hex): {}", hex::encode(ed25519_public.to_bytes()));
        println!("   Private Key Length: {} bytes", ed25519_key.to_bytes().len());
        println!("   Private Key Bits: {} bits", ed25519_key.to_bytes().len() * 8);
        
        // Assert that Ed25519 also uses 32-byte private key
        assert_eq!(ed25519_key.to_bytes().len(), 32, "Ed25519 private key must be exactly 32 bytes (256 bits)");
        assert_eq!(ed25519_key.to_bytes().len() * 8, 256, "Ed25519 private key must be exactly 256 bits");
        println!("✅ Ed25519 private key length verification passed: {} bytes ({} bits)", 
                ed25519_key.to_bytes().len(), ed25519_key.to_bytes().len() * 8);
        
        // Verify that both private keys are identical
        assert_eq!(ecdsa_private_key, ed25519_key.to_bytes().into(), "Both private keys must be identical 256-bit integers");
        println!("✅ Private key consistency verification passed: Same 256-bit integer");
        
        // The private key is the same 256-bit integer, but public keys are different
        println!("\n🎯 Conclusion:");
        println!("   ✅ Same 256-bit private key can be used for both algorithms!");
        println!("   📏 Private key space: [0, 2^256-1] (same for both)");
        println!("   🔐 ECDSA public key: Derived using secp256k1 curve");
        println!("   🔐 Ed25519 public key: Derived using Edwards curve");
        println!("   🔄 The difference is in the public key derivation algorithm, not the private key space");
    }
}

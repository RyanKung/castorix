pub mod custody_handlers;
pub mod ens_handlers;
pub mod hub_handlers;
pub mod key_handlers;
pub mod signers_handlers;

use crate::cli::types::*;
use anyhow::Result;

/// CLI command handler
pub struct CliHandler;

impl CliHandler {
    /// Handle key management commands (legacy)
    pub async fn handle_key_command(
        command: KeyCommands,
        key_manager: &crate::core::crypto::key_manager::KeyManager,
    ) -> Result<()> {
        crate::cli::handlers::key_handlers::core::handle_key_command(command, key_manager).await
    }

    /// Handle Hub Ed25519 key management commands
    pub async fn handle_hub_key_command(command: HubKeyCommands) -> Result<()> {
        crate::cli::handlers::key_handlers::hub::handle_hub_key_command(command).await
    }

    /// Handle ENS commands
    pub async fn handle_ens_command(
        command: EnsCommands,
        ens_proof: &crate::ens_proof::EnsProof,
    ) -> Result<()> {
        ens_handlers::handle_ens_command(command, ens_proof).await
    }

    /// Handle Farcaster Hub commands
    pub async fn handle_hub_command(
        command: HubCommands,
        hub_client: &crate::core::client::hub_client::FarcasterClient,
    ) -> Result<()> {
        hub_handlers::handle_hub_command(command, hub_client).await
    }

    /// Handle ECDSA custody key management commands
    pub async fn handle_custody_command(command: CustodyCommands) -> Result<()> {
        custody_handlers::handle_custody_command(command).await
    }

    /// Handle signer management commands
    pub async fn handle_signers_command(
        command: SignersCommands,
        hub_client: &crate::core::client::hub_client::FarcasterClient,
    ) -> Result<()> {
        signers_handlers::handle_signers_command(command, hub_client).await
    }
}

//! Farcaster Signer query tools for MCP

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use crate::core::client::FarcasterClient;
use crate::mcp::error::McpError;
use crate::mcp::error::Result;
use crate::mcp::tools::base::McpTool;
use crate::mcp::types::InputSchema;
use crate::mcp::types::Tool;

/// Context for signer tools
pub struct SignerContext {
    pub client: Arc<FarcasterClient>,
}

impl SignerContext {
    pub fn new(hub_url: String) -> Self {
        Self {
            client: Arc::new(FarcasterClient::read_only(hub_url)),
        }
    }
}

// ============================================================================
// 1. signers_list_local - List local Ed25519 signer keys
// ============================================================================

pub struct SignersListLocalTool;

impl SignersListLocalTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SignersListLocalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpTool for SignersListLocalTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "signers_list_local".to_string(),
            description: "List all locally stored Ed25519 signer keys. Returns FID, public key, and creation date for each signer stored in the local encrypted key store.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({}),
                required: vec![],
            },
        }
    }

    async fn execute(&self, _arguments: Value) -> Result<Value> {
        // Load Ed25519 key manager
        let keys_file =
            crate::core::crypto::encrypted_storage::EncryptedEd25519KeyManager::default_keys_file()
                .map_err(|e| {
                    McpError::InternalError(format!("Failed to get keys file path: {}", e))
                })?;

        let manager =
            crate::core::crypto::encrypted_storage::EncryptedEd25519KeyManager::load_from_file(
                &keys_file,
            )
            .map_err(|e| McpError::InternalError(format!("Failed to load key manager: {}", e)))?;

        // List keys (no password needed for public information)
        let key_infos = manager
            .list_keys_with_info("")
            .map_err(|e| McpError::InternalError(format!("Failed to list keys: {}", e)))?;

        let keys: Vec<Value> = key_infos
            .into_iter()
            .map(|info| {
                json!({
                    "fid": info.fid,
                    "public_key": info.public_key,
                    "created_at": info.created_at,
                    "type": "Ed25519"
                })
            })
            .collect();

        Ok(json!({
            "keys": keys,
            "count": keys.len()
        }))
    }
}

// ============================================================================
// 2. signers_get_info - Get signer information for a FID
// ============================================================================

pub struct SignersGetInfoTool {
    context: Arc<SignerContext>,
}

impl SignersGetInfoTool {
    pub fn new(context: Arc<SignerContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetSignerInfoArgs {
    fid: u64,
}

#[async_trait]
impl McpTool for SignersGetInfoTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "signers_get_info".to_string(),
            description: "Get signer information for a Farcaster ID from the Hub. Returns all Ed25519 public keys that are authorized to sign messages for this FID.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "fid": {
                        "type": "number",
                        "description": "The Farcaster ID (FID) to get signers for"
                    }
                }),
                required: vec!["fid".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetSignerInfoArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid FID: {}", e)))?;

        let signers = self
            .context
            .client
            .get_signers(args.fid)
            .await
            .map_err(|e| McpError::HubConnectionFailed(format!("Failed to get signers: {}", e)))?;

        Ok(json!({
            "fid": args.fid,
            "signers": signers,
            "count": signers.len()
        }))
    }
}

/// Create all signer tools
pub fn create_signer_tools(context: Arc<SignerContext>) -> Vec<Box<dyn McpTool>> {
    vec![
        Box::new(SignersListLocalTool::new()),
        Box::new(SignersGetInfoTool::new(context)),
    ]
}

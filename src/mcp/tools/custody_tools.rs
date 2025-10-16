//! Custody key query tools for MCP

use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;

use crate::mcp::error::McpError;
use crate::mcp::error::Result;
use crate::mcp::tools::base::McpTool;
use crate::mcp::types::InputSchema;
use crate::mcp::types::Tool;

// ============================================================================
// custody_list_local - List local custody keys
// ============================================================================

pub struct CustodyListLocalTool;

impl CustodyListLocalTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CustodyListLocalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl McpTool for CustodyListLocalTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "custody_list_local".to_string(),
            description: "List all locally stored ECDSA custody keys. Returns FID, address, and creation date for each custody key stored in the local encrypted key store.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({}),
                required: vec![],
            },
        }
    }

    async fn execute(&self, _arguments: Value) -> Result<Value> {
        // Load ECDSA key manager
        let keys_file =
            crate::core::crypto::encrypted_storage::EncryptedEthKeyManager::default_keys_file()
                .map_err(|e| {
                    McpError::InternalError(format!("Failed to get keys file path: {}", e))
                })?;

        let manager =
            crate::core::crypto::encrypted_storage::EncryptedEthKeyManager::load_from_file(
                &keys_file,
            )
            .map_err(|e| McpError::InternalError(format!("Failed to load key manager: {}", e)))?;

        // List keys (no password needed for public information)
        let key_list = manager.list_keys();

        let keys: Vec<Value> = key_list
            .into_iter()
            .map(|(fid, created_at, _)| {
                // Note: Address would require password to decrypt
                // We only return FID and creation info
                json!({
                    "fid": fid,
                    "created_at": created_at,
                    "type": "ECDSA_Custody",
                    "note": "Address requires password to decrypt"
                })
            })
            .collect();

        Ok(json!({
            "keys": keys,
            "count": keys.len()
        }))
    }
}

/// Create all custody tools
pub fn create_custody_tools() -> Vec<Box<dyn McpTool>> {
    vec![Box::new(CustodyListLocalTool::new())]
}

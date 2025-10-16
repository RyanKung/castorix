//! ENS domain query tools for MCP

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use crate::core::crypto::key_manager::KeyManager;
use crate::ens_proof::EnsProof;
use crate::mcp::error::McpError;
use crate::mcp::error::Result;
use crate::mcp::tools::base::McpTool;
use crate::mcp::types::InputSchema;
use crate::mcp::types::Tool;

/// Context for ENS tools
pub struct EnsContext {
    pub ens_proof: Arc<EnsProof>,
}

impl EnsContext {
    pub fn new(eth_rpc_url: String, _base_rpc_url: String) -> Result<Self> {
        // Create a dummy key manager for read-only operations
        let dummy_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let key_manager = KeyManager::from_private_key(dummy_key)
            .map_err(|e| McpError::InternalError(format!("Failed to create key manager: {}", e)))?;

        // For now, use ETH mainnet RPC for ENS
        // In future, could support both ETH and Base
        let ens_proof = EnsProof::new(key_manager, eth_rpc_url);

        Ok(Self {
            ens_proof: Arc::new(ens_proof),
        })
    }
}

// ============================================================================
// 1. ens_resolve_domain - Resolve ENS domain to address
// ============================================================================

pub struct EnsResolveDomainTool {
    context: Arc<EnsContext>,
}

impl EnsResolveDomainTool {
    pub fn new(context: Arc<EnsContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct ResolveDomainArgs {
    domain: String,
}

#[async_trait]
impl McpTool for EnsResolveDomainTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "ens_resolve_domain".to_string(),
            description: "Resolve an ENS domain name to an Ethereum address. Works with .eth domains and Base .base.eth subdomains.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "domain": {
                        "type": "string",
                        "description": "ENS domain to resolve (e.g., 'vitalik.eth' or 'alice.base.eth')"
                    }
                }),
                required: vec!["domain".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: ResolveDomainArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        let address = self
            .context
            .ens_proof
            .resolve_ens(&args.domain)
            .await
            .map_err(|e| McpError::DataNotFound(format!("Failed to resolve domain: {}", e)))?;

        Ok(json!({
            "domain": args.domain,
            "address": format!("{:?}", address)
        }))
    }
}

// ============================================================================
// 2. ens_check_base_subdomain - Check Base subdomain
// ============================================================================

pub struct EnsCheckBaseSubdomainTool {
    context: Arc<EnsContext>,
}

impl EnsCheckBaseSubdomainTool {
    pub fn new(context: Arc<EnsContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct CheckBaseSubdomainArgs {
    domain: String,
}

#[async_trait]
impl McpTool for EnsCheckBaseSubdomainTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "ens_check_base_subdomain".to_string(),
            description: "Check if a Base subdomain (*.base.eth) exists and get its owner address."
                .to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "domain": {
                        "type": "string",
                        "description": "Base subdomain to check (e.g., 'alice.base.eth')"
                    }
                }),
                required: vec!["domain".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: CheckBaseSubdomainArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        match self
            .context
            .ens_proof
            .query_base_ens_contract(&args.domain)
            .await
        {
            Ok(Some(owner)) => Ok(json!({
                "domain": args.domain,
                "exists": true,
                "owner": owner
            })),
            Ok(None) => Ok(json!({
                "domain": args.domain,
                "exists": false
            })),
            Err(e) => Err(McpError::RpcConnectionFailed(format!(
                "Failed to query Base contract: {}",
                e
            ))),
        }
    }
}

// ============================================================================
// 3. ens_verify_ownership - Verify domain ownership
// ============================================================================

pub struct EnsVerifyOwnershipTool {
    context: Arc<EnsContext>,
}

impl EnsVerifyOwnershipTool {
    pub fn new(context: Arc<EnsContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct VerifyOwnershipArgs {
    domain: String,
    address: String,
}

#[async_trait]
impl McpTool for EnsVerifyOwnershipTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "ens_verify_ownership".to_string(),
            description: "Verify if an address owns a specific ENS domain. Resolves the domain and checks if it matches the provided address.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "domain": {
                        "type": "string",
                        "description": "ENS domain to check (e.g., 'vitalik.eth')"
                    },
                    "address": {
                        "type": "string",
                        "description": "Ethereum address to verify (0x...)"
                    }
                }),
                required: vec!["domain".to_string(), "address".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: VerifyOwnershipArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        // Parse the provided address
        let check_address: ethers::types::Address = args
            .address
            .parse()
            .map_err(|e| McpError::InvalidArguments(format!("Invalid address format: {}", e)))?;

        // Resolve the domain
        let resolved_address = self
            .context
            .ens_proof
            .resolve_ens(&args.domain)
            .await
            .map_err(|e| McpError::DataNotFound(format!("Failed to resolve domain: {}", e)))?;

        let owns_domain = resolved_address == check_address;

        Ok(json!({
            "domain": args.domain,
            "provided_address": args.address,
            "resolved_address": format!("{:?}", resolved_address),
            "owns_domain": owns_domain
        }))
    }
}

/// Create all ENS tools
pub fn create_ens_tools(
    eth_rpc_url: String,
    base_rpc_url: String,
) -> Result<Vec<Box<dyn McpTool>>> {
    let context = Arc::new(EnsContext::new(eth_rpc_url, base_rpc_url)?);

    Ok(vec![
        Box::new(EnsResolveDomainTool::new(context.clone())),
        Box::new(EnsCheckBaseSubdomainTool::new(context.clone())),
        Box::new(EnsVerifyOwnershipTool::new(context)),
    ])
}

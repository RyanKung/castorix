//! Farcaster contract query tools for MCP

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use crate::farcaster::ContractAddresses;
use crate::farcaster::FarcasterContractClient;
use crate::mcp::error::McpError;
use crate::mcp::error::Result;
use crate::mcp::tools::base::McpTool;
use crate::mcp::types::InputSchema;
use crate::mcp::types::Tool;

/// Context for contract tools
pub struct ContractContext {
    pub client: Arc<FarcasterContractClient>,
}

impl ContractContext {
    pub fn new(rpc_url: String) -> Result<Self> {
        let addresses = ContractAddresses::default(); // Optimism mainnet
        let client = FarcasterContractClient::new(rpc_url, addresses).map_err(|e| {
            McpError::RpcConnectionFailed(format!("Failed to create contract client: {}", e))
        })?;

        Ok(Self {
            client: Arc::new(client),
        })
    }
}

// ============================================================================
// 1. fid_get_price - Get FID registration price
// ============================================================================

pub struct FidGetPriceTool {
    context: Arc<ContractContext>,
}

impl FidGetPriceTool {
    pub fn new(context: Arc<ContractContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl McpTool for FidGetPriceTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "fid_get_price".to_string(),
            description: "Get the current price to register a new Farcaster ID (FID). Returns the price in Wei and ETH.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({}),
                required: vec![],
            },
        }
    }

    async fn execute(&self, _arguments: Value) -> Result<Value> {
        let price_wei = self
            .context
            .client
            .get_registration_price()
            .await
            .map_err(|e| {
                McpError::RpcConnectionFailed(format!("Failed to get FID price: {}", e))
            })?;

        // Convert to ETH (divide by 10^18)
        let price_eth = price_wei.as_u128() as f64 / 1_000_000_000_000_000_000.0;

        Ok(json!({
            "price_wei": price_wei.to_string(),
            "price_eth": price_eth,
            "currency": "ETH"
        }))
    }
}

// ============================================================================
// 2. storage_get_price - Get storage rental price
// ============================================================================

pub struct StorageGetPriceTool {
    context: Arc<ContractContext>,
}

impl StorageGetPriceTool {
    pub fn new(context: Arc<ContractContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct GetStoragePriceArgs {
    units: u32,
}

#[async_trait]
impl McpTool for StorageGetPriceTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "storage_get_price".to_string(),
            description: "Get the price to rent storage units for a Farcaster ID. Returns the price in Wei and ETH for the specified number of units.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "units": {
                        "type": "number",
                        "description": "Number of storage units to check price for"
                    }
                }),
                required: vec!["units".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: GetStoragePriceArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        let price_wei = self
            .context
            .client
            .get_storage_price(args.units as u64)
            .await
            .map_err(|e| {
                McpError::RpcConnectionFailed(format!("Failed to get storage price: {}", e))
            })?;

        // Convert to ETH
        let price_eth = price_wei.as_u128() as f64 / 1_000_000_000_000_000_000.0;

        Ok(json!({
            "units": args.units,
            "price_wei": price_wei.to_string(),
            "price_eth": price_eth,
            "currency": "ETH"
        }))
    }
}

// ============================================================================
// 3. fid_check_address - Check if address has FID
// ============================================================================

pub struct FidCheckAddressTool {
    context: Arc<ContractContext>,
}

impl FidCheckAddressTool {
    pub fn new(context: Arc<ContractContext>) -> Self {
        Self { context }
    }
}

#[derive(Debug, Deserialize)]
struct CheckAddressArgs {
    address: String,
}

#[async_trait]
impl McpTool for FidCheckAddressTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "fid_check_address".to_string(),
            description: "Check if an Ethereum address owns a Farcaster ID. Returns the FID if the address has one registered.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({
                    "address": {
                        "type": "string",
                        "description": "Ethereum address to check (0x...)"
                    }
                }),
                required: vec!["address".to_string()],
            },
        }
    }

    async fn execute(&self, arguments: Value) -> Result<Value> {
        let args: CheckAddressArgs = serde_json::from_value(arguments)
            .map_err(|e| McpError::InvalidArguments(format!("Invalid arguments: {}", e)))?;

        // Parse address
        let address: ethers::types::Address = args
            .address
            .parse()
            .map_err(|e| McpError::InvalidArguments(format!("Invalid address format: {}", e)))?;

        let fid_option = self
            .context
            .client
            .address_has_fid(address)
            .await
            .map_err(|e| {
                McpError::RpcConnectionFailed(format!("Failed to check address: {}", e))
            })?;

        match fid_option {
            Some(fid) => Ok(json!({
                "address": args.address,
                "has_fid": true,
                "fid": fid
            })),
            None => Ok(json!({
                "address": args.address,
                "has_fid": false
            })),
        }
    }
}

// ============================================================================
// 4. storage_check_units - Check rented storage units
// ============================================================================

pub struct StorageCheckUnitsTool {
    context: Arc<ContractContext>,
}

impl StorageCheckUnitsTool {
    pub fn new(context: Arc<ContractContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl McpTool for StorageCheckUnitsTool {
    fn definition(&self) -> Tool {
        Tool {
            name: "storage_check_units".to_string(),
            description: "Check total rented storage units across all Farcaster IDs. Returns the total number of storage units currently rented on the network.".to_string(),
            input_schema: InputSchema {
                type_: "object".to_string(),
                properties: json!({}),
                required: vec![],
            },
        }
    }

    async fn execute(&self, _arguments: Value) -> Result<Value> {
        let units_result = self
            .context
            .client
            .storage_registry
            .rented_units()
            .await
            .map_err(|e| {
                McpError::RpcConnectionFailed(format!("Failed to get rented units: {}", e))
            })?;

        let units = match units_result {
            crate::farcaster::ContractResult::Success(u) => u,
            crate::farcaster::ContractResult::Error(e) => {
                return Err(McpError::RpcConnectionFailed(format!(
                    "Contract error: {}",
                    e
                )));
            }
        };

        Ok(json!({
            "total_rented_units": units,
            "note": "Total storage units rented across all FIDs"
        }))
    }
}

/// Create all contract tools
pub fn create_contract_tools(rpc_url: String) -> Result<Vec<Box<dyn McpTool>>> {
    let context = Arc::new(ContractContext::new(rpc_url)?);

    Ok(vec![
        Box::new(FidGetPriceTool::new(context.clone())),
        Box::new(StorageGetPriceTool::new(context.clone())),
        Box::new(FidCheckAddressTool::new(context.clone())),
        Box::new(StorageCheckUnitsTool::new(context)),
    ])
}

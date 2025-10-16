//! Model Context Protocol (MCP) implementation for Castorix
//!
//! This module implements the MCP server that exposes Castorix's
//! Farcaster query capabilities to AI assistants like Claude.

pub mod error;
pub mod registry;
pub mod server;
pub mod tools;
pub mod types;
pub mod utils;

pub use error::McpError;
pub use error::Result;
pub use registry::ToolRegistry;
pub use server::McpServer;
pub use tools::create_contract_tools;
pub use tools::create_custody_tools;
pub use tools::create_ens_tools;
pub use tools::create_hub_tools;
pub use tools::create_signer_tools;
pub use tools::HubContext;
pub use tools::McpTool;
pub use tools::SignerContext;
pub use types::JsonRpcRequest;
pub use types::JsonRpcResponse;
pub use types::Tool;

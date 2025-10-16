//! MCP (Model Context Protocol) command handlers

use std::sync::Arc;

use anyhow::Result;
use tracing::info;
use tracing::Level;

use crate::cli::types::McpCommands;
use crate::mcp::create_contract_tools;
use crate::mcp::create_custody_tools;
use crate::mcp::create_ens_tools;
use crate::mcp::create_hub_tools;
use crate::mcp::create_signer_tools;
use crate::mcp::HubContext;
use crate::mcp::McpServer;
use crate::mcp::SignerContext;
use crate::mcp::ToolRegistry;

/// Handle MCP commands
pub async fn handle_mcp_command(command: McpCommands, hub_url: String) -> Result<()> {
    match command {
        McpCommands::Serve => {
            // Initialize tracing
            let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
            tracing_subscriber::fmt()
                .with_max_level(match log_level.as_str() {
                    "trace" => Level::TRACE,
                    "debug" => Level::DEBUG,
                    "info" => Level::INFO,
                    "warn" => Level::WARN,
                    "error" => Level::ERROR,
                    _ => Level::INFO,
                })
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false)
                .init();

            info!(
                "Starting Castorix MCP Server v{}",
                env!("CARGO_PKG_VERSION")
            );
            info!("Connecting to Farcaster Hub: {}", hub_url);

            // Create contexts
            let hub_context = Arc::new(HubContext::new(hub_url.clone()));
            let signer_context = Arc::new(SignerContext::new(hub_url));

            // Create tool registry
            let mut registry = ToolRegistry::new();

            // Register Hub tools
            info!("Registering Hub tools...");
            let hub_tools = create_hub_tools(hub_context);
            registry.register_all(hub_tools);

            // Register Signer tools
            info!("Registering Signer tools...");
            let signer_tools = create_signer_tools(signer_context);
            registry.register_all(signer_tools);

            // Register Custody tools
            info!("Registering Custody tools...");
            let custody_tools = create_custody_tools();
            registry.register_all(custody_tools);

            // Register Contract tools (FID and Storage queries)
            info!("Registering Contract tools...");
            if let Ok(op_rpc_url) = std::env::var("ETH_OP_RPC_URL") {
                match create_contract_tools(op_rpc_url) {
                    Ok(contract_tools) => {
                        registry.register_all(contract_tools);
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not load contract tools: {}. Skipping contract-based queries.", e);
                    }
                }
            } else {
                eprintln!("Warning: ETH_OP_RPC_URL not set. Skipping contract-based queries.");
            }

            // Register ENS tools
            info!("Registering ENS tools...");
            if let (Ok(eth_rpc_url), Ok(base_rpc_url)) = (
                std::env::var("ETH_RPC_URL"),
                std::env::var("ETH_BASE_RPC_URL"),
            ) {
                match create_ens_tools(eth_rpc_url, base_rpc_url) {
                    Ok(ens_tools) => {
                        registry.register_all(ens_tools);
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Could not load ENS tools: {}. Skipping ENS queries.",
                            e
                        );
                    }
                }
            } else {
                eprintln!(
                    "Warning: ETH_RPC_URL or ETH_BASE_RPC_URL not set. Skipping ENS queries."
                );
            }

            info!("Registered {} tools total", registry.count());

            // Create and start MCP server
            let server = McpServer::new(registry);

            info!("MCP server ready (stdio mode)");
            info!("Waiting for JSON-RPC requests on stdin...");

            // Run server in stdio mode
            server.run_stdio().await?;

            info!("MCP server shutting down");
            Ok(())
        }
    }
}

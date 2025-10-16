//! MCP server implementation
//!
//! This module implements the core MCP server that handles JSON-RPC requests
//! and manages tool execution.

use std::sync::Arc;

use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::sync::RwLock;
use tracing::debug;
use tracing::error;
use tracing::info;

use crate::mcp::error::McpError;
use crate::mcp::registry::ToolRegistry;
use crate::mcp::types::JsonRpcError;
use crate::mcp::types::JsonRpcRequest;
use crate::mcp::types::JsonRpcResponse;
use crate::mcp::types::ToolCallParams;
use crate::mcp::types::ToolCallResponse;

/// MCP server that handles JSON-RPC requests
pub struct McpServer {
    registry: Arc<RwLock<ToolRegistry>>,
}

impl McpServer {
    /// Create a new MCP server with a tool registry
    pub fn new(registry: ToolRegistry) -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry)),
        }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        debug!("Handling request: method={}", request.method);

        match request.method.as_str() {
            "tools/list" => self.handle_tools_list(request.id).await,
            "tools/call" => self.handle_tool_call(request.id, request.params).await,
            "initialize" => self.handle_initialize(request.id).await,
            _ => {
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method))
            }
        }
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        let registry = self.registry.read().await;
        let tools_list = registry.list_tools();

        match serde_json::to_value(tools_list) {
            Ok(result) => {
                info!("Listed {} tools", registry.count());
                JsonRpcResponse::success(id, result)
            }
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::internal_error(&format!("Failed to serialize tools: {}", e)),
            ),
        }
    }

    /// Handle tools/call request
    async fn handle_tool_call(
        &self,
        id: Option<serde_json::Value>,
        params: Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing parameters"),
                )
            }
        };

        let tool_params: ToolCallParams = match serde_json::from_value(params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(&format!("Invalid parameters: {}", e)),
                )
            }
        };

        debug!("Calling tool: {}", tool_params.name);

        // Get the tool from registry
        let registry = self.registry.read().await;
        let tool = match registry.get(&tool_params.name) {
            Some(t) => t,
            None => {
                return JsonRpcResponse::error(id, JsonRpcError::tool_not_found(&tool_params.name))
            }
        };

        // Execute the tool
        match tool.execute(tool_params.arguments).await {
            Ok(result) => {
                info!("Tool '{}' executed successfully", tool_params.name);

                // Format result as MCP tool response
                let text =
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| result.to_string());
                let tool_response = ToolCallResponse::text(text);

                match serde_json::to_value(tool_response) {
                    Ok(response_value) => JsonRpcResponse::success(id, response_value),
                    Err(e) => JsonRpcResponse::error(
                        id,
                        JsonRpcError::internal_error(&format!(
                            "Failed to serialize response: {}",
                            e
                        )),
                    ),
                }
            }
            Err(e) => {
                error!("Tool '{}' execution failed: {}", tool_params.name, e);
                JsonRpcResponse::error(
                    id,
                    JsonRpcError::tool_execution_failed(&tool_params.name, &e.to_string()),
                )
            }
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&self, id: Option<serde_json::Value>) -> JsonRpcResponse {
        info!("Initializing MCP server");

        let init_response = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "castorix-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        JsonRpcResponse::success(id, init_response)
    }

    /// Run the server in stdio mode (for Claude Desktop integration)
    pub async fn run_stdio(&self) -> Result<(), McpError> {
        info!("Starting MCP server in stdio mode");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("EOF reached, shutting down");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received: {}", trimmed);

                    // Parse JSON-RPC request
                    let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("Failed to parse request: {}", e);
                            let error_response = JsonRpcResponse::error(
                                None,
                                JsonRpcError::parse_error(&e.to_string()),
                            );
                            self.send_response(&mut stdout, &error_response).await?;
                            continue;
                        }
                    };

                    // Handle request
                    let response = self.handle_request(request).await;

                    // Send response
                    self.send_response(&mut stdout, &response).await?;
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    return Err(McpError::IoError(e));
                }
            }
        }

        Ok(())
    }

    /// Send a JSON-RPC response to stdout
    async fn send_response(
        &self,
        stdout: &mut tokio::io::Stdout,
        response: &JsonRpcResponse,
    ) -> Result<(), McpError> {
        let json = serde_json::to_string(response)?;
        debug!("Sending: {}", json);

        stdout.write_all(json.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_handle_tools_list() {
        let registry = ToolRegistry::new();
        let server = McpServer::new(registry);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let registry = ToolRegistry::new();
        let server = McpServer::new(registry);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_none());
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let registry = ToolRegistry::new();
        let server = McpServer::new(registry);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());

        let result = response.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "castorix-mcp-server");
    }
}

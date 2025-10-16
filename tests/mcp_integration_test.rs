//! MCP integration tests

use std::sync::Arc;

use castorix::mcp::create_custody_tools;
use castorix::mcp::create_ens_tools;
use castorix::mcp::create_hub_tools;
use castorix::mcp::create_signer_tools;
use castorix::mcp::HubContext;
use castorix::mcp::McpServer;
use castorix::mcp::SignerContext;
use castorix::mcp::ToolRegistry;
use serde_json::json;

#[tokio::test]
async fn test_mcp_server_initialization() {
    let hub_url = "https://hub-api.neynar.com".to_string();
    let hub_context = Arc::new(HubContext::new(hub_url.clone()));
    let signer_context = Arc::new(SignerContext::new(hub_url));

    let mut registry = ToolRegistry::new();

    // Register all tool types
    let hub_tools = create_hub_tools(hub_context);
    registry.register_all(hub_tools);

    let signer_tools = create_signer_tools(signer_context);
    registry.register_all(signer_tools);

    let custody_tools = create_custody_tools();
    registry.register_all(custody_tools);

    let server = McpServer::new(registry);

    // Test tools/list
    let request = castorix::mcp::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.result.is_some());
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();

    // With ENS tools: 12 Hub + 2 Signer + 1 Custody + 3 ENS = 18 minimum
    // Without ENS tools: 12 Hub + 2 Signer + 1 Custody = 15 minimum
    assert!(
        tools.len() >= 15,
        "Should have at least 15 tools, found {}",
        tools.len()
    );
}

#[tokio::test]
async fn test_mcp_tools_list() {
    let hub_url = "https://hub-api.neynar.com".to_string();
    let hub_context = Arc::new(HubContext::new(hub_url.clone()));
    let signer_context = Arc::new(SignerContext::new(hub_url));

    let mut registry = ToolRegistry::new();

    // Register all tool types
    let hub_tools = create_hub_tools(hub_context);
    registry.register_all(hub_tools);

    let signer_tools = create_signer_tools(signer_context);
    registry.register_all(signer_tools);

    let custody_tools = create_custody_tools();
    registry.register_all(custody_tools);

    // Register ENS tools if environment variables are available
    if let (Ok(eth_rpc_url), Ok(base_rpc_url)) = (
        std::env::var("ETH_RPC_URL"),
        std::env::var("ETH_BASE_RPC_URL"),
    ) {
        if let Ok(ens_tools) = create_ens_tools(eth_rpc_url, base_rpc_url) {
            registry.register_all(ens_tools);
        }
    }

    let server = McpServer::new(registry);

    let request = castorix::mcp::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    // Phase 1 tools
    assert!(tool_names.contains(&"hub_get_user"));
    assert!(tool_names.contains(&"hub_get_profile"));
    assert!(tool_names.contains(&"hub_get_stats"));
    assert!(tool_names.contains(&"hub_get_followers"));
    assert!(tool_names.contains(&"hub_get_following"));

    // Phase 2 tools
    assert!(tool_names.contains(&"hub_get_eth_addresses"));
    assert!(tool_names.contains(&"hub_get_custody_address"));
    assert!(tool_names.contains(&"hub_get_info"));
    assert!(tool_names.contains(&"hub_get_ens_domains"));
    assert!(tool_names.contains(&"hub_check_spam"));
    assert!(tool_names.contains(&"hub_get_spam_stats"));

    // Bonus tools
    assert!(tool_names.contains(&"hub_get_casts"));

    // Phase 4 tools
    assert!(tool_names.contains(&"signers_list_local"));
    assert!(tool_names.contains(&"signers_get_info"));
    assert!(tool_names.contains(&"custody_list_local"));
}

#[tokio::test]
async fn test_mcp_unknown_method() {
    let registry = ToolRegistry::new();
    let server = McpServer::new(registry);

    let request = castorix::mcp::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "unknown/method".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.result.is_none());
    assert!(response.error.is_some());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32601); // Method not found
}

#[tokio::test]
async fn test_mcp_initialize() {
    let registry = ToolRegistry::new();
    let server = McpServer::new(registry);

    let request = castorix::mcp::JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.result.is_some());

    let result = response.result.unwrap();
    assert_eq!(result["serverInfo"]["name"], "castorix-mcp-server");
    assert_eq!(result["protocolVersion"], "2024-11-05");
}

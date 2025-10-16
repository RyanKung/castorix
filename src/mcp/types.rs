//! MCP (Model Context Protocol) type definitions
//!
//! This module defines the core types used in the MCP protocol implementation.

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: InputSchema,
}

/// Input schema for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSchema {
    #[serde(rename = "type")]
    pub type_: String,
    pub properties: Value,
    pub required: Vec<String>,
}

/// Tool call request parameters
#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    pub arguments: Value,
}

/// Tool call response
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResponse {
    pub content: Vec<Content>,
}

impl ToolCallResponse {
    pub fn text(text: String) -> Self {
        Self {
            content: vec![Content {
                type_: "text".to_string(),
                text,
            }],
        }
    }
}

/// Response content
#[derive(Debug, Clone, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

/// Tools list response
#[derive(Debug, Clone, Serialize)]
pub struct ToolsListResponse {
    pub tools: Vec<Tool>,
}

/// MCP error codes
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    // JSON-RPC standard errors
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,

    // MCP application errors
    ToolNotFound = -32000,
    ToolExecutionFailed = -32001,
    InvalidArguments = -32002,
    RateLimitExceeded = -32003,
    Unauthorized = -32004,

    // External service errors
    HubConnectionFailed = -32100,
    RpcConnectionFailed = -32101,
    DataNotFound = -32102,
}

impl JsonRpcError {
    pub fn parse_error(message: &str) -> Self {
        Self {
            code: ErrorCode::ParseError as i32,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn invalid_request(message: &str) -> Self {
        Self {
            code: ErrorCode::InvalidRequest as i32,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: ErrorCode::MethodNotFound as i32,
            message: format!("Method '{}' not found", method),
            data: None,
        }
    }

    pub fn invalid_params(message: &str) -> Self {
        Self {
            code: ErrorCode::InvalidParams as i32,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn internal_error(message: &str) -> Self {
        Self {
            code: ErrorCode::InternalError as i32,
            message: message.to_string(),
            data: None,
        }
    }

    pub fn tool_not_found(tool_name: &str) -> Self {
        Self {
            code: ErrorCode::ToolNotFound as i32,
            message: format!("Tool '{}' not found", tool_name),
            data: Some(serde_json::json!({ "tool": tool_name })),
        }
    }

    pub fn tool_execution_failed(tool_name: &str, reason: &str) -> Self {
        Self {
            code: ErrorCode::ToolExecutionFailed as i32,
            message: format!("Tool '{}' execution failed", tool_name),
            data: Some(serde_json::json!({
                "tool": tool_name,
                "reason": reason
            })),
        }
    }

    pub fn invalid_arguments(tool_name: &str, reason: &str) -> Self {
        Self {
            code: ErrorCode::InvalidArguments as i32,
            message: format!("Invalid arguments for tool '{}'", tool_name),
            data: Some(serde_json::json!({
                "tool": tool_name,
                "reason": reason
            })),
        }
    }

    pub fn data_not_found(resource: &str) -> Self {
        Self {
            code: ErrorCode::DataNotFound as i32,
            message: format!("Data not found: {}", resource),
            data: Some(serde_json::json!({ "resource": resource })),
        }
    }
}

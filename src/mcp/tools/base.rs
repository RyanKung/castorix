//! Base trait for MCP tools

use async_trait::async_trait;
use serde_json::Value;

use crate::mcp::error::Result;
use crate::mcp::types::Tool;

/// Base trait for all MCP tools
#[async_trait]
pub trait McpTool: Send + Sync {
    /// Get the tool definition (name, description, input schema)
    fn definition(&self) -> Tool;

    /// Execute the tool with given arguments
    async fn execute(&self, arguments: Value) -> Result<Value>;

    /// Get the tool name
    fn name(&self) -> String {
        self.definition().name.clone()
    }

    /// Validate arguments before execution (optional)
    fn validate_arguments(&self, _arguments: &Value) -> Result<()> {
        Ok(())
    }
}

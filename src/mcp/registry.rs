//! MCP tool registry
//!
//! This module manages the registration and lookup of MCP tools.

use std::collections::HashMap;
use std::sync::Arc;

use crate::mcp::tools::McpTool;
use crate::mcp::types::Tool;
use crate::mcp::types::ToolsListResponse;

/// Tool registry that manages all available MCP tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<Box<dyn McpTool>>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register(&mut self, tool: Box<dyn McpTool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Register multiple tools
    pub fn register_all(&mut self, tools: Vec<Box<dyn McpTool>>) {
        for tool in tools {
            self.register(tool);
        }
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<Box<dyn McpTool>>> {
        self.tools.get(name).cloned()
    }

    /// Get all tool definitions
    pub fn list_tools(&self) -> ToolsListResponse {
        let tools: Vec<Tool> = self.tools.values().map(|tool| tool.definition()).collect();

        ToolsListResponse { tools }
    }

    /// Get the number of registered tools
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Check if a tool exists
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use serde_json::json;
    use serde_json::Value;

    use super::*;
    use crate::mcp::error::Result;
    use crate::mcp::types::InputSchema;

    struct TestTool;

    #[async_trait]
    impl McpTool for TestTool {
        fn definition(&self) -> Tool {
            Tool {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                input_schema: InputSchema {
                    type_: "object".to_string(),
                    properties: json!({}),
                    required: vec![],
                },
            }
        }

        async fn execute(&self, _arguments: Value) -> Result<Value> {
            Ok(json!({"result": "success"}))
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(TestTool));

        assert_eq!(registry.count(), 1);
        assert!(registry.contains("test_tool"));
        assert!(registry.get("test_tool").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_registry_list_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(TestTool));

        let list = registry.list_tools();
        assert_eq!(list.tools.len(), 1);
        assert_eq!(list.tools[0].name, "test_tool");
    }
}

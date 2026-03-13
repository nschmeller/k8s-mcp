//! Tool registry for MCP tools.
//!
//! This module provides the tool registry and handler trait for implementing
//! MCP tools that interact with Kubernetes.
//!
//! # Example
//!
//! ```
//! use k8s_mcp::tools::ToolRegistry;
//!
//! let registry = ToolRegistry::new();
//! assert!(registry.is_empty());
//! assert_eq!(registry.len(), 0);
//! ```

use crate::error::{Error, Result};
use crate::k8s::K8sClient;
use crate::mcp::protocol::{CallToolResult, Tool};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Tool handler trait.
///
/// Implement this trait to create custom MCP tools.
///
/// # Example
///
/// ```
/// use k8s_mcp::tools::registry::{ToolHandler, text_result};
/// use k8s_mcp::mcp::protocol::{Tool, ToolInputSchema, CallToolResult};
/// use async_trait::async_trait;
/// use std::collections::HashMap;
/// use serde_json::Value;
///
/// struct MyTool;
///
/// #[async_trait]
/// impl ToolHandler for MyTool {
///     async fn call(&self, _args: HashMap<String, Value>) -> k8s_mcp::error::Result<CallToolResult> {
///         Ok(text_result("Hello from MyTool"))
///     }
///
///     fn definition(&self) -> Tool {
///         Tool::new("my_tool", "My custom tool", ToolInputSchema::object())
///     }
/// }
/// ```
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Execute the tool with the given arguments.
    async fn call(&self, args: HashMap<String, Value>) -> Result<CallToolResult>;

    /// Get the tool definition.
    fn definition(&self) -> Tool;

    /// Whether this tool requires write access.
    fn is_write_tool(&self) -> bool {
        false
    }
}

/// Type alias for a boxed tool handler.
type BoxedToolHandler = Box<dyn ToolHandler>;

/// Tool registry that manages all available tools.
pub struct ToolRegistry {
    tools: HashMap<String, BoxedToolHandler>,
    write_tools: Vec<String>,
    client: Option<Arc<K8sClient>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
            write_tools: Vec::new(),
            client: None,
        }
    }

    /// Create a tool registry with a Kubernetes client.
    pub fn with_client(client: K8sClient) -> Self {
        ToolRegistry {
            tools: HashMap::new(),
            write_tools: Vec::new(),
            client: Some(Arc::new(client)),
        }
    }

    /// Register a tool handler.
    pub fn register<H: ToolHandler + 'static>(&mut self, handler: H) {
        let tool = handler.definition();
        let name = tool.name.clone();

        if handler.is_write_tool() {
            self.write_tools.push(name.clone());
        }

        info!(
            "Registering tool: {} (write={})",
            name,
            handler.is_write_tool()
        );
        self.tools.insert(name, Box::new(handler));
    }

    /// Get the Kubernetes client.
    pub fn client(&self) -> Option<&K8sClient> {
        self.client.as_deref()
    }

    /// List all registered tools.
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().map(|h| h.definition()).collect()
    }

    /// Call a tool by name.
    pub async fn call_tool(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<CallToolResult> {
        let handler = self
            .tools
            .get(name)
            .ok_or_else(|| Error::Tool(format!("Unknown tool: {}", name)))?;

        debug!("Calling tool: {}", name);
        handler.call(args).await
    }

    /// Check if a tool requires write access.
    pub fn is_write_tool(&self, name: &str) -> bool {
        self.write_tools.contains(&name.to_string())
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro to create tool input schemas.
#[macro_export]
macro_rules! tool_schema {
    () => {
        ToolInputSchema::object()
    };
    ($($name:expr => $prop:expr),+ $(,)?) => {
        ToolInputSchema::object()
            .with_properties(
                std::collections::HashMap::from([
                    $(($name.to_string(), $prop)),+
                ])
            )
    };
}

/// Helper macro to create string property.
#[macro_export]
macro_rules! prop_string {
    () => {
        $crate::mcp::protocol::PropertySchema::string()
    };
    ($desc:expr) => {
        $crate::mcp::protocol::PropertySchema::string().description($desc)
    };
}

/// Helper macro to create integer property.
#[macro_export]
macro_rules! prop_integer {
    () => {
        $crate::mcp::protocol::PropertySchema::integer()
    };
    ($desc:expr) => {
        $crate::mcp::protocol::PropertySchema::integer().description($desc)
    };
}

/// Helper macro to create boolean property.
#[macro_export]
macro_rules! prop_boolean {
    () => {
        $crate::mcp::protocol::PropertySchema::boolean()
    };
    ($desc:expr) => {
        $crate::mcp::protocol::PropertySchema::boolean().description($desc)
    };
}

/// Helper to extract a string argument.
pub fn get_string_arg(args: &HashMap<String, Value>, name: &str) -> Result<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            Error::json_rpc_invalid_params(format!("Missing required parameter: {}", name))
        })
}

/// Helper to extract an optional string argument.
pub fn get_optional_string_arg(args: &HashMap<String, Value>, name: &str) -> Option<String> {
    args.get(name)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Helper to extract an integer argument.
pub fn get_integer_arg(args: &HashMap<String, Value>, name: &str) -> Result<i64> {
    args.get(name).and_then(|v| v.as_i64()).ok_or_else(|| {
        Error::json_rpc_invalid_params(format!("Missing required parameter: {}", name))
    })
}

/// Helper to extract an optional integer argument.
pub fn get_optional_integer_arg(args: &HashMap<String, Value>, name: &str) -> Option<i64> {
    args.get(name).and_then(|v| v.as_i64())
}

/// Helper to extract a boolean argument.
pub fn get_boolean_arg(args: &HashMap<String, Value>, name: &str, default: bool) -> bool {
    args.get(name).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Helper to create a successful text result.
pub fn text_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::text(text)
}

/// Helper to create an error result.
pub fn error_result(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::protocol::ToolInputSchema;

    struct TestTool;

    #[async_trait]
    impl ToolHandler for TestTool {
        async fn call(&self, _args: HashMap<String, Value>) -> Result<CallToolResult> {
            Ok(text_result("test"))
        }

        fn definition(&self) -> Tool {
            Tool::new("test", "A test tool", ToolInputSchema::object())
        }
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        assert_eq!(registry.len(), 1);
        assert!(!registry.is_write_tool("test"));
    }

    #[tokio::test]
    async fn test_registry_call() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool);

        let result = registry.call_tool("test", HashMap::new()).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
    }
}

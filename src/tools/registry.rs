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
    use serde_json::json;

    /// A simple test tool for registry tests.
    struct TestTool {
        name: String,
        is_write: bool,
    }

    impl TestTool {
        fn new(name: &str) -> Self {
            TestTool {
                name: name.to_string(),
                is_write: false,
            }
        }

        fn write_tool(name: &str) -> Self {
            TestTool {
                name: name.to_string(),
                is_write: true,
            }
        }
    }

    #[async_trait]
    impl ToolHandler for TestTool {
        async fn call(&self, _args: HashMap<String, Value>) -> Result<CallToolResult> {
            Ok(text_result(format!("Tool {} called", self.name)))
        }

        fn definition(&self) -> Tool {
            Tool::new(&self.name, "Test tool", ToolInputSchema::object())
        }

        fn is_write_tool(&self) -> bool {
            self.is_write
        }
    }

    // ToolRegistry Tests

    #[test]
    fn test_registry_new() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = ToolRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_registry_register() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool::new("test-tool"));

        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_register_multiple() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool::new("tool-1"));
        registry.register(TestTool::new("tool-2"));
        registry.register(TestTool::new("tool-3"));

        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_registry_list_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool::new("tool-a"));
        registry.register(TestTool::new("tool-b"));

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 2);

        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"tool-a"));
        assert!(names.contains(&"tool-b"));
    }

    #[tokio::test]
    async fn test_registry_call_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool::new("my-tool"));

        let result = registry.call_tool("my-tool", HashMap::new()).await.unwrap();
        assert!(!result.is_error.unwrap_or(true));
    }

    #[tokio::test]
    async fn test_registry_call_unknown_tool() {
        let registry = ToolRegistry::new();

        let result = registry.call_tool("unknown-tool", HashMap::new()).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_is_write_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(TestTool::new("read-tool"));
        registry.register(TestTool::write_tool("write-tool"));

        assert!(!registry.is_write_tool("read-tool"));
        assert!(registry.is_write_tool("write-tool"));
        assert!(!registry.is_write_tool("unknown-tool"));
    }

    #[test]
    fn test_registry_client_none() {
        let registry = ToolRegistry::new();
        assert!(registry.client().is_none());
    }

    // Argument Extraction Tests

    #[test]
    fn test_get_string_arg_present() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), json!("test-value"));

        let result = get_string_arg(&args, "name").unwrap();
        assert_eq!(result, "test-value");
    }

    #[test]
    fn test_get_string_arg_missing() {
        let args = HashMap::<String, Value>::new();

        let result = get_string_arg(&args, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_string_arg_wrong_type() {
        let mut args = HashMap::new();
        args.insert("count".to_string(), json!(42));

        let result = get_string_arg(&args, "count");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_optional_string_arg_present() {
        let mut args = HashMap::new();
        args.insert("namespace".to_string(), json!("default"));

        let result = get_optional_string_arg(&args, "namespace");
        assert_eq!(result, Some("default".to_string()));
    }

    #[test]
    fn test_get_optional_string_arg_missing() {
        let args = HashMap::<String, Value>::new();

        let result = get_optional_string_arg(&args, "namespace");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_optional_string_arg_wrong_type() {
        let mut args = HashMap::new();
        args.insert("flag".to_string(), json!(true));

        let result = get_optional_string_arg(&args, "flag");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_integer_arg_present() {
        let mut args = HashMap::new();
        args.insert("count".to_string(), json!(42));

        let result = get_integer_arg(&args, "count").unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_get_integer_arg_missing() {
        let args = HashMap::<String, Value>::new();

        let result = get_integer_arg(&args, "count");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_integer_arg_wrong_type() {
        let mut args = HashMap::new();
        args.insert("count".to_string(), json!("not-a-number"));

        let result = get_integer_arg(&args, "count");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_optional_integer_arg_present() {
        let mut args = HashMap::new();
        args.insert("limit".to_string(), json!(100));

        let result = get_optional_integer_arg(&args, "limit");
        assert_eq!(result, Some(100));
    }

    #[test]
    fn test_get_optional_integer_arg_missing() {
        let args = HashMap::<String, Value>::new();

        let result = get_optional_integer_arg(&args, "limit");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_boolean_arg_present() {
        let mut args = HashMap::new();
        args.insert("enabled".to_string(), json!(true));

        let result = get_boolean_arg(&args, "enabled", false);
        assert!(result);
    }

    #[test]
    fn test_get_boolean_arg_missing_default() {
        let args = HashMap::<String, Value>::new();

        let result = get_boolean_arg(&args, "enabled", true);
        assert!(result);

        let result = get_boolean_arg(&args, "enabled", false);
        assert!(!result);
    }

    #[test]
    fn test_get_boolean_arg_wrong_type() {
        let mut args = HashMap::new();
        args.insert("enabled".to_string(), json!("yes"));

        let result = get_boolean_arg(&args, "enabled", false);
        assert!(!result); // Should return default
    }

    // Result Helper Tests

    #[test]
    fn test_text_result() {
        let result = text_result("Hello, world!");

        assert!(!result.is_error.unwrap_or(true));
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_error_result() {
        let result = error_result("Something went wrong");

        assert!(result.is_error.unwrap_or(false));
        assert_eq!(result.content.len(), 1);
    }
}

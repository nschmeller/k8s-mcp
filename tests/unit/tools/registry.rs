//! Unit tests for tools/registry.rs.

use async_trait::async_trait;
use k8s_mcp::mcp::protocol::{CallToolResult, Tool, ToolInputSchema};
use k8s_mcp::tools::registry::{
    ToolHandler, ToolRegistry, error_result, get_boolean_arg, get_integer_arg,
    get_optional_integer_arg, get_optional_string_arg, get_string_arg, text_result,
};
use serde_json::{Value, json};
use std::collections::HashMap;

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
    async fn call(&self, _args: HashMap<String, Value>) -> k8s_mcp::error::Result<CallToolResult> {
        Ok(text_result(format!("Tool {} called", self.name)))
    }

    fn definition(&self) -> Tool {
        Tool::new(&self.name, "Test tool", ToolInputSchema::object())
    }

    fn is_write_tool(&self) -> bool {
        self.is_write
    }
}

// ============================================================================
// ToolRegistry Tests
// ============================================================================

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

// ============================================================================
// Argument Extraction Tests
// ============================================================================

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

// ============================================================================
// Result Helper Tests
// ============================================================================

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

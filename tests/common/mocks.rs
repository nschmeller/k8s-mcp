//! Mock implementations for testing.

use async_trait::async_trait;
use k8s_mcp::error::Result;
use k8s_mcp::mcp::protocol::{CallToolResult, Tool, ToolInputSchema};
use k8s_mcp::tools::registry::ToolHandler;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A mock tool handler for testing.
pub struct MockTool {
    name: String,
    description: String,
    result_text: String,
    is_error: bool,
    is_write: bool,
    call_count: AtomicUsize,
}

impl MockTool {
    pub fn new(name: impl Into<String>) -> Self {
        MockTool {
            name: name.into(),
            description: "A mock tool for testing".to_string(),
            result_text: "mock result".to_string(),
            is_error: false,
            is_write: false,
            call_count: AtomicUsize::new(0),
        }
    }

    pub fn with_result_text(mut self, text: impl Into<String>) -> Self {
        self.result_text = text.into();
        self
    }

    pub fn with_error(mut self, error_text: impl Into<String>) -> Self {
        self.result_text = error_text.into();
        self.is_error = true;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_write(mut self, is_write: bool) -> Self {
        self.is_write = is_write;
        self
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ToolHandler for MockTool {
    async fn call(&self, _args: HashMap<String, Value>) -> Result<CallToolResult> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        if self.is_error {
            Ok(CallToolResult::error(&self.result_text))
        } else {
            Ok(CallToolResult::text(&self.result_text))
        }
    }

    fn definition(&self) -> Tool {
        Tool::new(
            self.name.clone(),
            self.description.clone(),
            ToolInputSchema::object(),
        )
    }

    fn is_write_tool(&self) -> bool {
        self.is_write
    }
}

/// Helper to create a simple tool arguments map.
pub fn make_args(pairs: &[(&str, &str)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
        .collect()
}

/// Helper to create tool arguments with various types.
pub fn make_args_typed(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_tool() {
        let tool = MockTool::new("test-tool")
            .with_description("Test description")
            .with_result_text("hello");

        let def = tool.definition();
        assert_eq!(def.name, "test-tool");
        assert_eq!(def.description, "Test description");

        let result = tool.call(HashMap::new()).await.unwrap();
        assert!(!result.is_error.unwrap_or(false));
        assert_eq!(tool.call_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_tool_error() {
        let tool = MockTool::new("error-tool").with_error("test error");

        let result = tool.call(HashMap::new()).await.unwrap();
        assert!(result.is_error.unwrap_or(false));
    }

    #[tokio::test]
    async fn test_mock_tool_write() {
        let tool = MockTool::new("write-tool").with_write(true);
        assert!(tool.is_write_tool());
    }

    #[test]
    fn test_make_args() {
        let args = make_args(&[("name", "test"), ("namespace", "default")]);
        assert_eq!(args.get("name").unwrap().as_str().unwrap(), "test");
        assert_eq!(args.get("namespace").unwrap().as_str().unwrap(), "default");
    }

    #[test]
    fn test_make_args_typed() {
        let args = make_args_typed(&[
            ("name", Value::String("test".to_string())),
            ("count", Value::Number(42.into())),
            ("enabled", Value::Bool(true)),
        ]);
        assert_eq!(args.get("name").unwrap().as_str().unwrap(), "test");
        assert_eq!(args.get("count").unwrap().as_i64().unwrap(), 42);
        assert!(args.get("enabled").unwrap().as_bool().unwrap());
    }
}

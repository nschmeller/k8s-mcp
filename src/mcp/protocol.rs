//! MCP (Model Context Protocol) types and definitions.
//!
//! This module implements the MCP protocol types based on the specification
//! at <https://modelcontextprotocol.io/>
//!
//! # Example
//!
//! ```
//! use k8s_mcp::mcp::protocol::{Tool, ToolInputSchema, PropertySchema};
//!
//! // Create a tool definition
//! let tool = Tool::new(
//!     "my_tool",
//!     "A sample tool",
//!     ToolInputSchema::object()
//!         .with_required(vec!["name".to_string()])
//! );
//!
//! assert_eq!(tool.name, "my_tool");
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// JSON-RPC 2.0 Types
// ============================================================================

/// JSON-RPC 2.0 Request
///
/// Represents a JSON-RPC request message.
///
/// # Example
///
/// ```
/// use k8s_mcp::mcp::protocol::{JsonRpcRequest, RequestId};
///
/// let request = JsonRpcRequest {
///     jsonrpc: "2.0".to_string(),
///     id: Some(RequestId::Number(1)),
///     method: "initialize".to_string(),
///     params: None,
/// };
///
/// assert_eq!(request.method, "initialize");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response (success)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    pub result: Value,
}

/// JSON-RPC 2.0 Error Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    pub error: JsonRpcErrorBody,
}

/// JSON-RPC Error body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Request ID type (can be number or string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

/// MCP Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_info: Option<Implementation>,
}

/// MCP Initialize result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Client capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampling: Option<()>,
}

/// Roots capability (for file system access)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootsCapability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Server capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<()>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Prompts capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Resources capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Tools capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Implementation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

// ============================================================================
// Tools Types
// ============================================================================

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: ToolInputSchema,
}

/// Tool input schema (JSON Schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, PropertySchema>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ToolInputSchema {
    pub fn object() -> Self {
        ToolInputSchema {
            type_: "object".to_string(),
            properties: None,
            required: None,
        }
    }

    pub fn with_properties(mut self, properties: HashMap<String, PropertySchema>) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }
}

/// Property schema for tool inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySchema {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<PropertySchema>>,
    #[serde(rename = "enum", default, skip_serializing_if = "Option::is_none")]
    pub enum_: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(rename = "$ref", default, skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
}

impl PropertySchema {
    pub fn string() -> Self {
        PropertySchema {
            type_: Some("string".to_string()),
            description: None,
            items: None,
            enum_: None,
            default: None,
            minimum: None,
            maximum: None,
            ref_: None,
        }
    }

    pub fn integer() -> Self {
        PropertySchema {
            type_: Some("integer".to_string()),
            description: None,
            items: None,
            enum_: None,
            default: None,
            minimum: None,
            maximum: None,
            ref_: None,
        }
    }

    pub fn boolean() -> Self {
        PropertySchema {
            type_: Some("boolean".to_string()),
            description: None,
            items: None,
            enum_: None,
            default: None,
            minimum: None,
            maximum: None,
            ref_: None,
        }
    }

    pub fn array(items: PropertySchema) -> Self {
        PropertySchema {
            type_: Some("array".to_string()),
            description: None,
            items: Some(Box::new(items)),
            enum_: None,
            default: None,
            minimum: None,
            maximum: None,
            ref_: None,
        }
    }

    pub fn object() -> Self {
        PropertySchema {
            type_: Some("object".to_string()),
            description: None,
            items: None,
            enum_: None,
            default: None,
            minimum: None,
            maximum: None,
            ref_: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn enum_values(mut self, values: Vec<String>) -> Self {
        self.enum_ = Some(values);
        self
    }

    pub fn default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn minimum(mut self, min: f64) -> Self {
        self.minimum = Some(min);
        self
    }

    pub fn maximum(mut self, max: f64) -> Self {
        self.maximum = Some(max);
        self
    }
}

/// Tools list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolParams {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<HashMap<String, Value>>,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content types for tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { resource: ResourceContents },
}

impl Content {
    pub fn text(text: impl Into<String>) -> Self {
        Content::Text { text: text.into() }
    }

    pub fn error(text: impl Into<String>) -> CallToolResult {
        CallToolResult {
            content: vec![Content::text(text)],
            is_error: Some(true),
        }
    }
}

/// Resource contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContents {
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

// ============================================================================
// Resources Types
// ============================================================================

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// List resources result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    pub resources: Vec<Resource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Read resource request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    pub uri: String,
}

/// Read resource result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContents>,
}

// ============================================================================
// Logging Types
// ============================================================================

/// Logging level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

/// Set logging level request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetLevelParams {
    pub level: LoggingLevel,
}

// ============================================================================
// Ping
// ============================================================================

/// Ping params (empty)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PingParams {}

// ============================================================================
// Helper implementations
// ============================================================================

impl Default for JsonRpcRequest {
    fn default() -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: String::new(),
            params: None,
        }
    }
}

impl JsonRpcResponse {
    pub fn new(id: Option<RequestId>, result: Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        }
    }
}

impl JsonRpcError {
    pub fn new(
        id: Option<RequestId>,
        code: i32,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        JsonRpcError {
            jsonrpc: "2.0".to_string(),
            id,
            error: JsonRpcErrorBody {
                code,
                message: message.into(),
                data,
            },
        }
    }

    pub fn method_not_found(id: Option<RequestId>, method: &str) -> Self {
        Self::new(id, -32601, format!("Method not found: {}", method), None)
    }

    pub fn invalid_params(id: Option<RequestId>, message: impl Into<String>) -> Self {
        Self::new(id, -32602, message, None)
    }

    pub fn internal_error(id: Option<RequestId>, message: impl Into<String>) -> Self {
        Self::new(id, -32603, message, None)
    }
}

impl Tool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: ToolInputSchema,
    ) -> Self {
        Tool {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

impl CallToolResult {
    pub fn success(content: Vec<Content>) -> Self {
        CallToolResult {
            content,
            is_error: Some(false),
        }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self::success(vec![Content::text(text)])
    }

    pub fn error(text: impl Into<String>) -> Self {
        CallToolResult {
            content: vec![Content::text(text)],
            is_error: Some(true),
        }
    }
}

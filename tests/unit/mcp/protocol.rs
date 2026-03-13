//! Unit tests for mcp/protocol.rs.

use k8s_mcp::mcp::protocol::*;
use serde_json::json;
use std::collections::HashMap;

// ============================================================================
// JSON-RPC Tests
// ============================================================================

#[test]
fn test_json_rpc_request_default() {
    let request = JsonRpcRequest::default();

    assert_eq!(request.jsonrpc, "2.0");
    assert!(request.id.is_none());
    assert!(request.method.is_empty());
    assert!(request.params.is_none());
}

#[test]
fn test_json_rpc_request_serialization() {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({"protocol_version": "2024-11-05"})),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"method\":\"initialize\""));
}

#[test]
fn test_json_rpc_request_deserialization() {
    let json = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
    let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.id, Some(RequestId::Number(1)));
    assert_eq!(request.method, "tools/list");
    assert!(request.params.is_some());
}

#[test]
fn test_json_rpc_request_string_id() {
    let json = r#"{"jsonrpc":"2.0","id":"abc-123","method":"ping"}"#;
    let request: JsonRpcRequest = serde_json::from_str(json).unwrap();

    assert_eq!(request.id, Some(RequestId::String("abc-123".to_string())));
}

#[test]
fn test_json_rpc_response_new() {
    let response = JsonRpcResponse::new(Some(RequestId::Number(1)), json!({"status": "ok"}));

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, Some(RequestId::Number(1)));
    assert_eq!(response.result, json!({"status": "ok"}));
}

#[test]
fn test_json_rpc_response_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        result: json!({"tools": []}),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"result\""));
    assert!(json.contains("\"tools\":[]"));
}

#[test]
fn test_json_rpc_error_new() {
    let error = JsonRpcError::new(Some(RequestId::Number(1)), -32601, "Method not found", None);

    assert_eq!(error.jsonrpc, "2.0");
    assert_eq!(error.id, Some(RequestId::Number(1)));
    assert_eq!(error.error.code, -32601);
    assert_eq!(error.error.message, "Method not found");
}

#[test]
fn test_json_rpc_error_method_not_found() {
    let error = JsonRpcError::method_not_found(Some(RequestId::Number(1)), "unknown_method");

    assert_eq!(error.error.code, -32601);
    assert!(error.error.message.contains("unknown_method"));
}

#[test]
fn test_json_rpc_error_invalid_params() {
    let error = JsonRpcError::invalid_params(Some(RequestId::Number(1)), "Missing parameter: name");

    assert_eq!(error.error.code, -32602);
    assert!(error.error.message.contains("Missing parameter"));
}

#[test]
fn test_json_rpc_error_internal_error() {
    let error = JsonRpcError::internal_error(Some(RequestId::Number(1)), "Something went wrong");

    assert_eq!(error.error.code, -32603);
    assert!(error.error.message.contains("Something went wrong"));
}

#[test]
fn test_request_id_equality() {
    assert_eq!(RequestId::Number(1), RequestId::Number(1));
    assert_eq!(
        RequestId::String("abc".to_string()),
        RequestId::String("abc".to_string())
    );
    assert_ne!(RequestId::Number(1), RequestId::Number(2));
    assert_ne!(RequestId::Number(1), RequestId::String("1".to_string()));
}

// ============================================================================
// MCP Protocol Tests
// ============================================================================

#[test]
fn test_initialize_params_deserialization() {
    let json = r#"{
        "protocol_version": "2024-11-05",
        "capabilities": {},
        "client_info": {
            "name": "test-client",
            "version": "1.0.0"
        }
    }"#;

    let params: InitializeParams = serde_json::from_str(json).unwrap();
    assert_eq!(params.protocol_version, "2024-11-05");
    assert_eq!(params.client_info.unwrap().name, "test-client");
}

#[test]
fn test_initialize_result_serialization() {
    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(false),
            }),
            ..Default::default()
        },
        server_info: Implementation {
            name: "k8s-mcp".to_string(),
            version: "0.1.0".to_string(),
        },
        instructions: Some("Test instructions".to_string()),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("2024-11-05"));
    assert!(json.contains("k8s-mcp"));
}

#[test]
fn test_client_capabilities_default() {
    let caps = ClientCapabilities::default();
    assert!(caps.experimental.is_none());
    assert!(caps.roots.is_none());
    assert!(caps.sampling.is_none());
}

#[test]
fn test_server_capabilities_default() {
    let caps = ServerCapabilities::default();
    assert!(caps.experimental.is_none());
    assert!(caps.logging.is_none());
    assert!(caps.prompts.is_none());
    assert!(caps.resources.is_none());
    assert!(caps.tools.is_none());
}

// ============================================================================
// Tool Tests
// ============================================================================

#[test]
fn test_tool_new() {
    let tool = Tool::new("test_tool", "A test tool", ToolInputSchema::object());

    assert_eq!(tool.name, "test_tool");
    assert_eq!(tool.description, "A test tool");
}

#[test]
fn test_tool_serialization() {
    let tool = Tool::new(
        "pods_list",
        "List Kubernetes Pods",
        ToolInputSchema::object().with_properties(HashMap::from([(
            "namespace".to_string(),
            PropertySchema::string().description("Namespace"),
        )])),
    );

    let json = serde_json::to_string(&tool).unwrap();
    assert!(json.contains("\"name\":\"pods_list\""));
    assert!(json.contains("\"description\":\"List Kubernetes Pods\""));
    assert!(json.contains("\"input_schema\""));
}

#[test]
fn test_tool_input_schema_object() {
    let schema = ToolInputSchema::object();

    assert_eq!(schema.type_, "object");
    assert!(schema.properties.is_none());
    assert!(schema.required.is_none());
}

#[test]
fn test_tool_input_schema_with_properties() {
    let mut props = HashMap::new();
    props.insert("name".to_string(), PropertySchema::string());
    props.insert("count".to_string(), PropertySchema::integer());

    let schema = ToolInputSchema::object()
        .with_properties(props)
        .with_required(vec!["name".to_string()]);

    assert!(schema.properties.is_some());
    assert!(schema.required.is_some());
    assert_eq!(schema.required.unwrap().len(), 1);
}

#[test]
fn test_property_schema_string() {
    let prop = PropertySchema::string()
        .description("A string property")
        .default(json!("default_value"));

    assert_eq!(prop.type_, Some("string".to_string()));
    assert_eq!(prop.description, Some("A string property".to_string()));
    assert_eq!(prop.default, Some(json!("default_value")));
}

#[test]
fn test_property_schema_integer() {
    let prop = PropertySchema::integer()
        .description("An integer")
        .minimum(0.0)
        .maximum(100.0);

    assert_eq!(prop.type_, Some("integer".to_string()));
    assert_eq!(prop.minimum, Some(0.0));
    assert_eq!(prop.maximum, Some(100.0));
}

#[test]
fn test_property_schema_boolean() {
    let prop = PropertySchema::boolean()
        .description("A boolean flag")
        .default(json!(true));

    assert_eq!(prop.type_, Some("boolean".to_string()));
    assert_eq!(prop.default, Some(json!(true)));
}

#[test]
fn test_property_schema_array() {
    let prop = PropertySchema::array(PropertySchema::string()).description("Array of strings");

    assert_eq!(prop.type_, Some("array".to_string()));
    assert!(prop.items.is_some());
}

#[test]
fn test_property_schema_enum() {
    let prop = PropertySchema::string().enum_values(vec![
        "table".to_string(),
        "json".to_string(),
        "yaml".to_string(),
    ]);

    assert_eq!(
        prop.enum_,
        Some(vec![
            "table".to_string(),
            "json".to_string(),
            "yaml".to_string()
        ])
    );
}

#[test]
fn test_property_schema_serialization() {
    let prop = PropertySchema::string()
        .description("Test property")
        .enum_values(vec!["a".to_string(), "b".to_string()]);

    let json = serde_json::to_string(&prop).unwrap();
    assert!(json.contains("\"type\":\"string\""));
    assert!(json.contains("\"description\":\"Test property\""));
    assert!(json.contains("\"enum\":[\"a\",\"b\"]"));
}

// ============================================================================
// Content Tests
// ============================================================================

#[test]
fn test_content_text() {
    let content = Content::text("Hello, world!");

    match content {
        Content::Text { text } => assert_eq!(text, "Hello, world!"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_content_error() {
    let result = Content::error("Something went wrong");

    assert!(result.is_error.unwrap_or(false));
    match &result.content[0] {
        Content::Text { text } => assert_eq!(text, "Something went wrong"),
        _ => panic!("Expected Text content"),
    }
}

#[test]
fn test_content_serialization() {
    let content = Content::text("Test message");
    let json = serde_json::to_string(&content).unwrap();

    assert!(json.contains("\"type\":\"text\""));
    assert!(json.contains("\"text\":\"Test message\""));
}

#[test]
fn test_call_tool_result_success() {
    let result = CallToolResult::success(vec![Content::text("Success!")]);

    assert!(!result.is_error.unwrap_or(true));
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_call_tool_result_text() {
    let result = CallToolResult::text("Simple text result");

    assert!(!result.is_error.unwrap_or(true));
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_call_tool_result_error() {
    let result = CallToolResult::error("Error occurred");

    assert!(result.is_error.unwrap_or(false));
    assert_eq!(result.content.len(), 1);
}

#[test]
fn test_call_tool_result_serialization() {
    let result = CallToolResult::text("Test result");
    let json = serde_json::to_string(&result).unwrap();

    assert!(json.contains("\"content\""));
    assert!(json.contains("\"is_error\":false"));
}

// ============================================================================
// List Tools Result Tests
// ============================================================================

#[test]
fn test_list_tools_result() {
    let result = ListToolsResult {
        tools: vec![
            Tool::new("tool1", "First tool", ToolInputSchema::object()),
            Tool::new("tool2", "Second tool", ToolInputSchema::object()),
        ],
        next_cursor: None,
    };

    assert_eq!(result.tools.len(), 2);
    assert!(result.next_cursor.is_none());
}

// ============================================================================
// Call Tool Params Tests
// ============================================================================

#[test]
fn test_call_tool_params() {
    let params = CallToolParams {
        name: "pods_list".to_string(),
        arguments: Some(HashMap::from([("namespace".to_string(), json!("default"))])),
    };

    assert_eq!(params.name, "pods_list");
    assert!(params.arguments.is_some());
}

#[test]
fn test_call_tool_params_deserialization() {
    let json = r#"{"name":"test_tool","arguments":{"key":"value"}}"#;
    let params: CallToolParams = serde_json::from_str(json).unwrap();

    assert_eq!(params.name, "test_tool");
    assert_eq!(
        params.arguments.unwrap().get("key").unwrap(),
        &json!("value")
    );
}

// ============================================================================
// Logging Tests
// ============================================================================

#[test]
fn test_logging_level_serialization() {
    assert_eq!(
        serde_json::to_string(&LoggingLevel::Debug).unwrap(),
        "\"debug\""
    );
    assert_eq!(
        serde_json::to_string(&LoggingLevel::Info).unwrap(),
        "\"info\""
    );
    assert_eq!(
        serde_json::to_string(&LoggingLevel::Error).unwrap(),
        "\"error\""
    );
}

#[test]
fn test_logging_level_deserialization() {
    let level: LoggingLevel = serde_json::from_str("\"warning\"").unwrap();
    assert!(matches!(level, LoggingLevel::Warning));
}

// ============================================================================
// Resource Tests
// ============================================================================

#[test]
fn test_resource() {
    let resource = Resource {
        uri: "file:///test.txt".to_string(),
        name: "Test Resource".to_string(),
        description: Some("A test resource".to_string()),
        mime_type: Some("text/plain".to_string()),
    };

    assert_eq!(resource.uri, "file:///test.txt");
    assert_eq!(resource.name, "Test Resource");
}

#[test]
fn test_list_resources_result() {
    let result = ListResourcesResult {
        resources: vec![],
        next_cursor: None,
    };

    assert!(result.resources.is_empty());
}

// ============================================================================
// Implementation Tests
// ============================================================================

#[test]
fn test_implementation() {
    let impl_info = Implementation {
        name: "test-client".to_string(),
        version: "1.0.0".to_string(),
    };

    assert_eq!(impl_info.name, "test-client");
    assert_eq!(impl_info.version, "1.0.0");
}

//! Unit tests for mcp/server.rs.

#![allow(dead_code)]

use k8s_mcp::mcp::{protocol::*, McpServer};
use k8s_mcp::tools::ToolRegistry;
use serde_json::json;

/// Create a basic server for testing.
fn create_test_server() -> McpServer {
    let registry = ToolRegistry::new();
    McpServer::new(registry, false)
}

/// Create a server with write mode enabled.
fn create_test_server_write() -> McpServer {
    let registry = ToolRegistry::new();
    McpServer::new(registry, true)
}

#[tokio::test]
async fn test_handle_initialize() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocol_version": "2024-11-05",
            "capabilities": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"]["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn test_handle_initialize_with_client_info() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocol_version": "2024-11-05",
            "capabilities": {},
            "client_info": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["server_info"]["name"].is_string());
}

#[tokio::test]
async fn test_handle_ping() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "ping".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert_eq!(response_json["result"], json!({}));
}

#[tokio::test]
async fn test_handle_tools_list() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["tools"].is_array());
}

#[tokio::test]
async fn test_handle_tools_call_unknown_tool() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "unknown_tool",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["error"].is_object());
}

#[tokio::test]
async fn test_handle_unknown_method() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "unknown/method".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["error"].is_object());
    assert_eq!(response_json["error"]["code"], -32601); // Method not found
}

#[tokio::test]
async fn test_handle_resources_list() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "resources/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["resources"].is_array());
    assert!(response_json["result"]["resources"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn test_handle_resources_read_not_implemented() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "resources/read".to_string(),
        params: Some(json!({"uri": "test://resource"})),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["error"].is_object());
}

#[tokio::test]
async fn test_handle_prompts_list() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "prompts/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["prompts"].is_array());
}

#[tokio::test]
async fn test_handle_logging_set_level() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "logging/setLevel".to_string(),
        params: Some(json!({
            "level": "debug"
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert_eq!(response_json["result"], json!({}));
}

#[tokio::test]
async fn test_handle_request_with_string_id() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::String("test-id-123".to_string())),
        method: "ping".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert_eq!(response_json["id"], "test-id-123");
}

#[tokio::test]
async fn test_handle_request_missing_params_for_tools_call() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: None, // Missing params
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["error"].is_object());
    assert_eq!(response_json["error"]["code"], -32602); // Invalid params
}

#[tokio::test]
async fn test_server_info_in_response() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocol_version": "2024-11-05",
            "capabilities": {}
        })),
    };

    let response = server.handle_request(request).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();

    assert_eq!(response_json["result"]["server_info"]["name"], "k8s-mcp");
    assert!(response_json["result"]["server_info"]["version"].is_string());
}

#[tokio::test]
async fn test_instructions_in_response() {
    let mut server = create_test_server();

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "initialize".to_string(),
        params: Some(json!({
            "protocol_version": "2024-11-05",
            "capabilities": {}
        })),
    };

    let response = server.handle_request(request).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();

    assert!(response_json["result"]["instructions"].is_string());
    // Should mention read-only mode
    assert!(response_json["result"]["instructions"]
        .as_str()
        .unwrap()
        .contains("Read-only"));
}

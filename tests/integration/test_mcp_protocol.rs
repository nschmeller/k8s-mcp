//! Integration tests for MCP protocol operations.

use crate::integration::{create_server_with_tools, setup_kind};
use k8s_mcp::mcp::protocol::*;
use serde_json::json;

#[tokio::test]
async fn test_initialize_returns_server_info() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

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
    assert_eq!(response_json["result"]["server_info"]["name"], "k8s-mcp");
}

#[tokio::test]
async fn test_tools_list_returns_kubernetes_tools() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let tools = response_json["result"]["tools"].as_array().unwrap();

    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    // Check for actual tool names
    assert!(tool_names.contains(&"pods_list"));
    assert!(tool_names.contains(&"pods_get"));
    assert!(tool_names.contains(&"deployments_list"));
    assert!(tool_names.contains(&"namespaces_list"));
    assert!(tool_names.contains(&"nodes_list"));
    assert!(tool_names.contains(&"configuration_contexts_list"));
}

#[tokio::test]
async fn test_ping_returns_empty_result() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

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
async fn test_unknown_method_returns_error() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

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
    assert_eq!(response_json["error"]["code"], -32601);
}

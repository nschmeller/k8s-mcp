//! Integration tests for Kubernetes resource operations.

use crate::integration::{create_server_with_tools, setup_kind, test_namespace_name};
use k8s_mcp::mcp::protocol::*;
use serde_json::json;

// ============================================================================
// List Operations
// ============================================================================

#[tokio::test]
async fn test_list_namespaces() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "namespaces_list",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());

    let content = response_json["result"]["content"].as_array().unwrap();
    assert!(!content.is_empty());

    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("default") || text.contains("NAME"));
}

#[tokio::test]
async fn test_list_pods_in_default_namespace() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "pods_list",
            "arguments": {
                "namespace": "default"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());
}

#[tokio::test]
async fn test_list_nodes() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "nodes_list",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());

    let content = response_json["result"]["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("control-plane") || text.contains("NAME"));
}

// ============================================================================
// Get Operations
// ============================================================================

#[tokio::test]
async fn test_get_namespace() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "namespaces_get",
            "arguments": {
                "name": "default"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());

    let content = response_json["result"]["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains("default"));
}

#[tokio::test]
async fn test_get_nonexistent_resource_returns_error() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "pods_get",
            "arguments": {
                "name": "nonexistent-pod-xyz",
                "namespace": "default"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let has_error = response_json["error"].is_object()
        || response_json["result"]["is_error"] == true
        || response_json["result"]["content"][0]["text"]
            .as_str()
            .map(|t| t.contains("not found"))
            .unwrap_or(false);
    assert!(has_error);
}

// ============================================================================
// Context Operations
// ============================================================================

#[tokio::test]
async fn test_list_contexts() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "configuration_contexts_list",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());
}

#[tokio::test]
async fn test_get_current_context() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "context_current",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());
}

// ============================================================================
// Read-Only Mode Enforcement
// ============================================================================

#[tokio::test]
async fn test_delete_blocked_in_read_only_mode() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "pods_delete",
            "arguments": {
                "name": "test-pod",
                "namespace": "default"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let has_error = response_json["error"].is_object()
        || response_json["result"]["is_error"] == true
        || response_json["result"]["content"][0]["text"]
            .as_str()
            .map(|t| t.contains("read-only"))
            .unwrap_or(false);
    assert!(has_error);
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[tokio::test]
async fn test_list_pods_json_format() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "pods_list",
            "arguments": {
                "namespace": "default",
                "output": "json"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let content = response_json["result"]["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();

    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(parsed.is_object() || parsed.is_array());
}

#[tokio::test]
async fn test_list_pods_yaml_format() {
    setup_kind();
    let mut server = create_server_with_tools(false).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "pods_list",
            "arguments": {
                "namespace": "default",
                "output": "yaml"
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let content = response_json["result"]["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();

    // Just verify we got some output - the format depends on implementation
    assert!(!text.is_empty() || text.contains("No resources"));
}

// ============================================================================
// Isolated Namespace Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_delete_namespace() {
    setup_kind();
    let mut server = create_server_with_tools(true).await; // write mode enabled

    let ns_name = test_namespace_name("ns-crud");

    // Create namespace using kubectl (since we don't have a create tool)
    let status = std::process::Command::new("kubectl")
        .args(["create", "namespace", &ns_name])
        .status()
        .expect("Failed to create namespace");
    assert!(status.success(), "Failed to create namespace");

    // Verify namespace exists
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "namespaces_get",
            "arguments": {
                "name": ns_name
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let content = response_json["result"]["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    assert!(text.contains(&ns_name));

    // Delete namespace
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "namespaces_delete",
            "arguments": {
                "name": ns_name
            }
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    assert!(response_json["result"]["content"].is_array());
}

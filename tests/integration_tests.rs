//! Integration tests for k8s-mcp.
//!
//! These tests require a Kubernetes cluster (kind) to be running.
//! The test harness will automatically create a kind cluster if needed.
//!
//! Run with: cargo test --test integration_tests -- --ignored
//! Requires a Kubernetes cluster (kind) to be running.

use k8s_mcp::ToolRegistry;
use k8s_mcp::k8s::{ApiDiscovery, K8sClient, K8sConfig};
use k8s_mcp::mcp::McpServer;
use k8s_mcp::mcp::protocol::*;
use k8s_mcp::tools::register_all_tools;
use serde_json::json;
use std::process::Command;
use std::sync::Arc;
use std::sync::Once;
use tokio::sync::RwLock;

const KIND_CLUSTER_NAME: &str = "k8s-mcp-test";

static INIT: Once = Once::new();

// ============================================================================
// Kind Cluster Management
// ============================================================================

/// Ensure a kind cluster is running and selected as current context.
/// This is idempotent and thread-safe - only creates the cluster once.
fn setup_kind() {
    INIT.call_once(|| {
        // Check if kind is installed
        if !is_kind_installed() {
            panic!("kind is not installed. Install with: go install sigs.k8s.io/kind@latest");
        }

        // Check if our test cluster exists
        if !kind_cluster_exists() {
            println!("\nCreating kind cluster '{}'...", KIND_CLUSTER_NAME);
            create_kind_cluster();
        }

        // Switch to kind context
        switch_to_kind_context();
    });
}

fn is_kind_installed() -> bool {
    Command::new("kind")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn kind_cluster_exists() -> bool {
    let output = Command::new("kind")
        .args(["get", "clusters"])
        .output()
        .expect("Failed to execute kind");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.trim() == KIND_CLUSTER_NAME)
}

fn create_kind_cluster() {
    let status = Command::new("kind")
        .args(["create", "cluster", "--name", KIND_CLUSTER_NAME])
        .status()
        .expect("Failed to create kind cluster");

    if !status.success() {
        panic!("Failed to create kind cluster '{}'", KIND_CLUSTER_NAME);
    }

    println!("Created kind cluster '{}'", KIND_CLUSTER_NAME);
}

fn switch_to_kind_context() {
    let context_name = format!("kind-{}", KIND_CLUSTER_NAME);

    let status = Command::new("kubectl")
        .args(["config", "use-context", &context_name])
        .status()
        .expect("Failed to switch kubectl context");

    if !status.success() {
        panic!("Failed to switch to context '{}'", context_name);
    }

    println!("Switched to context '{}'", context_name);
}

fn delete_kind_cluster() {
    let status = Command::new("kind")
        .args(["delete", "cluster", "--name", KIND_CLUSTER_NAME])
        .status()
        .expect("Failed to delete kind cluster");

    if status.success() {
        println!("\nDeleted kind cluster '{}'", KIND_CLUSTER_NAME);
    }
}

/// Global teardown to clean up the kind cluster after all tests.
/// This is called automatically via the #[ctor] crate.
#[cfg(test)]
mod teardown {
    use ctor::dtor;

    #[dtor]
    fn cleanup() {
        if super::kind_cluster_exists() {
            super::delete_kind_cluster();
        }
    }
}

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a fully configured MCP server with all tools registered.
async fn create_server_with_tools(read_write: bool) -> McpServer {
    let config = K8sConfig::new();
    let client = K8sClient::new(&config)
        .await
        .expect("Failed to create Kubernetes client");
    let discovery = Arc::new(RwLock::new(ApiDiscovery::new()));

    let mut registry = ToolRegistry::new();
    register_all_tools(&mut registry, client, config, discovery);

    McpServer::new(registry, read_write)
}

/// Generate a unique test namespace name.
fn test_namespace_name(test_name: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("test-{}-{}", test_name, timestamp)
        .to_lowercase()
        .replace('_', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

// ============================================================================
// MCP Protocol Tests
// ============================================================================

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

// ============================================================================
// Graceful Context Handling Tests
// ============================================================================

/// Create a server with a temporary kubeconfig that has no current context
async fn create_server_without_context() -> (McpServer, tempfile::TempDir) {
    use std::io::Write;

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let kubeconfig_path = temp_dir.path().join("kubeconfig");

    let mut file = std::fs::File::create(&kubeconfig_path).expect("Failed to create kubeconfig");
    write!(
        file,
        r#"
apiVersion: v1
kind: Config
current-context: ""
contexts:
- name: test-context
  context:
    cluster: test-cluster
clusters:
- name: test-cluster
  cluster:
    server: https://localhost:6443
users: []
"#
    )
    .expect("Failed to write kubeconfig");

    let config = K8sConfig::new().with_kubeconfig(&kubeconfig_path);
    let client = K8sClient::new(&config)
        .await
        .expect("K8sClient::new should succeed even without a valid context");

    let discovery = Arc::new(RwLock::new(ApiDiscovery::new()));
    let mut registry = ToolRegistry::new();
    register_all_tools(&mut registry, client, config, discovery);

    (McpServer::new(registry, false), temp_dir)
}

#[tokio::test]
async fn test_server_starts_without_active_context() {
    // Server should start successfully even without an active context
    let (mut server, _temp_dir) = create_server_without_context().await;
    assert!(
        server
            .handle_request(JsonRpcRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(RequestId::Number(1)),
                method: "initialize".to_string(),
                params: Some(json!({"protocol_version": "2024-11-05", "capabilities": {}})),
            })
            .await
            .is_some()
    );
}

#[tokio::test]
async fn test_tool_returns_helpful_error_without_context() {
    let (mut server, _temp_dir) = create_server_without_context().await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "pods_list",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request).await;
    assert!(response.is_some());

    let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();

    // Should have an error
    let error_message = if response_json["error"].is_object() {
        response_json["error"]["message"]
            .as_str()
            .unwrap_or("")
            .to_string()
    } else if response_json["result"]["is_error"] == true {
        response_json["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string()
    } else {
        panic!("Expected an error response");
    };

    // Error message should be helpful
    assert!(
        error_message.contains("No Kubernetes context")
            || error_message.contains("context is currently active"),
        "Error message should mention missing context: {}",
        error_message
    );
}

#[tokio::test]
async fn test_tool_returns_helpful_error_without_cluster() {
    let (mut server, _temp_dir) = create_server_without_context().await;

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

    // Should have an error
    let has_error = response_json["error"].is_object()
        || response_json["result"]["is_error"] == true
        || response_json["result"]["content"][0]["text"]
            .as_str()
            .map(|t| t.contains("No Kubernetes context") || t.contains("context"))
            .unwrap_or(false);

    assert!(has_error, "Should return an error about missing context");
}

#[tokio::test]
async fn test_context_tools_work_without_cluster() {
    // Context listing tools should work even without a cluster connection
    let (mut server, _temp_dir) = create_server_without_context().await;

    // configuration_contexts_list should work (just reads kubeconfig)
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

    // Should succeed - context list doesn't need cluster connection
    // It reads from kubeconfig which exists
    let is_success = response_json["result"]["content"].is_array();
    assert!(
        is_success,
        "configuration_contexts_list should work without cluster"
    );
}

#[tokio::test]
async fn test_initialize_works_without_context() {
    let (mut server, _temp_dir) = create_server_without_context().await;

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
    assert_eq!(response_json["result"]["server_info"]["name"], "k8s-mcp");
}

#[tokio::test]
async fn test_tools_list_works_without_context() {
    let (mut server, _temp_dir) = create_server_without_context().await;

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
    assert!(!tools.is_empty(), "Should have tools registered");
}

#[tokio::test]
async fn test_server_reconnects_when_context_becomes_available() {
    // This test verifies that the server can reconnect when a context becomes available
    // We start with no context, then switch to a valid context

    // First, ensure we have a kind cluster running
    setup_kind();

    // Save the current context
    let original_context = Command::new("kubectl")
        .args(["config", "current-context"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string());

    // Unset the current context
    let _ = Command::new("kubectl")
        .args(["config", "unset", "current-context"])
        .status();

    // Create server with no context
    let config = K8sConfig::new();
    let client = K8sClient::new(&config)
        .await
        .expect("K8sClient::new should succeed");
    let discovery = Arc::new(RwLock::new(ApiDiscovery::new()));
    let mut registry = ToolRegistry::new();
    register_all_tools(&mut registry, client, config.clone(), discovery);
    let mut server = McpServer::new(registry, false);

    // Tool call should fail
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(RequestId::Number(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "namespaces_list",
            "arguments": {}
        })),
    };

    let response = server.handle_request(request.clone()).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();
    let has_error = response_json["error"].is_object()
        || response_json["result"]["is_error"] == true
        || response_json["result"]["content"][0]["text"]
            .as_str()
            .map(|t| t.contains("No Kubernetes context"))
            .unwrap_or(false);
    assert!(has_error, "First call should fail with no context");

    // Restore the context
    if let Some(ref ctx) = original_context {
        if !ctx.is_empty() {
            let _ = Command::new("kubectl")
                .args(["config", "use-context", ctx])
                .status();
        }
    }

    // Create a new server (simulating a new tool call)
    // The client should reconnect
    let client = K8sClient::new(&config)
        .await
        .expect("K8sClient::new should succeed");
    let discovery = Arc::new(RwLock::new(ApiDiscovery::new()));
    let mut registry = ToolRegistry::new();
    register_all_tools(&mut registry, client, config, discovery);
    let mut server = McpServer::new(registry, false);

    // Tool call should now succeed
    let response = server.handle_request(request).await.unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();

    // Should have a successful result
    let is_success = response_json["result"]["content"].is_array()
        && response_json["result"]["is_error"]
            .as_bool()
            .unwrap_or(true)
            == false;
    assert!(
        is_success,
        "Second call should succeed after context is restored: {:?}",
        response_json
    );
}

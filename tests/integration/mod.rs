//! Integration tests for k8s-mcp.
//!
//! These tests require a Kubernetes cluster (kind) to be running.
//! The test harness will automatically create a kind cluster if needed.

pub mod kind;
mod test_mcp_protocol;
mod test_resource_operations;

use k8s_mcp::k8s::{ApiDiscovery, K8sClient, K8sConfig};
use k8s_mcp::mcp::McpServer;
use k8s_mcp::tools::register_all_tools;
use k8s_mcp::ToolRegistry;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Setup function to ensure kind cluster is ready.
pub fn setup_kind() {
    kind::ensure_kind_cluster();
}

/// Create a fully configured MCP server with all tools registered.
pub async fn create_server_with_tools(read_write: bool) -> McpServer {
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
pub fn test_namespace_name(test_name: &str) -> String {
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

//! Unit tests for k8s/client.rs.
//!
//! Note: Most client tests require a running Kubernetes cluster.
//! These tests focus on the parts that can be tested without a cluster.

use k8s_mcp::k8s::K8sConfig;

/// Test that K8sConfig builder produces expected configuration.
#[test]
fn test_config_builder_for_client() {
    let config = K8sConfig::new()
        .with_context("test-context")
        .with_kubeconfig("/path/to/config");

    assert_eq!(config.context, Some("test-context".to_string()));
    assert_eq!(
        config.kubeconfig_path,
        Some(std::path::PathBuf::from("/path/to/config"))
    );
}

/// Test default configuration values.
#[test]
fn test_default_config() {
    let config = K8sConfig::default();

    assert!(config.kubeconfig_path.is_none());
    assert!(config.context.is_none());
    assert!(config.cluster_url.is_none());
}

/// Test configuration cloning.
#[test]
fn test_config_clone() {
    let config = K8sConfig::new()
        .with_context("my-context")
        .with_cluster_url("https://cluster.example.com");

    let cloned = config.clone();

    assert_eq!(config.context, cloned.context);
    assert_eq!(config.cluster_url, cloned.cluster_url);
}

/// Test kubeconfig options generation.
#[test]
fn test_kubeconfig_options() {
    let config = K8sConfig::new().with_context("production");
    let options = config.kubeconfig_options();

    assert_eq!(options.context, Some("production".to_string()));
    assert_eq!(options.cluster, None);
    assert_eq!(options.user, None);
}

/// Test in-cluster detection (should be false in test environment).
#[test]
fn test_is_in_cluster() {
    // In a normal test environment, we're not in a cluster
    assert!(!k8s_mcp::k8s::config::is_in_cluster());
}

/// Test configuration with all options.
#[test]
fn test_config_with_all_options() {
    let config = K8sConfig::new()
        .with_kubeconfig("/custom/kubeconfig")
        .with_context("custom-context")
        .with_cluster_url("https://custom.cluster.local");

    assert_eq!(
        config.kubeconfig_path,
        Some(std::path::PathBuf::from("/custom/kubeconfig"))
    );
    assert_eq!(config.context, Some("custom-context".to_string()));
    assert_eq!(
        config.cluster_url,
        Some("https://custom.cluster.local".to_string())
    );
}

/// Test configuration debug output.
#[test]
fn test_config_debug() {
    let config = K8sConfig::new().with_context("debug-test");
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("debug-test"));
}

/// Test ContextInfo serialization.
#[test]
fn test_context_info_serialization() {
    use k8s_mcp::k8s::ContextInfo;
    use serde_json;

    let info = ContextInfo {
        name: "test-context".to_string(),
        cluster: Some("test-cluster".to_string()),
        user: Some("test-user".to_string()),
        namespace: Some("test-ns".to_string()),
        is_current: Some(true),
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("test-context"));
    assert!(json.contains("test-cluster"));
    assert!(json.contains("test-user"));
    assert!(json.contains("test-ns"));
    assert!(json.contains("is_current"));

    // Deserialize back
    let deserialized: ContextInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "test-context");
    assert_eq!(deserialized.cluster, Some("test-cluster".to_string()));
}

/// Test ContextInfo with minimal fields.
#[test]
fn test_context_info_minimal() {
    use k8s_mcp::k8s::ContextInfo;

    let info = ContextInfo {
        name: "minimal".to_string(),
        cluster: None,
        user: None,
        namespace: None,
        is_current: None,
    };

    let json = serde_json::to_string(&info).unwrap();
    // Optional fields with None should be skipped
    assert!(!json.contains("cluster"));
    assert!(!json.contains("user"));
    assert!(!json.contains("namespace"));
    assert!(!json.contains("is_current"));
}

//! Unit tests for k8s/config.rs.

use k8s_mcp::k8s::{ContextInfo, K8sConfig};
use std::path::PathBuf;

#[test]
fn test_k8s_config_new() {
    let config = K8sConfig::new();

    assert!(config.kubeconfig_path.is_none());
    assert!(config.context.is_none());
    assert!(config.cluster_url.is_none());
}

#[test]
fn test_k8s_config_default() {
    let config = K8sConfig::default();

    assert!(config.kubeconfig_path.is_none());
    assert!(config.context.is_none());
    assert!(config.cluster_url.is_none());
}

#[test]
fn test_k8s_config_builder_pattern() {
    let config = K8sConfig::new()
        .with_kubeconfig("/path/to/kubeconfig")
        .with_context("my-context")
        .with_cluster_url("https://cluster.example.com");

    assert_eq!(
        config.kubeconfig_path,
        Some(PathBuf::from("/path/to/kubeconfig"))
    );
    assert_eq!(config.context, Some("my-context".to_string()));
    assert_eq!(
        config.cluster_url,
        Some("https://cluster.example.com".to_string())
    );
}

#[test]
fn test_k8s_config_with_kubeconfig() {
    let config = K8sConfig::new().with_kubeconfig("/custom/path");

    assert_eq!(config.kubeconfig_path, Some(PathBuf::from("/custom/path")));
    assert!(config.context.is_none());
}

#[test]
fn test_k8s_config_with_context() {
    let config = K8sConfig::new().with_context("production");

    assert!(config.kubeconfig_path.is_none());
    assert_eq!(config.context, Some("production".to_string()));
}

#[test]
fn test_k8s_config_with_cluster_url() {
    let config = K8sConfig::new().with_cluster_url("https://k8s.example.com:6443");

    assert_eq!(
        config.cluster_url,
        Some("https://k8s.example.com:6443".to_string())
    );
}

#[test]
fn test_kubeconfig_options_empty() {
    let config = K8sConfig::new();
    let options = config.kubeconfig_options();

    assert!(options.context.is_none());
    assert!(options.cluster.is_none());
    assert!(options.user.is_none());
}

#[test]
fn test_kubeconfig_options_with_context() {
    let config = K8sConfig::new().with_context("dev-cluster");
    let options = config.kubeconfig_options();

    assert_eq!(options.context, Some("dev-cluster".to_string()));
    assert!(options.cluster.is_none());
    assert!(options.user.is_none());
}

#[test]
fn test_is_in_cluster_no_env() {
    // In test environment, KUBERNETES_SERVICE_HOST should not be set
    let result = k8s_mcp::k8s::config::is_in_cluster();
    assert!(!result);
}

#[test]
fn test_context_info_creation() {
    let info = ContextInfo {
        name: "test-context".to_string(),
        cluster: Some("test-cluster".to_string()),
        user: Some("admin".to_string()),
        namespace: Some("default".to_string()),
        is_current: Some(true),
    };

    assert_eq!(info.name, "test-context");
    assert_eq!(info.cluster, Some("test-cluster".to_string()));
    assert_eq!(info.user, Some("admin".to_string()));
    assert_eq!(info.namespace, Some("default".to_string()));
    assert_eq!(info.is_current, Some(true));
}

#[test]
fn test_context_info_serialization() {
    let info = ContextInfo {
        name: "prod".to_string(),
        cluster: Some("prod-cluster".to_string()),
        user: None,
        namespace: None,
        is_current: Some(false),
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"name\":\"prod\""));
    assert!(json.contains("\"cluster\":\"prod-cluster\""));
    // user and namespace are None, should be skipped
    assert!(!json.contains("\"user\""));
    assert!(!json.contains("\"namespace\""));
}

#[test]
fn test_context_info_deserialization() {
    let json = r#"{"name":"test","cluster":"test-cluster","is_current":true}"#;
    let info: ContextInfo = serde_json::from_str(json).unwrap();

    assert_eq!(info.name, "test");
    assert_eq!(info.cluster, Some("test-cluster".to_string()));
    assert_eq!(info.user, None);
    assert_eq!(info.namespace, None);
    assert_eq!(info.is_current, Some(true));
}

#[test]
fn test_config_clone() {
    let config = K8sConfig::new()
        .with_context("original")
        .with_kubeconfig("/path");

    let cloned = config.clone();

    assert_eq!(config.context, cloned.context);
    assert_eq!(config.kubeconfig_path, cloned.kubeconfig_path);
}

#[test]
fn test_config_debug() {
    let config = K8sConfig::new().with_context("debug-context");
    let debug_output = format!("{:?}", config);

    assert!(debug_output.contains("debug-context"));
}

#[test]
fn test_context_info_debug() {
    let info = ContextInfo {
        name: "debug-test".to_string(),
        cluster: None,
        user: None,
        namespace: None,
        is_current: None,
    };

    let debug_output = format!("{:?}", info);
    assert!(debug_output.contains("debug-test"));
}

#[test]
fn test_config_chained_builders() {
    // Test that builder methods can be chained multiple times
    // (though only the last value should stick)
    let config = K8sConfig::new()
        .with_context("first")
        .with_context("second")
        .with_context("final");

    assert_eq!(config.context, Some("final".to_string()));
}

//! Unit tests for k8s/resources.rs.

use k8s_mcp::k8s::{PatchStrategy, parse_api_version, parse_gvk};
use serde_json::json;

#[test]
fn test_parse_gvk_basic() {
    let gvk = parse_gvk("apps", "v1", "Deployment");

    assert_eq!(gvk.group, "apps");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Deployment");
}

#[test]
fn test_parse_gvk_core_group() {
    let gvk = parse_gvk("", "v1", "Pod");

    assert_eq!(gvk.group, "");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Pod");
}

#[test]
fn test_parse_gvk_long_group() {
    let gvk = parse_gvk("networking.k8s.io", "v1", "Ingress");

    assert_eq!(gvk.group, "networking.k8s.io");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Ingress");
}

#[test]
fn test_parse_gvk_batch() {
    let gvk = parse_gvk("batch", "v1", "Job");

    assert_eq!(gvk.group, "batch");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Job");
}

#[test]
fn test_parse_api_version_with_group() {
    let gvk = parse_api_version("apps/v1", "Deployment");

    assert_eq!(gvk.group, "apps");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Deployment");
}

#[test]
fn test_parse_api_version_without_group() {
    let gvk = parse_api_version("v1", "Pod");

    assert_eq!(gvk.group, "");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Pod");
}

#[test]
fn test_parse_api_version_networking() {
    let gvk = parse_api_version("networking.k8s.io/v1", "Ingress");

    assert_eq!(gvk.group, "networking.k8s.io");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "Ingress");
}

#[test]
fn test_parse_api_version_batch() {
    let gvk = parse_api_version("batch/v1", "CronJob");

    assert_eq!(gvk.group, "batch");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "CronJob");
}

#[test]
fn test_parse_api_version_storage() {
    let gvk = parse_api_version("storage.k8s.io/v1", "StorageClass");

    assert_eq!(gvk.group, "storage.k8s.io");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "StorageClass");
}

#[test]
fn test_parse_api_version_apiextensions() {
    let gvk = parse_api_version("apiextensions.k8s.io/v1", "CustomResourceDefinition");

    assert_eq!(gvk.group, "apiextensions.k8s.io");
    assert_eq!(gvk.version, "v1");
    assert_eq!(gvk.kind, "CustomResourceDefinition");
}

#[test]
fn test_patch_strategy_default() {
    let strategy = PatchStrategy::default();
    assert!(matches!(strategy, PatchStrategy::Merge));
}

#[test]
fn test_patch_strategy_merge() {
    let strategy = PatchStrategy::Merge;
    assert!(matches!(strategy, PatchStrategy::Merge));
}

#[test]
fn test_patch_strategy_strategic() {
    let strategy = PatchStrategy::Strategic;
    assert!(matches!(strategy, PatchStrategy::Strategic));
}

#[test]
fn test_patch_strategy_json() {
    let strategy = PatchStrategy::Json;
    assert!(matches!(strategy, PatchStrategy::Json));
}

#[test]
fn test_gvk_equality() {
    let gvk1 = parse_gvk("apps", "v1", "Deployment");
    let gvk2 = parse_gvk("apps", "v1", "Deployment");
    let gvk3 = parse_gvk("apps", "v1", "StatefulSet");

    assert_eq!(gvk1, gvk2);
    assert_ne!(gvk1, gvk3);
}

#[test]
fn test_gvk_debug() {
    let gvk = parse_gvk("batch", "v1", "Job");
    let debug_str = format!("{:?}", gvk);

    assert!(debug_str.contains("batch"));
    assert!(debug_str.contains("v1"));
    assert!(debug_str.contains("Job"));
}

#[test]
fn test_gvk_clone() {
    let gvk = parse_gvk("apps", "v1", "Deployment");
    let cloned = gvk.clone();

    assert_eq!(gvk.group, cloned.group);
    assert_eq!(gvk.version, cloned.version);
    assert_eq!(gvk.kind, cloned.kind);
}

#[test]
fn test_dynamic_object_from_json() {
    use k8s_mcp::k8s::resources::dynamic_object_from_json;

    let json = json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": {
            "name": "test-pod",
            "namespace": "default"
        }
    });

    let result = dynamic_object_from_json(json);
    assert!(result.is_ok());

    let obj = result.unwrap();
    assert_eq!(obj.metadata.name, Some("test-pod".to_string()));
    assert_eq!(obj.metadata.namespace, Some("default".to_string()));
}

#[test]
fn test_dynamic_object_from_json_invalid() {
    use k8s_mcp::k8s::resources::dynamic_object_from_json;

    // Invalid JSON structure (metadata should be an object)
    let json = json!({
        "apiVersion": "v1",
        "kind": "Pod",
        "metadata": "not-an-object"
    });

    let result = dynamic_object_from_json(json);
    // Should fail since metadata must be an object
    assert!(result.is_err());
}

#[test]
fn test_dynamic_object_from_json_minimal() {
    use k8s_mcp::k8s::resources::dynamic_object_from_json;

    let json = json!({
        "metadata": {}
    });

    let result = dynamic_object_from_json(json);
    assert!(result.is_ok());

    let obj = result.unwrap();
    assert!(obj.metadata.name.is_none());
}

#[test]
fn test_parse_api_version_edge_cases() {
    // Multiple slashes - only first one is used
    let gvk = parse_api_version("group/version/extra", "Kind");
    assert_eq!(gvk.group, "group");
    assert_eq!(gvk.version, "version/extra");

    // No version after slash
    let gvk = parse_api_version("group/", "Kind");
    assert_eq!(gvk.group, "group");
    assert_eq!(gvk.version, "");
}

#[test]
fn test_parse_gvk_with_special_chars() {
    // Test with CRD-style names
    let gvk = parse_gvk("mycompany.com", "v1alpha1", "MyCustomResource");
    assert_eq!(gvk.group, "mycompany.com");
    assert_eq!(gvk.version, "v1alpha1");
    assert_eq!(gvk.kind, "MyCustomResource");
}

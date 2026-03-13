//! Unit tests for k8s/discovery.rs.

use k8s_mcp::k8s::{ApiDiscovery, ApiResourceInfo};

#[test]
fn test_api_discovery_new() {
    let discovery = ApiDiscovery::new();

    assert!(!discovery.is_discovered());
    assert!(discovery.is_empty());
    assert_eq!(discovery.len(), 0);
}

#[test]
fn test_api_discovery_default() {
    let discovery = ApiDiscovery::default();

    assert!(!discovery.is_discovered());
    assert!(discovery.is_empty());
}

#[test]
fn test_api_discovery_list_empty() {
    let discovery = ApiDiscovery::new();
    let list = discovery.list();

    assert!(list.is_empty());
}

#[test]
fn test_api_discovery_get_empty() {
    let discovery = ApiDiscovery::new();

    assert!(discovery.get("", "v1", "Pod").is_none());
    assert!(discovery.get("apps", "v1", "Deployment").is_none());
}

#[test]
fn test_api_discovery_resolve_empty() {
    let discovery = ApiDiscovery::new();

    assert!(discovery.resolve("pod").is_none());
    assert!(discovery.resolve("Pod").is_none());
    assert!(discovery.resolve("pods").is_none());
}

#[test]
fn test_api_discovery_list_by_category_empty() {
    let discovery = ApiDiscovery::new();
    let list = discovery.list_by_category("all");

    assert!(list.is_empty());
}

#[test]
fn test_common_resources() {
    use k8s_mcp::k8s::discovery::shortcuts::common_resources;

    let resources = common_resources();

    assert!(!resources.is_empty());

    // Check that we have expected resource kinds
    let kinds: Vec<&str> = resources.iter().map(|r| r.kind.as_str()).collect();
    assert!(kinds.contains(&"Pod"));
    assert!(kinds.contains(&"Deployment"));
    assert!(kinds.contains(&"Service"));
    assert!(kinds.contains(&"ConfigMap"));
    assert!(kinds.contains(&"Secret"));
    assert!(kinds.contains(&"Namespace"));
    assert!(kinds.contains(&"Node"));
    assert!(kinds.contains(&"StatefulSet"));
    assert!(kinds.contains(&"DaemonSet"));
    assert!(kinds.contains(&"Job"));
    assert!(kinds.contains(&"CronJob"));
    assert!(kinds.contains(&"Ingress"));
}

#[test]
fn test_common_resources_namespaced() {
    use k8s_mcp::k8s::discovery::shortcuts::common_resources;

    let resources = common_resources();

    // Check namespaced resources
    let pod = resources.iter().find(|r| r.kind == "Pod").unwrap();
    assert!(pod.namespaced);

    let deployment = resources.iter().find(|r| r.kind == "Deployment").unwrap();
    assert!(deployment.namespaced);

    // Check cluster-scoped resources
    let node = resources.iter().find(|r| r.kind == "Node").unwrap();
    assert!(!node.namespaced);

    let namespace = resources.iter().find(|r| r.kind == "Namespace").unwrap();
    assert!(!namespace.namespaced);
}

#[test]
fn test_common_resources_short_names() {
    use k8s_mcp::k8s::discovery::shortcuts::common_resources;

    let resources = common_resources();

    let pod = resources.iter().find(|r| r.kind == "Pod").unwrap();
    assert!(pod.short_names.contains(&"po".to_string()));

    let deployment = resources.iter().find(|r| r.kind == "Deployment").unwrap();
    assert!(deployment.short_names.contains(&"deploy".to_string()));

    let service = resources.iter().find(|r| r.kind == "Service").unwrap();
    assert!(service.short_names.contains(&"svc".to_string()));

    let namespace = resources.iter().find(|r| r.kind == "Namespace").unwrap();
    assert!(namespace.short_names.contains(&"ns".to_string()));
}

#[test]
fn test_common_resources_api_versions() {
    use k8s_mcp::k8s::discovery::shortcuts::common_resources;

    let resources = common_resources();

    let pod = resources.iter().find(|r| r.kind == "Pod").unwrap();
    assert_eq!(pod.api_version, "v1");
    assert_eq!(pod.group, "");

    let deployment = resources.iter().find(|r| r.kind == "Deployment").unwrap();
    assert_eq!(deployment.api_version, "v1");
    assert_eq!(deployment.group, "apps");

    let ingress = resources.iter().find(|r| r.kind == "Ingress").unwrap();
    assert_eq!(ingress.api_version, "v1");
    assert_eq!(ingress.group, "networking.k8s.io");
}

#[test]
fn test_common_resources_verbs() {
    use k8s_mcp::k8s::discovery::shortcuts::common_resources;

    let resources = common_resources();

    let pod = resources.iter().find(|r| r.kind == "Pod").unwrap();
    assert!(pod.verbs.contains(&"get".to_string()));
    assert!(pod.verbs.contains(&"list".to_string()));
    assert!(pod.verbs.contains(&"create".to_string()));
    assert!(pod.verbs.contains(&"delete".to_string()));

    let node = resources.iter().find(|r| r.kind == "Node").unwrap();
    assert!(node.verbs.contains(&"get".to_string()));
    assert!(node.verbs.contains(&"list".to_string()));
    // Nodes cannot be created directly via API
    assert!(!node.verbs.contains(&"create".to_string()));
}

#[test]
fn test_common_resources_categories() {
    use k8s_mcp::k8s::discovery::shortcuts::common_resources;

    let resources = common_resources();

    let pod = resources.iter().find(|r| r.kind == "Pod").unwrap();
    assert!(pod.categories.contains(&"all".to_string()));

    let deployment = resources.iter().find(|r| r.kind == "Deployment").unwrap();
    assert!(deployment.categories.contains(&"all".to_string()));

    // Secrets are not in the "all" category
    let secret = resources.iter().find(|r| r.kind == "Secret").unwrap();
    assert!(!secret.categories.contains(&"all".to_string()));
}

#[test]
fn test_api_resource_info_serialization() {
    let info = ApiResourceInfo {
        name: "pods".to_string(),
        singular: "pod".to_string(),
        kind: "Pod".to_string(),
        api_version: "v1".to_string(),
        group: "".to_string(),
        namespaced: true,
        short_names: vec!["po".to_string()],
        verbs: vec!["get".to_string(), "list".to_string()],
        categories: vec!["all".to_string()],
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"name\":\"pods\""));
    assert!(json.contains("\"kind\":\"Pod\""));
    assert!(json.contains("\"namespaced\":true"));

    let deserialized: ApiResourceInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "pods");
    assert_eq!(deserialized.kind, "Pod");
    assert!(deserialized.namespaced);
}

#[test]
fn test_api_resource_info_debug() {
    let info = ApiResourceInfo {
        name: "deployments".to_string(),
        singular: "deployment".to_string(),
        kind: "Deployment".to_string(),
        api_version: "v1".to_string(),
        group: "apps".to_string(),
        namespaced: true,
        short_names: vec!["deploy".to_string()],
        verbs: vec![],
        categories: vec![],
    };

    let debug_output = format!("{:?}", info);
    assert!(debug_output.contains("deployments"));
    assert!(debug_output.contains("Deployment"));
    assert!(debug_output.contains("apps"));
}

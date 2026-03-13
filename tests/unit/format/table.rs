//! Unit tests for format/table.rs.

use k8s_mcp::format::*;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec, DeploymentStatus};
use k8s_openapi::api::core::v1::{
    ConfigMap, Namespace, Node, NodeCondition, NodeStatus, Pod, PodSpec, PodStatus, Secret,
    Service, ServicePort, ServiceSpec,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

// ============================================================================
// Pod Table Tests
// ============================================================================

fn create_test_pod(name: &str, namespace: &str, phase: &str) -> Pod {
    Pod {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(PodSpec {
            containers: vec![k8s_openapi::api::core::v1::Container {
                name: "main".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        }),
        status: Some(PodStatus {
            phase: Some(phase.to_string()),
            ..Default::default()
        }),
    }
}

#[test]
fn test_format_pods_table_empty() {
    let pods: Vec<Pod> = vec![];
    let output = format_pods_table(&pods);

    assert!(output.contains("NAME"));
    assert!(output.contains("READY"));
    assert!(output.contains("STATUS"));
}

#[test]
fn test_format_pods_table_single() {
    let pods = vec![create_test_pod("test-pod", "default", "Running")];
    let output = format_pods_table(&pods);

    assert!(output.contains("test-pod"));
    assert!(output.contains("Running"));
}

#[test]
fn test_format_pods_table_multiple() {
    let pods = vec![
        create_test_pod("pod-1", "ns-1", "Running"),
        create_test_pod("pod-2", "ns-2", "Pending"),
    ];
    let output = format_pods_table(&pods);

    assert!(output.contains("pod-1"));
    assert!(output.contains("pod-2"));
}

// ============================================================================
// Deployment Table Tests
// ============================================================================

fn create_test_deployment(name: &str, namespace: &str) -> Deployment {
    Deployment {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(DeploymentSpec {
            replicas: Some(1),
            selector: Default::default(),
            template: Default::default(),
            ..Default::default()
        }),
        status: Some(DeploymentStatus {
            replicas: Some(1),
            ready_replicas: Some(1),
            updated_replicas: Some(1),
            available_replicas: Some(1),
            ..Default::default()
        }),
    }
}

#[test]
fn test_format_deployments_table_empty() {
    let deployments: Vec<Deployment> = vec![];
    let output = format_deployments_table(&deployments);

    assert!(output.contains("NAME"));
    assert!(output.contains("READY"));
}

#[test]
fn test_format_deployments_table_single() {
    let deployments = vec![create_test_deployment("my-app", "production")];
    let output = format_deployments_table(&deployments);

    assert!(output.contains("my-app"));
    assert!(output.contains("production"));
}

// ============================================================================
// Service Table Tests
// ============================================================================

fn create_test_service(name: &str, namespace: &str) -> Service {
    Service {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            type_: Some("ClusterIP".to_string()),
            cluster_ip: Some("10.0.0.1".to_string()),
            ports: Some(vec![ServicePort {
                port: 80,
                protocol: Some("TCP".to_string()),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[test]
fn test_format_services_table_empty() {
    let services: Vec<Service> = vec![];
    let output = format_services_table(&services);

    assert!(output.contains("NAME"));
    assert!(output.contains("TYPE"));
}

#[test]
fn test_format_services_table_single() {
    let services = vec![create_test_service("my-service", "default")];
    let output = format_services_table(&services);

    assert!(output.contains("my-service"));
}

// ============================================================================
// ConfigMap Table Tests
// ============================================================================

fn create_test_configmap(name: &str, namespace: &str) -> ConfigMap {
    ConfigMap {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn test_format_configmaps_table_empty() {
    let configmaps: Vec<ConfigMap> = vec![];
    let output = format_configmaps_table(&configmaps);

    assert!(output.contains("NAME"));
    assert!(output.contains("DATA"));
}

#[test]
fn test_format_configmaps_table_single() {
    let configmaps = vec![create_test_configmap("my-config", "default")];
    let output = format_configmaps_table(&configmaps);

    assert!(output.contains("my-config"));
}

// ============================================================================
// Secret Table Tests
// ============================================================================

fn create_test_secret(name: &str, namespace: &str) -> Secret {
    Secret {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        type_: Some("Opaque".to_string()),
        ..Default::default()
    }
}

#[test]
fn test_format_secrets_table_empty() {
    let secrets: Vec<Secret> = vec![];
    let output = format_secrets_table(&secrets);

    assert!(output.contains("NAME"));
    assert!(output.contains("TYPE"));
}

#[test]
fn test_format_secrets_table_single() {
    let secrets = vec![create_test_secret("my-secret", "default")];
    let output = format_secrets_table(&secrets);

    assert!(output.contains("my-secret"));
}

// ============================================================================
// Namespace Table Tests
// ============================================================================

fn create_test_namespace(name: &str) -> Namespace {
    Namespace {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            ..Default::default()
        },
        spec: None,
        status: Some(k8s_openapi::api::core::v1::NamespaceStatus {
            phase: Some("Active".to_string()),
            ..Default::default()
        }),
    }
}

#[test]
fn test_format_namespaces_table_empty() {
    let namespaces: Vec<Namespace> = vec![];
    let output = format_namespaces_table(&namespaces);

    assert!(output.contains("NAME"));
    assert!(output.contains("STATUS"));
}

#[test]
fn test_format_namespaces_table_single() {
    let namespaces = vec![create_test_namespace("my-namespace")];
    let output = format_namespaces_table(&namespaces);

    assert!(output.contains("my-namespace"));
}

// ============================================================================
// Node Table Tests
// ============================================================================

fn create_test_node(name: &str, ready: bool) -> Node {
    Node {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            ..Default::default()
        },
        spec: None,
        status: Some(NodeStatus {
            conditions: Some(vec![NodeCondition {
                type_: "Ready".to_string(),
                status: if ready { "True" } else { "False" }.to_string(),
                ..Default::default()
            }]),
            node_info: Some(k8s_openapi::api::core::v1::NodeSystemInfo {
                kubelet_version: "v1.32.0".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
    }
}

#[test]
fn test_format_nodes_table_empty() {
    let nodes: Vec<Node> = vec![];
    let output = format_nodes_table(&nodes);

    assert!(output.contains("NAME"));
    assert!(output.contains("STATUS"));
}

#[test]
fn test_format_nodes_table_single() {
    let nodes = vec![create_test_node("node-1", true)];
    let output = format_nodes_table(&nodes);

    assert!(output.contains("node-1"));
    assert!(output.contains("Ready"));
}

#[test]
fn test_format_nodes_table_not_ready() {
    let nodes = vec![create_test_node("node-2", false)];
    let output = format_nodes_table(&nodes);

    assert!(output.contains("NotReady"));
}

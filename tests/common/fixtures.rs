//! Test fixtures and data builders for Kubernetes resources.
//!
//! This module provides simplified builders for creating test data.

#![allow(dead_code)]

use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec, DeploymentStatus};
use k8s_openapi::api::core::v1::{
    ConfigMap, Namespace, Node, NodeCondition, NodeStatus, Pod, PodSpec, PodStatus, Secret,
    Service, ServicePort, ServiceSpec,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use std::collections::BTreeMap;

/// Builder for creating test Pod objects.
pub struct PodBuilder {
    name: String,
    namespace: String,
    phase: String,
}

impl PodBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        PodBuilder {
            name: name.into(),
            namespace: "default".to_string(),
            phase: "Running".to_string(),
        }
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn phase(mut self, phase: impl Into<String>) -> Self {
        self.phase = phase.into();
        self
    }

    pub fn build(self) -> Pod {
        Pod {
            metadata: ObjectMeta {
                name: Some(self.name),
                namespace: Some(self.namespace),
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
                phase: Some(self.phase),
                ..Default::default()
            }),
        }
    }
}

/// Builder for creating test Deployment objects.
pub struct DeploymentBuilder {
    name: String,
    namespace: String,
    replicas: i32,
    ready_replicas: i32,
}

impl DeploymentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        DeploymentBuilder {
            name: name.into(),
            namespace: "default".to_string(),
            replicas: 1,
            ready_replicas: 1,
        }
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn replicas(mut self, replicas: i32) -> Self {
        self.replicas = replicas;
        self
    }

    pub fn ready_replicas(mut self, ready: i32) -> Self {
        self.ready_replicas = ready;
        self
    }

    pub fn build(self) -> Deployment {
        Deployment {
            metadata: ObjectMeta {
                name: Some(self.name),
                namespace: Some(self.namespace),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(self.replicas),
                selector: Default::default(),
                template: Default::default(),
                ..Default::default()
            }),
            status: Some(DeploymentStatus {
                replicas: Some(self.replicas),
                ready_replicas: Some(self.ready_replicas),
                updated_replicas: Some(self.ready_replicas),
                available_replicas: Some(self.ready_replicas),
                ..Default::default()
            }),
        }
    }
}

/// Builder for creating test Service objects.
pub struct ServiceBuilder {
    name: String,
    namespace: String,
    service_type: String,
    cluster_ip: String,
}

impl ServiceBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        ServiceBuilder {
            name: name.into(),
            namespace: "default".to_string(),
            service_type: "ClusterIP".to_string(),
            cluster_ip: "10.0.0.1".to_string(),
        }
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn build(self) -> Service {
        Service {
            metadata: ObjectMeta {
                name: Some(self.name),
                namespace: Some(self.namespace),
                ..Default::default()
            },
            spec: Some(ServiceSpec {
                type_: Some(self.service_type),
                cluster_ip: Some(self.cluster_ip),
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
}

/// Builder for creating test Node objects.
pub struct NodeBuilder {
    name: String,
    ready: bool,
    kubelet_version: String,
}

impl NodeBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        NodeBuilder {
            name: name.into(),
            ready: true,
            kubelet_version: "v1.32.0".to_string(),
        }
    }

    pub fn ready(mut self, ready: bool) -> Self {
        self.ready = ready;
        self
    }

    pub fn build(self) -> Node {
        Node {
            metadata: ObjectMeta {
                name: Some(self.name),
                ..Default::default()
            },
            spec: None,
            status: Some(NodeStatus {
                conditions: Some(vec![NodeCondition {
                    type_: "Ready".to_string(),
                    status: if self.ready { "True" } else { "False" }.to_string(),
                    ..Default::default()
                }]),
                node_info: Some(k8s_openapi::api::core::v1::NodeSystemInfo {
                    kubelet_version: self.kubelet_version,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        }
    }
}

/// Builder for creating test Namespace objects.
pub struct NamespaceBuilder {
    name: String,
    phase: String,
}

impl NamespaceBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        NamespaceBuilder {
            name: name.into(),
            phase: "Active".to_string(),
        }
    }

    pub fn build(self) -> Namespace {
        Namespace {
            metadata: ObjectMeta {
                name: Some(self.name),
                ..Default::default()
            },
            spec: None,
            status: Some(k8s_openapi::api::core::v1::NamespaceStatus {
                phase: Some(self.phase),
                ..Default::default()
            }),
        }
    }
}

/// Builder for creating test ConfigMap objects.
pub struct ConfigMapBuilder {
    name: String,
    namespace: String,
    data: BTreeMap<String, String>,
}

impl ConfigMapBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        ConfigMapBuilder {
            name: name.into(),
            namespace: "default".to_string(),
            data: BTreeMap::new(),
        }
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    pub fn build(self) -> ConfigMap {
        ConfigMap {
            metadata: ObjectMeta {
                name: Some(self.name),
                namespace: Some(self.namespace),
                ..Default::default()
            },
            data: if self.data.is_empty() {
                None
            } else {
                Some(self.data)
            },
            ..Default::default()
        }
    }
}

/// Builder for creating test Secret objects.
pub struct SecretBuilder {
    name: String,
    namespace: String,
    secret_type: String,
}

impl SecretBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        SecretBuilder {
            name: name.into(),
            namespace: "default".to_string(),
            secret_type: "Opaque".to_string(),
        }
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn build(self) -> Secret {
        Secret {
            metadata: ObjectMeta {
                name: Some(self.name),
                namespace: Some(self.namespace),
                ..Default::default()
            },
            type_: Some(self.secret_type),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_builder() {
        let pod = PodBuilder::new("test-pod")
            .namespace("my-namespace")
            .phase("Running")
            .build();

        assert_eq!(pod.metadata.name, Some("test-pod".to_string()));
        assert_eq!(pod.metadata.namespace, Some("my-namespace".to_string()));
        assert_eq!(
            pod.status.as_ref().unwrap().phase,
            Some("Running".to_string())
        );
    }

    #[test]
    fn test_deployment_builder() {
        let deploy = DeploymentBuilder::new("test-deploy")
            .namespace("production")
            .replicas(3)
            .ready_replicas(2)
            .build();

        assert_eq!(deploy.metadata.name, Some("test-deploy".to_string()));
        assert_eq!(deploy.metadata.namespace, Some("production".to_string()));
        assert_eq!(deploy.status.as_ref().unwrap().replicas, Some(3));
        assert_eq!(deploy.status.as_ref().unwrap().ready_replicas, Some(2));
    }

    #[test]
    fn test_node_builder() {
        let node = NodeBuilder::new("node-1").ready(true).build();
        assert_eq!(node.metadata.name, Some("node-1".to_string()));
    }

    #[test]
    fn test_service_builder() {
        let svc = ServiceBuilder::new("my-service").build();
        assert_eq!(svc.metadata.name, Some("my-service".to_string()));
    }
}

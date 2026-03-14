//! Get resource tool implementation.

use crate::error::Result;
use crate::k8s::{AdaptiveResource, K8sClient, parse_api_version};
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{ToolHandler, get_optional_string_arg, get_string_arg, text_result};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::k8s::ApiDiscovery;

/// Get a specific resource by name.
pub struct GetResourceTool {
    client: Arc<K8sClient>,
    discovery: Arc<RwLock<ApiDiscovery>>,
}

impl GetResourceTool {
    pub fn new(client: Arc<K8sClient>, discovery: Arc<RwLock<ApiDiscovery>>) -> Self {
        GetResourceTool { client, discovery }
    }
}

#[async_trait]
impl ToolHandler for GetResourceTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let api_version = get_string_arg(&args, "apiVersion")?;
        let kind = get_string_arg(&args, "kind")?;
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");

        let result = self
            .get_resource(&api_version, &kind, &name, namespace.as_deref())
            .await?;

        let output = serde_json::to_string_pretty(&result)?;
        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "resources_get",
            "Get a Kubernetes resource by name. Returns the full resource definition.",
            ToolInputSchema::object()
                .with_properties(HashMap::from([
                    (
                        "apiVersion".to_string(),
                        PropertySchema::string()
                            .description("API version (e.g., 'v1' for core, 'apps/v1' for apps)")
                            .enum_values(vec![
                                "v1".to_string(),
                                "apps/v1".to_string(),
                                "batch/v1".to_string(),
                                "networking.k8s.io/v1".to_string(),
                            ]),
                    ),
                    (
                        "kind".to_string(),
                        PropertySchema::string()
                            .description("Resource kind (e.g., 'Pod', 'Deployment', 'Service')")
                            .enum_values(vec![
                                "Pod".to_string(),
                                "Deployment".to_string(),
                                "Service".to_string(),
                                "ConfigMap".to_string(),
                                "Secret".to_string(),
                                "Namespace".to_string(),
                                "Node".to_string(),
                                "Ingress".to_string(),
                                "StatefulSet".to_string(),
                                "DaemonSet".to_string(),
                                "Job".to_string(),
                                "CronJob".to_string(),
                                "ReplicaSet".to_string(),
                                "PersistentVolumeClaim".to_string(),
                                "PersistentVolume".to_string(),
                            ]),
                    ),
                    (
                        "name".to_string(),
                        PropertySchema::string().description("Resource name"),
                    ),
                    (
                        "namespace".to_string(),
                        PropertySchema::string()
                            .description("Namespace (optional, uses default if not specified)"),
                    ),
                ]))
                .with_required(vec![
                    "apiVersion".to_string(),
                    "kind".to_string(),
                    "name".to_string(),
                ]),
        )
    }
}

impl GetResourceTool {
    async fn get_resource(
        &self,
        api_version: &str,
        kind: &str,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<serde_json::Value> {
        // Use typed APIs for known types (for better performance and special handling)
        match (api_version, kind) {
            // Core v1 resources with typed handling
            ("v1", "Pod") => {
                let api = self.client.pods_api(namespace).await?;
                let pod = api.get(name).await?;
                Ok(serde_json::to_value(pod)?)
            }
            ("v1", "Service") => {
                let api = self.client.services_api(namespace).await?;
                let svc = api.get(name).await?;
                Ok(serde_json::to_value(svc)?)
            }
            ("v1", "ConfigMap") => {
                let api = self.client.configmaps_api(namespace).await?;
                let cm = api.get(name).await?;
                Ok(serde_json::to_value(cm)?)
            }
            ("v1", "Secret") => {
                let api = self.client.secrets_api(namespace).await?;
                let secret = api.get(name).await?;
                // Redact secret data for security
                let mut value = serde_json::to_value(&secret)?;
                if let Some(data) = value.get_mut("data").and_then(|d| d.as_object_mut()) {
                    for (_key, val) in data.iter_mut() {
                        *val = json!("REDACTED");
                    }
                }
                if let Some(string_data) =
                    value.get_mut("stringData").and_then(|d| d.as_object_mut())
                {
                    for (_key, val) in string_data.iter_mut() {
                        *val = json!("REDACTED");
                    }
                }
                Ok(value)
            }
            ("v1", "Namespace") => {
                let api = self.client.namespaces_api().await?;
                let ns = api.get(name).await?;
                Ok(serde_json::to_value(ns)?)
            }
            ("v1", "Node") => {
                let api = self.client.nodes_api().await?;
                let node = api.get(name).await?;
                Ok(serde_json::to_value(node)?)
            }
            ("v1", "PersistentVolumeClaim") => {
                let api = self.client.pvcs_api(namespace).await?;
                let pvc = api.get(name).await?;
                Ok(serde_json::to_value(pvc)?)
            }
            ("v1", "PersistentVolume") => {
                let api = self.client.pvs_api().await?;
                let pv = api.get(name).await?;
                Ok(serde_json::to_value(pv)?)
            }
            // Apps v1 resources
            ("apps/v1", "Deployment") => {
                let api = self.client.deployments_api(namespace).await?;
                let deploy = api.get(name).await?;
                Ok(serde_json::to_value(deploy)?)
            }
            ("apps/v1", "StatefulSet") => {
                let api = self.client.statefulsets_api(namespace).await?;
                let sts = api.get(name).await?;
                Ok(serde_json::to_value(sts)?)
            }
            ("apps/v1", "DaemonSet") => {
                let api = self.client.daemonsets_api(namespace).await?;
                let ds = api.get(name).await?;
                Ok(serde_json::to_value(ds)?)
            }
            ("apps/v1", "ReplicaSet") => {
                let api = self.client.replicasets_api(namespace).await?;
                let rs = api.get(name).await?;
                Ok(serde_json::to_value(rs)?)
            }
            // Batch v1 resources
            ("batch/v1", "Job") => {
                let api = self.client.jobs_api(namespace).await?;
                let job = api.get(name).await?;
                Ok(serde_json::to_value(job)?)
            }
            ("batch/v1", "CronJob") => {
                let api = self.client.cronjobs_api(namespace).await?;
                let cj = api.get(name).await?;
                Ok(serde_json::to_value(cj)?)
            }
            // Networking v1 resources
            ("networking.k8s.io/v1", "Ingress") => {
                let api = self.client.ingresses_api(namespace).await?;
                let ing = api.get(name).await?;
                Ok(serde_json::to_value(ing)?)
            }
            // Use adaptive API for all other resource types
            _ => {
                let gvk = parse_api_version(api_version, kind);
                let adaptive = AdaptiveResource::new(self.client.clone(), self.discovery.clone());
                let resource = adaptive.get(&gvk, name, namespace).await?;
                Ok(serde_json::to_value(resource)?)
            }
        }
    }
}

/// Get pod tool (convenience wrapper).
pub struct PodsGetTool {
    client: Arc<K8sClient>,
}

impl PodsGetTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        PodsGetTool { client }
    }
}

#[async_trait]
impl ToolHandler for PodsGetTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");

        let api = self.client.pods_api(namespace.as_deref()).await?;
        let pod = api.get(&name).await?;

        let output = serde_json::to_string_pretty(&pod)?;
        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "pods_get",
            "Get a Kubernetes Pod by name",
            ToolInputSchema::object()
                .with_properties(HashMap::from([
                    (
                        "name".to_string(),
                        PropertySchema::string().description("Pod name"),
                    ),
                    (
                        "namespace".to_string(),
                        PropertySchema::string().description("Namespace (optional)"),
                    ),
                ]))
                .with_required(vec!["name".to_string()]),
        )
    }
}

/// Get deployment tool.
pub struct DeploymentsGetTool {
    client: Arc<K8sClient>,
}

impl DeploymentsGetTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        DeploymentsGetTool { client }
    }
}

#[async_trait]
impl ToolHandler for DeploymentsGetTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");

        let api = self.client.deployments_api(namespace.as_deref()).await?;
        let deploy = api.get(&name).await?;

        let output = serde_json::to_string_pretty(&deploy)?;
        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "deployments_get",
            "Get a Kubernetes Deployment by name",
            ToolInputSchema::object()
                .with_properties(HashMap::from([
                    (
                        "name".to_string(),
                        PropertySchema::string().description("Deployment name"),
                    ),
                    (
                        "namespace".to_string(),
                        PropertySchema::string().description("Namespace (optional)"),
                    ),
                ]))
                .with_required(vec!["name".to_string()]),
        )
    }
}

/// Get service tool.
pub struct ServicesGetTool {
    client: Arc<K8sClient>,
}

impl ServicesGetTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        ServicesGetTool { client }
    }
}

#[async_trait]
impl ToolHandler for ServicesGetTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");

        let api = self.client.services_api(namespace.as_deref()).await?;
        let svc = api.get(&name).await?;

        let output = serde_json::to_string_pretty(&svc)?;
        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "services_get",
            "Get a Kubernetes Service by name",
            ToolInputSchema::object()
                .with_properties(HashMap::from([
                    (
                        "name".to_string(),
                        PropertySchema::string().description("Service name"),
                    ),
                    (
                        "namespace".to_string(),
                        PropertySchema::string().description("Namespace (optional)"),
                    ),
                ]))
                .with_required(vec!["name".to_string()]),
        )
    }
}

/// Get node tool.
pub struct NodesGetTool {
    client: Arc<K8sClient>,
}

impl NodesGetTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        NodesGetTool { client }
    }
}

#[async_trait]
impl ToolHandler for NodesGetTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;

        let api = self.client.nodes_api().await?;
        let node = api.get(&name).await?;

        let output = serde_json::to_string_pretty(&node)?;
        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "nodes_get",
            "Get a Kubernetes Node by name",
            ToolInputSchema::object()
                .with_properties(HashMap::from([(
                    "name".to_string(),
                    PropertySchema::string().description("Node name"),
                )]))
                .with_required(vec!["name".to_string()]),
        )
    }
}

/// Get namespace tool.
pub struct NamespacesGetTool {
    client: Arc<K8sClient>,
}

impl NamespacesGetTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        NamespacesGetTool { client }
    }
}

#[async_trait]
impl ToolHandler for NamespacesGetTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;

        let api = self.client.namespaces_api().await?;
        let ns = api.get(&name).await?;

        let output = serde_json::to_string_pretty(&ns)?;
        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "namespaces_get",
            "Get a Kubernetes Namespace by name",
            ToolInputSchema::object()
                .with_properties(HashMap::from([(
                    "name".to_string(),
                    PropertySchema::string().description("Namespace name"),
                )]))
                .with_required(vec!["name".to_string()]),
        )
    }
}

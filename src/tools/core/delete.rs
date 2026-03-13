//! Delete resources tool implementation.

use crate::error::{Error, Result};
use crate::k8s::K8sClient;
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{
    ToolHandler, get_optional_integer_arg, get_optional_string_arg, get_string_arg, text_result,
};
use async_trait::async_trait;
use kube::api::DeleteParams;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Delete a resource tool.
pub struct DeleteResourceTool {
    client: Arc<K8sClient>,
}

impl DeleteResourceTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        DeleteResourceTool { client }
    }
}

#[async_trait]
impl ToolHandler for DeleteResourceTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let api_version = get_string_arg(&args, "apiVersion")?;
        let kind = get_string_arg(&args, "kind")?;
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");
        let grace_period = get_optional_integer_arg(&args, "gracePeriodSeconds").map(|s| s as u32);
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);

        let params = if force {
            DeleteParams {
                grace_period_seconds: Some(0),
                ..Default::default()
            }
        } else if let Some(seconds) = grace_period {
            DeleteParams {
                grace_period_seconds: Some(seconds),
                ..Default::default()
            }
        } else {
            DeleteParams::default()
        };

        self.delete_resource(&api_version, &kind, &name, namespace.as_deref(), params)
            .await?;

        Ok(text_result(format!(
            "Resource {}/{} deleted successfully",
            kind, name
        )))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "resources_delete",
            "Delete a Kubernetes resource by name",
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
                            .description("Resource kind (e.g., 'Pod', 'Deployment')")
                            .enum_values(vec![
                                "Pod".to_string(),
                                "Deployment".to_string(),
                                "Service".to_string(),
                                "ConfigMap".to_string(),
                                "Secret".to_string(),
                                "Namespace".to_string(),
                                "Ingress".to_string(),
                                "StatefulSet".to_string(),
                                "DaemonSet".to_string(),
                                "ReplicaSet".to_string(),
                                "Job".to_string(),
                                "CronJob".to_string(),
                                "PersistentVolumeClaim".to_string(),
                            ]),
                    ),
                    (
                        "name".to_string(),
                        PropertySchema::string().description("Resource name"),
                    ),
                    (
                        "namespace".to_string(),
                        PropertySchema::string().description("Namespace (optional)"),
                    ),
                    (
                        "gracePeriodSeconds".to_string(),
                        PropertySchema::integer()
                            .description("Grace period in seconds before deletion"),
                    ),
                    (
                        "force".to_string(),
                        PropertySchema::boolean()
                            .description("Force immediate deletion (sets grace period to 0)")
                            .default(json!(false)),
                    ),
                ]))
                .with_required(vec![
                    "apiVersion".to_string(),
                    "kind".to_string(),
                    "name".to_string(),
                ]),
        )
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}

impl DeleteResourceTool {
    async fn delete_resource(
        &self,
        api_version: &str,
        kind: &str,
        name: &str,
        namespace: Option<&str>,
        params: DeleteParams,
    ) -> Result<()> {
        match (api_version, kind) {
            // Core v1 resources
            ("v1", "Pod") => {
                let api = self.client.pods_api(namespace);
                api.delete(name, &params).await?;
            }
            ("v1", "Service") => {
                let api = self.client.services_api(namespace);
                api.delete(name, &params).await?;
            }
            ("v1", "ConfigMap") => {
                let api = self.client.configmaps_api(namespace);
                api.delete(name, &params).await?;
            }
            ("v1", "Secret") => {
                let api = self.client.secrets_api(namespace);
                api.delete(name, &params).await?;
            }
            ("v1", "Namespace") => {
                let api = self.client.namespaces_api();
                api.delete(name, &params).await?;
            }
            ("v1", "PersistentVolumeClaim") => {
                let api = self.client.pvcs_api(namespace);
                api.delete(name, &params).await?;
            }
            // Apps v1 resources
            ("apps/v1", "Deployment") => {
                let api = self.client.deployments_api(namespace);
                api.delete(name, &params).await?;
            }
            ("apps/v1", "StatefulSet") => {
                let api = self.client.statefulsets_api(namespace);
                api.delete(name, &params).await?;
            }
            ("apps/v1", "DaemonSet") => {
                let api = self.client.daemonsets_api(namespace);
                api.delete(name, &params).await?;
            }
            ("apps/v1", "ReplicaSet") => {
                let api = self.client.replicasets_api(namespace);
                api.delete(name, &params).await?;
            }
            // Batch v1 resources
            ("batch/v1", "Job") => {
                let api = self.client.jobs_api(namespace);
                api.delete(name, &params).await?;
            }
            ("batch/v1", "CronJob") => {
                let api = self.client.cronjobs_api(namespace);
                api.delete(name, &params).await?;
            }
            // Networking v1 resources
            ("networking.k8s.io/v1", "Ingress") => {
                let api = self.client.ingresses_api(namespace);
                api.delete(name, &params).await?;
            }
            _ => {
                return Err(Error::InvalidParameter(format!(
                    "Unsupported resource type: {}/{}",
                    api_version, kind
                )));
            }
        }

        Ok(())
    }
}

/// Delete pod tool.
pub struct PodsDeleteTool {
    client: Arc<K8sClient>,
}

impl PodsDeleteTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        PodsDeleteTool { client }
    }
}

#[async_trait]
impl ToolHandler for PodsDeleteTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");
        let force = args.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
        let grace_period = get_optional_integer_arg(&args, "gracePeriodSeconds").map(|s| s as u32);

        let params = if force {
            DeleteParams {
                grace_period_seconds: Some(0),
                ..Default::default()
            }
        } else if let Some(seconds) = grace_period {
            DeleteParams {
                grace_period_seconds: Some(seconds),
                ..Default::default()
            }
        } else {
            DeleteParams::default()
        };

        let api = self.client.pods_api(namespace.as_deref());
        api.delete(&name, &params).await?;

        Ok(text_result(format!("Pod {} deleted successfully", name)))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "pods_delete",
            "Delete a Kubernetes Pod",
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
                    (
                        "gracePeriodSeconds".to_string(),
                        PropertySchema::integer().description("Grace period in seconds"),
                    ),
                    (
                        "force".to_string(),
                        PropertySchema::boolean()
                            .description("Force immediate deletion")
                            .default(json!(false)),
                    ),
                ]))
                .with_required(vec!["name".to_string()]),
        )
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}

/// Delete deployment tool.
pub struct DeploymentsDeleteTool {
    client: Arc<K8sClient>,
}

impl DeploymentsDeleteTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        DeploymentsDeleteTool { client }
    }
}

#[async_trait]
impl ToolHandler for DeploymentsDeleteTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");

        let api = self.client.deployments_api(namespace.as_deref());
        api.delete(&name, &DeleteParams::default()).await?;

        Ok(text_result(format!(
            "Deployment {} deleted successfully",
            name
        )))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "deployments_delete",
            "Delete a Kubernetes Deployment",
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

    fn is_write_tool(&self) -> bool {
        true
    }
}

/// Delete namespace tool.
pub struct NamespacesDeleteTool {
    client: Arc<K8sClient>,
}

impl NamespacesDeleteTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        NamespacesDeleteTool { client }
    }
}

#[async_trait]
impl ToolHandler for NamespacesDeleteTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;

        let api = self.client.namespaces_api();
        api.delete(&name, &DeleteParams::default()).await?;

        Ok(text_result(format!(
            "Namespace {} deleted successfully",
            name
        )))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "namespaces_delete",
            "Delete a Kubernetes Namespace",
            ToolInputSchema::object()
                .with_properties(HashMap::from([(
                    "name".to_string(),
                    PropertySchema::string().description("Namespace name"),
                )]))
                .with_required(vec!["name".to_string()]),
        )
    }

    fn is_write_tool(&self) -> bool {
        true
    }
}

//! List resources tool implementation.

use crate::error::Result;
use crate::format::OutputFormat;
use crate::k8s::{AdaptiveResource, K8sClient, parse_api_version};
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{
    ToolHandler, get_optional_integer_arg, get_optional_string_arg, text_result,
};
use async_trait::async_trait;
use kube::api::ListParams;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::k8s::ApiDiscovery;

/// List resources tool.
pub struct ListResourcesTool {
    client: Arc<K8sClient>,
    discovery: Arc<RwLock<ApiDiscovery>>,
}

impl ListResourcesTool {
    pub fn new(client: Arc<K8sClient>, discovery: Arc<RwLock<ApiDiscovery>>) -> Self {
        ListResourcesTool { client, discovery }
    }
}

#[async_trait]
impl ToolHandler for ListResourcesTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let api_version =
            get_optional_string_arg(&args, "apiVersion").unwrap_or_else(|| "v1".to_string());
        let kind = get_optional_string_arg(&args, "kind").unwrap_or_else(|| "Pod".to_string());
        let namespace = get_optional_string_arg(&args, "namespace");
        let label_selector = get_optional_string_arg(&args, "labelSelector");
        let field_selector = get_optional_string_arg(&args, "fieldSelector");
        let limit = get_optional_integer_arg(&args, "limit").map(|l| l as u32);
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let mut params = ListParams::default();

        if let Some(selector) = label_selector {
            params = params.labels(&selector);
        }
        if let Some(selector) = field_selector {
            params = params.fields(&selector);
        }
        if let Some(limit) = limit {
            params = params.limit(limit);
        }

        let result = self
            .list_resources(
                &api_version,
                &kind,
                namespace.as_deref(),
                params,
                output_format,
            )
            .await?;
        Ok(text_result(result))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "resources_list",
            "List Kubernetes resources with optional selectors",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "apiVersion".to_string(),
                    PropertySchema::string()
                        .description("API version (e.g., 'v1', 'apps/v1')")
                        .enum_values(vec![
                            "v1".to_string(),
                            "apps/v1".to_string(),
                            "batch/v1".to_string(),
                            "networking.k8s.io/v1".to_string(),
                        ])
                        .default(json!("v1")),
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
                            "Node".to_string(),
                            "Ingress".to_string(),
                            "StatefulSet".to_string(),
                            "DaemonSet".to_string(),
                            "Job".to_string(),
                            "CronJob".to_string(),
                            "ReplicaSet".to_string(),
                            "PersistentVolumeClaim".to_string(),
                        ])
                        .default(json!("Pod")),
                ),
                (
                    "namespace".to_string(),
                    PropertySchema::string().description(
                        "Namespace (optional, lists from all namespaces if not specified)",
                    ),
                ),
                (
                    "labelSelector".to_string(),
                    PropertySchema::string()
                        .description("Label selector (e.g., 'app=myapp,tier=frontend')"),
                ),
                (
                    "fieldSelector".to_string(),
                    PropertySchema::string()
                        .description("Field selector (e.g., 'status.phase=Running')"),
                ),
                (
                    "limit".to_string(),
                    PropertySchema::integer().description("Maximum number of results"),
                ),
                (
                    "output".to_string(),
                    PropertySchema::string()
                        .description("Output format")
                        .enum_values(vec![
                            "table".to_string(),
                            "json".to_string(),
                            "yaml".to_string(),
                        ])
                        .default(json!("table")),
                ),
            ])),
        )
    }
}

impl ListResourcesTool {
    async fn list_resources(
        &self,
        api_version: &str,
        kind: &str,
        namespace: Option<&str>,
        params: ListParams,
        format: OutputFormat,
    ) -> Result<String> {
        match (api_version, kind) {
            // Core v1 resources
            ("v1", "Pod") => {
                let api = self.client.pods_api(namespace).await?;
                let pods = api.list(&params).await?;
                Ok(format.format_pods(&pods.items))
            }
            ("v1", "Service") => {
                let api = self.client.services_api(namespace).await?;
                let services = api.list(&params).await?;
                Ok(format.format_services(&services.items))
            }
            ("v1", "ConfigMap") => {
                let api = self.client.configmaps_api(namespace).await?;
                let cms = api.list(&params).await?;
                Ok(format.format_configmaps(&cms.items))
            }
            ("v1", "Secret") => {
                let api = self.client.secrets_api(namespace).await?;
                let secrets = api.list(&params).await?;
                Ok(format.format_secrets(&secrets.items))
            }
            ("v1", "Namespace") => {
                let api = self.client.namespaces_api().await?;
                let namespaces = api.list(&params).await?;
                Ok(format.format_namespaces(&namespaces.items))
            }
            ("v1", "Node") => {
                let api = self.client.nodes_api().await?;
                let nodes = api.list(&params).await?;
                Ok(format.format_nodes(&nodes.items))
            }
            ("v1", "PersistentVolumeClaim") => {
                let api = self.client.pvcs_api(namespace).await?;
                let pvcs = api.list(&params).await?;
                Ok(format.format_pvcs(&pvcs.items))
            }
            ("v1", "PersistentVolume") => {
                let api = self.client.pvs_api().await?;
                let pvs = api.list(&params).await?;
                Ok(format.format_pvs(&pvs.items))
            }
            ("v1", "Event") => {
                let api = self.client.events_api(namespace).await?;
                let events = api.list(&params).await?;
                Ok(format.format_events(&events.items))
            }
            // Apps v1 resources
            ("apps/v1", "Deployment") => {
                let api = self.client.deployments_api(namespace).await?;
                let deploys = api.list(&params).await?;
                Ok(format.format_deployments(&deploys.items))
            }
            ("apps/v1", "StatefulSet") => {
                let api = self.client.statefulsets_api(namespace).await?;
                let sts = api.list(&params).await?;
                Ok(format.format_statefulsets(&sts.items))
            }
            ("apps/v1", "DaemonSet") => {
                let api = self.client.daemonsets_api(namespace).await?;
                let ds = api.list(&params).await?;
                Ok(format.format_daemonsets(&ds.items))
            }
            ("apps/v1", "ReplicaSet") => {
                let api = self.client.replicasets_api(namespace).await?;
                let rs = api.list(&params).await?;
                Ok(format.format_replicasets(&rs.items))
            }
            // Batch v1 resources
            ("batch/v1", "Job") => {
                let api = self.client.jobs_api(namespace).await?;
                let jobs = api.list(&params).await?;
                Ok(format.format_jobs(&jobs.items))
            }
            ("batch/v1", "CronJob") => {
                let api = self.client.cronjobs_api(namespace).await?;
                let cjs = api.list(&params).await?;
                Ok(format.format_cronjobs(&cjs.items))
            }
            // Networking v1 resources
            ("networking.k8s.io/v1", "Ingress") => {
                let api = self.client.ingresses_api(namespace).await?;
                let ingresses = api.list(&params).await?;
                Ok(format.format_ingresses(&ingresses.items))
            }
            // Use adaptive API for all other resource types
            _ => {
                let gvk = parse_api_version(api_version, kind);
                let adaptive = AdaptiveResource::new(self.client.clone(), self.discovery.clone());
                let resources = adaptive.list(&gvk, namespace, params).await?;
                // For unknown types, return JSON format
                Ok(serde_json::to_string_pretty(&resources)?)
            }
        }
    }
}

/// List pods tool (convenience wrapper).
pub struct PodsListTool {
    client: Arc<K8sClient>,
}

impl PodsListTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        PodsListTool { client }
    }
}

#[async_trait]
impl ToolHandler for PodsListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let namespace = get_optional_string_arg(&args, "namespace");
        let label_selector = get_optional_string_arg(&args, "labelSelector");
        let field_selector = get_optional_string_arg(&args, "fieldSelector");
        let all_namespaces = args
            .get("all_namespaces")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let mut params = ListParams::default();
        if let Some(selector) = label_selector {
            params = params.labels(&selector);
        }
        if let Some(selector) = field_selector {
            params = params.fields(&selector);
        }

        let api = if all_namespaces {
            self.client.pods_api(None).await?
        } else {
            self.client.pods_api(namespace.as_deref()).await?
        };

        let pods = api.list(&params).await?;
        Ok(text_result(output_format.format_pods(&pods.items)))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "pods_list",
            "List Kubernetes Pods",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "namespace".to_string(),
                    PropertySchema::string().description("Namespace (optional)"),
                ),
                (
                    "labelSelector".to_string(),
                    PropertySchema::string().description("Label selector"),
                ),
                (
                    "fieldSelector".to_string(),
                    PropertySchema::string()
                        .description("Field selector (e.g., 'status.phase=Running')"),
                ),
                (
                    "all_namespaces".to_string(),
                    PropertySchema::boolean()
                        .description("List pods from all namespaces")
                        .default(json!(false)),
                ),
                (
                    "output".to_string(),
                    PropertySchema::string()
                        .description("Output format")
                        .enum_values(vec![
                            "table".to_string(),
                            "json".to_string(),
                            "yaml".to_string(),
                        ])
                        .default(json!("table")),
                ),
            ])),
        )
    }
}

/// List deployments tool.
pub struct DeploymentsListTool {
    client: Arc<K8sClient>,
}

impl DeploymentsListTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        DeploymentsListTool { client }
    }
}

#[async_trait]
impl ToolHandler for DeploymentsListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let namespace = get_optional_string_arg(&args, "namespace");
        let label_selector = get_optional_string_arg(&args, "labelSelector");
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let mut params = ListParams::default();
        if let Some(selector) = label_selector {
            params = params.labels(&selector);
        }

        let api = self.client.deployments_api(namespace.as_deref()).await?;
        let deploys = api.list(&params).await?;

        Ok(text_result(
            output_format.format_deployments(&deploys.items),
        ))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "deployments_list",
            "List Kubernetes Deployments",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "namespace".to_string(),
                    PropertySchema::string().description("Namespace (optional)"),
                ),
                (
                    "labelSelector".to_string(),
                    PropertySchema::string().description("Label selector"),
                ),
                (
                    "output".to_string(),
                    PropertySchema::string()
                        .description("Output format")
                        .enum_values(vec![
                            "table".to_string(),
                            "json".to_string(),
                            "yaml".to_string(),
                        ])
                        .default(json!("table")),
                ),
            ])),
        )
    }
}

/// List services tool.
pub struct ServicesListTool {
    client: Arc<K8sClient>,
}

impl ServicesListTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        ServicesListTool { client }
    }
}

#[async_trait]
impl ToolHandler for ServicesListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let namespace = get_optional_string_arg(&args, "namespace");
        let label_selector = get_optional_string_arg(&args, "labelSelector");
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let mut params = ListParams::default();
        if let Some(selector) = label_selector {
            params = params.labels(&selector);
        }

        let api = self.client.services_api(namespace.as_deref()).await?;
        let services = api.list(&params).await?;

        Ok(text_result(output_format.format_services(&services.items)))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "services_list",
            "List Kubernetes Services",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "namespace".to_string(),
                    PropertySchema::string().description("Namespace (optional)"),
                ),
                (
                    "labelSelector".to_string(),
                    PropertySchema::string().description("Label selector"),
                ),
                (
                    "output".to_string(),
                    PropertySchema::string()
                        .description("Output format")
                        .enum_values(vec![
                            "table".to_string(),
                            "json".to_string(),
                            "yaml".to_string(),
                        ])
                        .default(json!("table")),
                ),
            ])),
        )
    }
}

/// List nodes tool.
pub struct NodesListTool {
    client: Arc<K8sClient>,
}

impl NodesListTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        NodesListTool { client }
    }
}

#[async_trait]
impl ToolHandler for NodesListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let label_selector = get_optional_string_arg(&args, "labelSelector");
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let mut params = ListParams::default();
        if let Some(selector) = label_selector {
            params = params.labels(&selector);
        }

        let api = self.client.nodes_api().await?;
        let nodes = api.list(&params).await?;

        Ok(text_result(output_format.format_nodes(&nodes.items)))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "nodes_list",
            "List Kubernetes Nodes",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "labelSelector".to_string(),
                    PropertySchema::string().description("Label selector"),
                ),
                (
                    "output".to_string(),
                    PropertySchema::string()
                        .description("Output format")
                        .enum_values(vec![
                            "table".to_string(),
                            "json".to_string(),
                            "yaml".to_string(),
                        ])
                        .default(json!("table")),
                ),
            ])),
        )
    }
}

/// List namespaces tool.
pub struct NamespacesListTool {
    client: Arc<K8sClient>,
}

impl NamespacesListTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        NamespacesListTool { client }
    }
}

#[async_trait]
impl ToolHandler for NamespacesListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let api = self.client.namespaces_api().await?;
        let namespaces = api.list(&ListParams::default()).await?;

        Ok(text_result(
            output_format.format_namespaces(&namespaces.items),
        ))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "namespaces_list",
            "List Kubernetes Namespaces",
            ToolInputSchema::object().with_properties(HashMap::from([(
                "output".to_string(),
                PropertySchema::string()
                    .description("Output format")
                    .enum_values(vec![
                        "table".to_string(),
                        "json".to_string(),
                        "yaml".to_string(),
                    ])
                    .default(json!("table")),
            )])),
        )
    }
}

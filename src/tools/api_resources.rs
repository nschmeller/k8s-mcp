//! API resources discovery tool implementation.

use crate::error::Result;
use crate::k8s::{ApiDiscovery, K8sClient};
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{ToolHandler, get_optional_string_arg, text_result};
use async_trait::async_trait;
use comfy_table::{Cell, Table};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// List API resources tool.
pub struct ApiResourcesListTool {
    client: Arc<K8sClient>,
    discovery: Arc<RwLock<ApiDiscovery>>,
}

impl ApiResourcesListTool {
    pub fn new(client: Arc<K8sClient>, discovery: Arc<RwLock<ApiDiscovery>>) -> Self {
        ApiResourcesListTool { client, discovery }
    }
}

#[async_trait]
impl ToolHandler for ApiResourcesListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let api_group = get_optional_string_arg(&args, "apiGroup");
        let output_format =
            get_optional_string_arg(&args, "output").unwrap_or_else(|| "table".to_string());

        // Run discovery if not already done
        {
            let mut discovery = self.discovery.write().await;
            if !discovery.is_discovered() {
                let client = self.client.inner().await?;
                discovery.discover(&client).await?;
            }
        }

        let discovery = self.discovery.read().await;
        let resources = discovery.list();

        // Filter by API group if specified
        let resources: Vec<_> = if let Some(group) = &api_group {
            resources
                .into_iter()
                .filter(|r| &r.group == group)
                .collect()
        } else {
            resources.into_iter().collect()
        };

        match output_format.as_str() {
            "json" => Ok(text_result(serde_json::to_string_pretty(&resources)?)),
            "yaml" => Ok(text_result(serde_yaml::to_string(&resources)?)),
            _ => {
                let mut table = Table::new();
                table.set_header(vec![
                    "NAME",
                    "SHORTNAMES",
                    "APIGROUP",
                    "NAMESPACED",
                    "KIND",
                    "VERBS",
                ]);

                for r in resources {
                    let short_names = r.short_names.join(",");
                    let verbs = r.verbs.join(",");
                    let group = if r.group.is_empty() {
                        "".to_string()
                    } else {
                        r.group.clone()
                    };

                    table.add_row(vec![
                        Cell::new(&r.name),
                        Cell::new(short_names),
                        Cell::new(group),
                        Cell::new(r.namespaced),
                        Cell::new(&r.kind),
                        Cell::new(verbs),
                    ]);
                }

                Ok(text_result(table.to_string()))
            }
        }
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "api_resources",
            "List the API resources available in the cluster",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "apiGroup".to_string(),
                    PropertySchema::string().description("Filter by API group (optional)"),
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
                        .default(serde_json::json!("table")),
                ),
            ])),
        )
    }
}

/// List API versions tool.
pub struct ApiVersionsTool {
    client: Arc<K8sClient>,
    discovery: Arc<RwLock<ApiDiscovery>>,
}

impl ApiVersionsTool {
    pub fn new(client: Arc<K8sClient>, discovery: Arc<RwLock<ApiDiscovery>>) -> Self {
        ApiVersionsTool { client, discovery }
    }
}

#[async_trait]
impl ToolHandler for ApiVersionsTool {
    async fn call(&self, _args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        // Run discovery if not already done
        {
            let mut discovery = self.discovery.write().await;
            if !discovery.is_discovered() {
                let client = self.client.inner().await?;
                discovery.discover(&client).await?;
            }
        }

        let discovery = self.discovery.read().await;
        let resources = discovery.list();

        // Collect unique API versions
        let mut versions: std::collections::HashSet<String> = std::collections::HashSet::new();
        for r in resources {
            let version = if r.group.is_empty() {
                r.api_version.clone()
            } else {
                format!("{}/{}", r.group, r.api_version)
            };
            versions.insert(version);
        }

        let mut versions: Vec<String> = versions.into_iter().collect();
        versions.sort();

        let mut table = Table::new();
        table.set_header(vec!["GROUP/VERSION"]);

        for v in versions {
            table.add_row(vec![Cell::new(v)]);
        }

        Ok(text_result(table.to_string()))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "api_versions",
            "List the API versions available in the cluster",
            ToolInputSchema::object(),
        )
    }
}

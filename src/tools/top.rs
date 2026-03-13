//! Top (metrics) tool implementation.

use crate::error::Result;
use crate::k8s::{K8sClient, MetricsClient};
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{get_optional_string_arg, text_result, ToolHandler};
use async_trait::async_trait;
use comfy_table::{Cell, Table};
use std::collections::HashMap;
use std::sync::Arc;

/// Top nodes tool.
pub struct TopNodesTool {
    client: Arc<K8sClient>,
}

impl TopNodesTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        TopNodesTool { client }
    }
}

#[async_trait]
impl ToolHandler for TopNodesTool {
    async fn call(&self, _args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let metrics_client = MetricsClient::new((*self.client).clone());
        let node_metrics = metrics_client.top_nodes().await?;

        let mut table = Table::new();
        table.set_header(vec![
            "NAME",
            "CPU(cores)",
            "CPU%",
            "MEMORY(bytes)",
            "MEMORY%",
        ]);

        for m in node_metrics {
            let cpu = m
                .cpu_usage
                .map(|c| format!("{}m", c))
                .unwrap_or_else(|| "N/A".to_string());
            let cpu_percent = m
                .cpu_percent
                .map(|p| format!("{}%", p))
                .unwrap_or_else(|| "N/A".to_string());
            let memory = m
                .memory_usage
                .map(format_bytes)
                .unwrap_or_else(|| "N/A".to_string());
            let memory_percent = m
                .memory_percent
                .map(|p| format!("{}%", p))
                .unwrap_or_else(|| "N/A".to_string());

            table.add_row(vec![
                Cell::new(m.name),
                Cell::new(cpu),
                Cell::new(cpu_percent),
                Cell::new(memory),
                Cell::new(memory_percent),
            ]);
        }

        Ok(text_result(table.to_string()))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "nodes_top",
            "Show resource usage (CPU/Memory) for nodes",
            ToolInputSchema::object(),
        )
    }
}

/// Top pods tool.
pub struct TopPodsTool {
    client: Arc<K8sClient>,
}

impl TopPodsTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        TopPodsTool { client }
    }
}

#[async_trait]
impl ToolHandler for TopPodsTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let namespace = get_optional_string_arg(&args, "namespace");
        let label_selector = get_optional_string_arg(&args, "labelSelector");

        let metrics_client = MetricsClient::new((*self.client).clone());
        let pod_metrics = metrics_client
            .top_pods(namespace.as_deref(), label_selector.as_deref())
            .await?;

        let mut table = Table::new();
        table.set_header(vec!["NAME", "NAMESPACE", "CPU(cores)", "MEMORY(bytes)"]);

        for m in pod_metrics {
            let cpu = m
                .cpu_usage
                .map(|c| format!("{}m", c))
                .unwrap_or_else(|| "N/A".to_string());
            let memory = m
                .memory_usage
                .map(format_bytes)
                .unwrap_or_else(|| "N/A".to_string());

            table.add_row(vec![
                Cell::new(m.name),
                Cell::new(m.namespace),
                Cell::new(cpu),
                Cell::new(memory),
            ]);
        }

        Ok(text_result(table.to_string()))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "pods_top",
            "Show resource usage (CPU/Memory) for pods",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "namespace".to_string(),
                    PropertySchema::string().description("Namespace (optional)"),
                ),
                (
                    "labelSelector".to_string(),
                    PropertySchema::string().description("Label selector"),
                ),
            ])),
        )
    }
}

/// Format bytes in human-readable format.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{}Gi", bytes / GB)
    } else if bytes >= MB {
        format!("{}Mi", bytes / MB)
    } else if bytes >= KB {
        format!("{}Ki", bytes / KB)
    } else {
        format!("{}B", bytes)
    }
}

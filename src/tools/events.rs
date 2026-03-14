//! Events tool implementation.

use crate::error::Result;
use crate::format::{OutputFormat, format_events_table};
use crate::k8s::K8sClient;
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{ToolHandler, get_optional_string_arg, text_result};
use async_trait::async_trait;
use kube::api::ListParams;
use std::collections::HashMap;
use std::sync::Arc;

/// List events tool.
pub struct EventsListTool {
    client: Arc<K8sClient>,
}

impl EventsListTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        EventsListTool { client }
    }
}

#[async_trait]
impl ToolHandler for EventsListTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let namespace = get_optional_string_arg(&args, "namespace");
        let field_selector = get_optional_string_arg(&args, "fieldSelector");
        let output_format = get_optional_string_arg(&args, "output")
            .map(|o| OutputFormat::from(o.as_str()))
            .unwrap_or(OutputFormat::Table);

        let mut params = ListParams::default();
        if let Some(selector) = field_selector {
            params = params.fields(&selector);
        }

        let api = self.client.events_api(namespace.as_deref()).await?;
        let events = api.list(&params).await?;

        match output_format {
            OutputFormat::Json => {
                let output = serde_json::to_string_pretty(&events.items)?;
                Ok(text_result(output))
            }
            OutputFormat::Yaml => {
                let output = serde_yaml::to_string(&events.items)?;
                Ok(text_result(output))
            }
            OutputFormat::Table => {
                let output = format_events_table(&events.items);
                Ok(text_result(output))
            }
        }
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "events_list",
            "List Kubernetes events",
            ToolInputSchema::object().with_properties(HashMap::from([
                (
                    "namespace".to_string(),
                    PropertySchema::string().description(
                        "Namespace (optional, lists from all namespaces if not specified)",
                    ),
                ),
                (
                    "fieldSelector".to_string(),
                    PropertySchema::string()
                        .description("Field selector (e.g., 'involvedObject.name=my-pod')"),
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

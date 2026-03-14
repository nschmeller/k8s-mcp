//! Pod logs tool implementation.

use crate::error::Result;
use crate::k8s::K8sClient;
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{
    ToolHandler, get_optional_integer_arg, get_optional_string_arg, get_string_arg, text_result,
};
use async_trait::async_trait;
use kube::api::LogParams;
use std::collections::HashMap;
use std::sync::Arc;

/// Get pod logs tool.
pub struct PodsLogsTool {
    client: Arc<K8sClient>,
}

impl PodsLogsTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        PodsLogsTool { client }
    }
}

#[async_trait]
impl ToolHandler for PodsLogsTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let name = get_string_arg(&args, "name")?;
        let namespace = get_optional_string_arg(&args, "namespace");
        let container = get_optional_string_arg(&args, "container");
        let tail_lines = get_optional_integer_arg(&args, "tailLines");
        let since_seconds = get_optional_integer_arg(&args, "sinceSeconds");
        let previous = args
            .get("previous")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let api = self.client.pods_api(namespace.as_deref()).await?;

        let log_params = LogParams {
            container,
            tail_lines,
            since_seconds,
            previous,
            ..Default::default()
        };

        let logs = api.logs(&name, &log_params).await?;

        Ok(text_result(logs))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "pods_log",
            "Get logs from a Kubernetes Pod",
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
                        "container".to_string(),
                        PropertySchema::string()
                            .description("Container name (optional, for multi-container pods)"),
                    ),
                    (
                        "tailLines".to_string(),
                        PropertySchema::integer()
                            .description("Number of lines to show from the end of the logs"),
                    ),
                    (
                        "sinceSeconds".to_string(),
                        PropertySchema::integer()
                            .description("Show logs since this many seconds ago"),
                    ),
                    (
                        "previous".to_string(),
                        PropertySchema::boolean()
                            .description("Show previous terminated container logs")
                            .default(serde_json::json!(false)),
                    ),
                ]))
                .with_required(vec!["name".to_string()]),
        )
    }
}

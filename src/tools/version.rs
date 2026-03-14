//! Kubernetes version tool implementation.

use crate::error::Result;
use crate::k8s::K8sClient;
use crate::mcp::protocol::{CallToolResult, Tool, ToolInputSchema};
use crate::tools::registry::{ToolHandler, text_result};
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Get Kubernetes version tool.
pub struct KubernetesVersionTool {
    client: Arc<K8sClient>,
}

impl KubernetesVersionTool {
    pub fn new(client: Arc<K8sClient>) -> Self {
        KubernetesVersionTool { client }
    }
}

#[async_trait]
impl ToolHandler for KubernetesVersionTool {
    async fn call(&self, _args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let version = self.client.kubernetes_version().await;

        let result = match version {
            Some(v) => {
                json!({
                    "major": v.major,
                    "minor": v.minor,
                    "gitVersion": v.git_version,
                    "platform": v.platform,
                    "gitCommit": v.git_commit,
                    "buildDate": v.build_date,
                    "goVersion": v.go_version,
                    "compiler": v.compiler,
                    "shortVersion": v.short_version(),
                })
            }
            None => {
                json!({
                    "error": "Version not available. Ensure the cluster is connected.",
                    "connected": self.client.is_connected().await,
                })
            }
        };

        Ok(text_result(serde_json::to_string_pretty(&result)?))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "kubernetes_version",
            "Get the Kubernetes cluster version information",
            ToolInputSchema::object(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_definition() {
        // Test that the tool definition is correct
        let tool = KubernetesVersionTool::new(Arc::new(K8sClient::from_client(
            // Create a minimal mock client - the actual client won't be used for definition
            kube::Client::try_from(kube::Config::new("http://localhost:8080".parse().unwrap()))
                .unwrap(),
            "default".to_string(),
        )));

        let def = tool.definition();
        assert_eq!(def.name, "kubernetes_version");
        assert_eq!(
            def.description,
            "Get the Kubernetes cluster version information"
        );
    }

    #[tokio::test]
    async fn test_tool_returns_not_connected_when_no_version() {
        let tool = KubernetesVersionTool::new(Arc::new(K8sClient::from_client(
            kube::Client::try_from(kube::Config::new("http://localhost:8080".parse().unwrap()))
                .unwrap(),
            "default".to_string(),
        )));

        let result = tool.call(HashMap::new()).await.unwrap();
        // The tool should return a result even when version is None
        assert!(!result.content.is_empty());
    }
}

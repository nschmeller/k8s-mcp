//! Context management tool implementation.

use crate::error::Result;
use crate::k8s::{K8sClient, K8sConfig};
use crate::mcp::protocol::{CallToolResult, PropertySchema, Tool, ToolInputSchema};
use crate::tools::registry::{text_result, ToolHandler};
use async_trait::async_trait;
use comfy_table::{Cell, Table};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// List contexts tool.
pub struct ContextsListTool {
    config: K8sConfig,
}

impl ContextsListTool {
    pub fn new(config: K8sConfig) -> Self {
        ContextsListTool { config }
    }
}

#[async_trait]
impl ToolHandler for ContextsListTool {
    async fn call(&self, _args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let contexts = self.config.list_contexts()?;
        let current = self.config.current_context()?;

        let mut table = Table::new();
        table.set_header(vec!["NAME", "CLUSTER", "AUTHINFO", "NAMESPACE", "CURRENT"]);

        for ctx in contexts {
            let is_current = current.as_ref().map(|c| c == &ctx.name).unwrap_or(false);
            table.add_row(vec![
                Cell::new(ctx.name),
                Cell::new(ctx.cluster.as_deref().unwrap_or("")),
                Cell::new(ctx.user.as_deref().unwrap_or("")),
                Cell::new(ctx.namespace.as_deref().unwrap_or("")),
                Cell::new(if is_current { "*" } else { "" }),
            ]);
        }

        Ok(text_result(table.to_string()))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "configuration_contexts_list",
            "List all contexts in the kubeconfig",
            ToolInputSchema::object(),
        )
    }
}

/// Get current context tool.
pub struct ContextCurrentTool {
    client: Arc<K8sClient>,
    config: K8sConfig,
}

impl ContextCurrentTool {
    pub fn new(client: Arc<K8sClient>, config: K8sConfig) -> Self {
        ContextCurrentTool { client, config }
    }
}

#[async_trait]
impl ToolHandler for ContextCurrentTool {
    async fn call(&self, _args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let current = self.config.current_context()?;
        let default_namespace = self.client.default_namespace().to_string();

        let result = json!({
            "currentContext": current,
            "defaultNamespace": default_namespace,
        });

        Ok(text_result(serde_json::to_string_pretty(&result)?))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "context_current",
            "Get the current Kubernetes context and default namespace",
            ToolInputSchema::object(),
        )
    }
}

/// View kubeconfig tool.
pub struct ConfigurationViewTool {
    config: K8sConfig,
}

impl ConfigurationViewTool {
    pub fn new(config: K8sConfig) -> Self {
        ConfigurationViewTool { config }
    }
}

#[async_trait]
impl ToolHandler for ConfigurationViewTool {
    async fn call(&self, args: HashMap<String, serde_json::Value>) -> Result<CallToolResult> {
        let minified = args
            .get("minified")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let kubeconfig = self.config.load().await?;

        let output = if minified {
            // Return only the relevant parts for the current context
            let current_context = kubeconfig.current_context.clone();
            let contexts: Vec<_> = kubeconfig
                .contexts
                .into_iter()
                .filter(|c| Some(&c.name) == current_context.as_ref())
                .collect();

            let clusters: Vec<_> = kubeconfig
                .clusters
                .into_iter()
                .filter(|c| {
                    contexts.iter().any(|ctx| {
                        ctx.context
                            .as_ref()
                            .map(|ctx_inner| ctx_inner.cluster == c.name)
                            .unwrap_or(false)
                    })
                })
                .collect();

            let users: Vec<_> = kubeconfig
                .auth_infos
                .into_iter()
                .filter(|u| {
                    contexts.iter().any(|ctx| {
                        ctx.context
                            .as_ref()
                            .map(|ctx_inner| ctx_inner.user == Some(u.name.clone()))
                            .unwrap_or(false)
                    })
                })
                .collect();

            let minified_config = kube::config::Kubeconfig {
                current_context,
                contexts,
                clusters,
                auth_infos: users,
                ..Default::default()
            };

            serde_yaml::to_string(&minified_config)?
        } else {
            serde_yaml::to_string(&kubeconfig)?
        };

        Ok(text_result(output))
    }

    fn definition(&self) -> Tool {
        Tool::new(
            "configuration_view",
            "Get the current Kubernetes configuration content as a kubeconfig YAML",
            ToolInputSchema::object()
                .with_properties(HashMap::from([
                    ("minified".to_string(), PropertySchema::boolean()
                        .description("Return a minified version of the configuration. If set to true, keeps only the current-context and the relevant pieces of the configuration for that context.")
                        .default(json!(true))),
                ])),
        )
    }
}

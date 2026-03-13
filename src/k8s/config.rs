//! Kubernetes configuration loading.
//!
//! This module provides configuration management for connecting to Kubernetes clusters.
//! It supports loading kubeconfig files, context selection, and in-cluster configuration.
//!
//! # Example
//!
//! ```
//! use k8s_mcp::k8s::K8sConfig;
//!
//! // Create a default configuration
//! let config = K8sConfig::new();
//!
//! // Create with a specific context
//! let config = K8sConfig::new()
//!     .with_context("production");
//!
//! // Create with a custom kubeconfig path
//! let config = K8sConfig::new()
//!     .with_kubeconfig("/path/to/kubeconfig");
//! ```

use crate::error::{Error, Result};
use kube::config::{KubeConfigOptions, Kubeconfig};
use std::path::PathBuf;
use tracing::{debug, info};

/// Configuration options for the Kubernetes client.
///
/// This struct uses the builder pattern for configuration.
///
/// # Example
///
/// ```
/// use k8s_mcp::k8s::K8sConfig;
///
/// let config = K8sConfig::new()
///     .with_context("my-context")
///     .with_kubeconfig("/custom/kubeconfig");
///
/// assert_eq!(config.context, Some("my-context".to_string()));
/// ```
#[derive(Debug, Clone, Default)]
pub struct K8sConfig {
    /// Path to kubeconfig file
    pub kubeconfig_path: Option<PathBuf>,
    /// Context to use
    pub context: Option<String>,
    /// Cluster URL (overrides kubeconfig)
    pub cluster_url: Option<String>,
}

impl K8sConfig {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the kubeconfig path.
    pub fn with_kubeconfig(mut self, path: impl Into<PathBuf>) -> Self {
        self.kubeconfig_path = Some(path.into());
        self
    }

    /// Set the context to use.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set the cluster URL.
    pub fn with_cluster_url(mut self, url: impl Into<String>) -> Self {
        self.cluster_url = Some(url.into());
        self
    }

    /// Load the kubeconfig.
    pub async fn load(&self) -> Result<Kubeconfig> {
        let kubeconfig_path = match &self.kubeconfig_path {
            Some(path) => {
                debug!("Loading kubeconfig from: {:?}", path);
                path.clone()
            }
            None => {
                // Use default kubeconfig location
                let default_path = dirs::home_dir()
                    .map(|h| h.join(".kube").join("config"))
                    .ok_or_else(|| {
                        Error::Config("Could not determine default kubeconfig path".to_string())
                    })?;
                debug!("Loading kubeconfig from default path: {:?}", default_path);
                default_path
            }
        };

        let kubeconfig = Kubeconfig::read_from(&kubeconfig_path).map_err(|e| {
            Error::Config(format!(
                "Failed to read kubeconfig from {:?}: {}",
                kubeconfig_path, e
            ))
        })?;

        info!(
            "Loaded kubeconfig with {} contexts",
            kubeconfig.contexts.len()
        );
        Ok(kubeconfig)
    }

    /// Get the list of available contexts.
    pub fn list_contexts(&self) -> Result<Vec<ContextInfo>> {
        let kubeconfig = futures::executor::block_on(self.load())?;
        let current = kubeconfig.current_context.clone();
        let contexts: Vec<ContextInfo> = kubeconfig
            .contexts
            .iter()
            .map(|c| {
                let ctx = c.context.as_ref();
                ContextInfo {
                    name: c.name.clone(),
                    cluster: ctx.map(|cx| cx.cluster.clone()),
                    user: ctx.and_then(|cx| cx.user.clone()),
                    namespace: ctx.and_then(|cx| cx.namespace.clone()),
                    is_current: Some(current.as_ref() == Some(&c.name)),
                }
            })
            .collect();
        Ok(contexts)
    }

    /// Get the current context name.
    pub fn current_context(&self) -> Result<Option<String>> {
        let kubeconfig = futures::executor::block_on(self.load())?;
        Ok(kubeconfig.current_context)
    }

    /// Create kubeconfig options for the specified context.
    pub fn kubeconfig_options(&self) -> KubeConfigOptions {
        KubeConfigOptions {
            context: self.context.clone(),
            cluster: None,
            user: None,
        }
    }
}

/// Information about a kubeconfig context.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextInfo {
    /// Context name
    pub name: String,
    /// Cluster name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<String>,
    /// User name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Default namespace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    /// Is this the current context
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_current: Option<bool>,
}

/// Check if we're running inside a Kubernetes cluster.
pub fn is_in_cluster() -> bool {
    std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
        && std::env::var("KUBERNETES_SERVICE_PORT").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k8s_config_default() {
        let config = K8sConfig::new();
        assert!(config.kubeconfig_path.is_none());
        assert!(config.context.is_none());
        assert!(config.cluster_url.is_none());
    }

    #[test]
    fn test_k8s_config_builder() {
        let config = K8sConfig::new()
            .with_context("my-context")
            .with_kubeconfig("/path/to/config");

        assert_eq!(config.context, Some("my-context".to_string()));
        assert_eq!(
            config.kubeconfig_path,
            Some(PathBuf::from("/path/to/config"))
        );
    }
}

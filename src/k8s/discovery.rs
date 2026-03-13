//! Kubernetes API discovery.
//!
//! This module provides API resource discovery functionality for caching
//! and resolving Kubernetes API resources.
//!
//! # Example
//!
//! ```no_run
//! use k8s_mcp::k8s::ApiDiscovery;
//!
//! // Create a new discovery cache
//! let mut discovery = ApiDiscovery::new();
//! assert!(!discovery.is_discovered());
//!
//! // Use common resources without cluster access
//! use k8s_mcp::k8s::discovery::shortcuts::common_resources;
//! let resources = common_resources();
//! assert!(!resources.is_empty());
//! ```

use crate::error::Result;
use kube::{discovery::Scope, Client, Discovery};
use std::collections::HashMap;
use tracing::info;

/// Discovered API resource information.
///
/// Contains metadata about a Kubernetes API resource including its
/// name, kind, API version, and capabilities.
///
/// # Example
///
/// ```
/// use k8s_mcp::k8s::discovery::shortcuts::common_resources;
///
/// let resources = common_resources();
/// let pod = resources.iter().find(|r| r.kind == "Pod").unwrap();
///
/// assert_eq!(pod.name, "pods");
/// assert!(pod.namespaced);
/// assert!(pod.short_names.contains(&"po".to_string()));
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiResourceInfo {
    /// Resource name (e.g., "pods")
    pub name: String,
    /// Singular name (e.g., "pod")
    pub singular: String,
    /// Resource kind (e.g., "Pod")
    pub kind: String,
    /// API version (e.g., "v1")
    pub api_version: String,
    /// API group (e.g., "", "apps", "batch")
    pub group: String,
    /// Whether the resource is namespaced
    pub namespaced: bool,
    /// Short names (e.g., ["po", "pod"])
    pub short_names: Vec<String>,
    /// Verbs available (e.g., ["get", "list", "create"])
    pub verbs: Vec<String>,
    /// Categories (e.g., ["all"])
    pub categories: Vec<String>,
}

/// API discovery cache.
pub struct ApiDiscovery {
    /// Cached resources by (group, version, kind)
    resources: HashMap<(String, String, String), ApiResourceInfo>,
    /// Resources by short name
    short_names: HashMap<String, (String, String, String)>,
    /// Resources by kind
    kinds: HashMap<String, (String, String, String)>,
    /// Whether discovery has been run
    discovered: bool,
}

impl ApiDiscovery {
    /// Create a new API discovery cache.
    pub fn new() -> Self {
        ApiDiscovery {
            resources: HashMap::new(),
            short_names: HashMap::new(),
            kinds: HashMap::new(),
            discovered: false,
        }
    }

    /// Run discovery and cache the results.
    pub async fn discover(&mut self, client: &Client) -> Result<()> {
        info!("Running API discovery...");

        let discovery = Discovery::new(client.clone())
            .run()
            .await
            .map_err(crate::error::Error::Kubernetes)?;

        self.resources.clear();
        self.short_names.clear();
        self.kinds.clear();

        for group in discovery.groups() {
            let group_name = group.name().to_string();
            for version in group.versions() {
                let version_str = version.to_string();
                let resources = group.versioned_resources(version);
                for (resource, caps) in resources {
                    let info = ApiResourceInfo {
                        name: resource.plural.clone(),
                        singular: resource.plural.clone().trim_end_matches('s').to_string(),
                        kind: resource.kind.clone(),
                        api_version: version_str.clone(),
                        group: group_name.clone(),
                        namespaced: caps.scope == Scope::Namespaced,
                        short_names: vec![], // Not available in kube 3.0 ApiResource
                        verbs: vec![],       // Not available in kube 3.0 ApiResource
                        categories: vec![],  // Not available in kube 3.0 ApiResource
                    };

                    let key = (
                        info.group.clone(),
                        info.api_version.clone(),
                        info.kind.clone(),
                    );
                    self.resources.insert(key.clone(), info.clone());

                    // Index by kind
                    self.kinds.insert(info.kind.to_lowercase(), key.clone());

                    // Also index by plural name
                    self.short_names
                        .insert(info.name.to_lowercase(), key.clone());
                }
            }
        }

        self.discovered = true;
        info!("Discovered {} API resources", self.resources.len());

        Ok(())
    }

    /// Get a resource by group, version, and kind.
    pub fn get(&self, group: &str, version: &str, kind: &str) -> Option<&ApiResourceInfo> {
        self.resources
            .get(&(group.to_string(), version.to_string(), kind.to_string()))
    }

    /// Resolve a resource by name (kind, short name, or plural).
    pub fn resolve(&self, name: &str) -> Option<&ApiResourceInfo> {
        let name_lower = name.to_lowercase();

        // Try short names first
        if let Some(key) = self.short_names.get(&name_lower) {
            return self.resources.get(key);
        }

        // Try kind
        if let Some(key) = self.kinds.get(&name_lower) {
            return self.resources.get(key);
        }

        None
    }

    /// List all discovered resources.
    pub fn list(&self) -> Vec<&ApiResourceInfo> {
        self.resources.values().collect()
    }

    /// List resources by category.
    pub fn list_by_category(&self, category: &str) -> Vec<&ApiResourceInfo> {
        self.resources
            .values()
            .filter(|r| r.categories.iter().any(|c| c == category))
            .collect()
    }

    /// Check if discovery has been run.
    pub fn is_discovered(&self) -> bool {
        self.discovered
    }

    /// Get the number of discovered resources.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if there are no discovered resources.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

impl Default for ApiDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Common resource shortcuts for quick access.
pub mod shortcuts {
    use super::ApiResourceInfo;

    /// Get common resource info for built-in types.
    pub fn common_resources() -> Vec<ApiResourceInfo> {
        vec![
            // Core v1
            ApiResourceInfo {
                name: "pods".to_string(),
                singular: "pod".to_string(),
                kind: "Pod".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: true,
                short_names: vec!["po".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "deletecollection".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "services".to_string(),
                singular: "service".to_string(),
                kind: "Service".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: true,
                short_names: vec!["svc".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "configmaps".to_string(),
                singular: "configmap".to_string(),
                kind: "ConfigMap".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: true,
                short_names: vec!["cm".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "secrets".to_string(),
                singular: "secret".to_string(),
                kind: "Secret".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: true,
                short_names: vec!["secret".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec![],
            },
            ApiResourceInfo {
                name: "namespaces".to_string(),
                singular: "namespace".to_string(),
                kind: "Namespace".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: false,
                short_names: vec!["ns".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec![],
            },
            ApiResourceInfo {
                name: "nodes".to_string(),
                singular: "node".to_string(),
                kind: "Node".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: false,
                short_names: vec!["no".to_string()],
                verbs: vec![
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec![],
            },
            ApiResourceInfo {
                name: "events".to_string(),
                singular: "event".to_string(),
                kind: "Event".to_string(),
                api_version: "v1".to_string(),
                group: "".to_string(),
                namespaced: true,
                short_names: vec!["ev".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec![],
            },
            // Apps v1
            ApiResourceInfo {
                name: "deployments".to_string(),
                singular: "deployment".to_string(),
                kind: "Deployment".to_string(),
                api_version: "v1".to_string(),
                group: "apps".to_string(),
                namespaced: true,
                short_names: vec!["deploy".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "deletecollection".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "statefulsets".to_string(),
                singular: "statefulset".to_string(),
                kind: "StatefulSet".to_string(),
                api_version: "v1".to_string(),
                group: "apps".to_string(),
                namespaced: true,
                short_names: vec!["sts".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "deletecollection".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "daemonsets".to_string(),
                singular: "daemonset".to_string(),
                kind: "DaemonSet".to_string(),
                api_version: "v1".to_string(),
                group: "apps".to_string(),
                namespaced: true,
                short_names: vec!["ds".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "replicasets".to_string(),
                singular: "replicaset".to_string(),
                kind: "ReplicaSet".to_string(),
                api_version: "v1".to_string(),
                group: "apps".to_string(),
                namespaced: true,
                short_names: vec!["rs".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "deletecollection".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            // Batch v1
            ApiResourceInfo {
                name: "jobs".to_string(),
                singular: "job".to_string(),
                kind: "Job".to_string(),
                api_version: "v1".to_string(),
                group: "batch".to_string(),
                namespaced: true,
                short_names: vec!["job".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "deletecollection".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            ApiResourceInfo {
                name: "cronjobs".to_string(),
                singular: "cronjob".to_string(),
                kind: "CronJob".to_string(),
                api_version: "v1".to_string(),
                group: "batch".to_string(),
                namespaced: true,
                short_names: vec!["cj".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "deletecollection".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
            // Networking v1
            ApiResourceInfo {
                name: "ingresses".to_string(),
                singular: "ingress".to_string(),
                kind: "Ingress".to_string(),
                api_version: "v1".to_string(),
                group: "networking.k8s.io".to_string(),
                namespaced: true,
                short_names: vec!["ing".to_string()],
                verbs: vec![
                    "create".to_string(),
                    "delete".to_string(),
                    "get".to_string(),
                    "list".to_string(),
                    "patch".to_string(),
                    "update".to_string(),
                    "watch".to_string(),
                ],
                categories: vec!["all".to_string()],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_discovery_new() {
        let discovery = ApiDiscovery::new();
        assert!(!discovery.is_discovered());
        assert!(discovery.is_empty());
    }

    #[test]
    fn test_common_resources() {
        let resources = shortcuts::common_resources();
        assert!(!resources.is_empty());

        // Check that we have the expected resources
        let kinds: Vec<&str> = resources.iter().map(|r| r.kind.as_str()).collect();
        assert!(kinds.contains(&"Pod"));
        assert!(kinds.contains(&"Deployment"));
        assert!(kinds.contains(&"Service"));
    }
}

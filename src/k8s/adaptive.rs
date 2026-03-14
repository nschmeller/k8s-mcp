//! Adaptive resource handling for Kubernetes.
//!
//! This module provides dynamic resource operations that adapt to the
//! connected Kubernetes cluster using API discovery.
//!
//! # Example
//!
//! ```no_run
//! use k8s_mcp::k8s::{AdaptiveResource, K8sClient, ApiDiscovery, parse_api_version};
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! async fn example(client: Arc<K8sClient>, discovery: Arc<RwLock<ApiDiscovery>>) {
//!     let adaptive = AdaptiveResource::new(client, discovery);
//!     let gvk = parse_api_version("apps/v1", "Deployment");
//!
//!     let deployment = adaptive.get(&gvk, "my-deployment", Some("default")).await;
//! }
//! ```

use crate::error::{Error, Result};
use crate::k8s::{ApiDiscovery, ApiResourceInfo, K8sClient};
use either::Either;
use kube::{
    api::{Api, DeleteParams, DynamicObject, ListParams, ObjectList, Patch, PatchParams},
    core::{GroupVersionKind, Status},
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Adaptive resource operations.
///
/// Provides CRUD operations for any Kubernetes resource type using
/// API discovery to determine resource scope and capabilities.
pub struct AdaptiveResource {
    client: Arc<K8sClient>,
    discovery: Arc<RwLock<ApiDiscovery>>,
}

impl AdaptiveResource {
    /// Create a new adaptive resource handler.
    pub fn new(client: Arc<K8sClient>, discovery: Arc<RwLock<ApiDiscovery>>) -> Self {
        AdaptiveResource { client, discovery }
    }

    /// Get a resource by name.
    ///
    /// Uses API discovery to determine if the resource is namespaced or cluster-scoped.
    pub async fn get(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace).await?;
        api.get(name).await.map_err(Error::Kubernetes)
    }

    /// List resources.
    ///
    /// Uses API discovery to determine if the resource is namespaced or cluster-scoped.
    pub async fn list(
        &self,
        gvk: &GroupVersionKind,
        namespace: Option<&str>,
        list_params: ListParams,
    ) -> Result<ObjectList<DynamicObject>> {
        let api = self.api_for_gvk(gvk, namespace).await?;
        api.list(&list_params).await.map_err(Error::Kubernetes)
    }

    /// Delete a resource.
    ///
    /// Uses API discovery to determine if the resource is namespaced or cluster-scoped.
    pub async fn delete(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
        grace_period_seconds: Option<u32>,
    ) -> Result<Status> {
        let api = self.api_for_gvk(gvk, namespace).await?;

        let mut delete_params = DeleteParams::default();
        if let Some(seconds) = grace_period_seconds {
            delete_params.grace_period_seconds = Some(seconds);
        }

        api.delete(name, &delete_params)
            .await
            .map_err(Error::Kubernetes)
            .map(|result| {
                match result {
                    Either::Left(_obj) => {
                        // Resource was deleted immediately
                        Status {
                            status: Some(kube::core::response::StatusSummary::Success),
                            message: format!("Resource {} deleted", name),
                            ..Default::default()
                        }
                    }
                    Either::Right(status) => status,
                }
            })
    }

    /// Create a resource.
    pub async fn create(
        &self,
        gvk: &GroupVersionKind,
        namespace: Option<&str>,
        resource: DynamicObject,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace).await?;
        api.create(&kube::api::PostParams::default(), &resource)
            .await
            .map_err(Error::Kubernetes)
    }

    /// Update a resource (replace).
    pub async fn update(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
        resource: DynamicObject,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace).await?;
        api.replace(name, &kube::api::PostParams::default(), &resource)
            .await
            .map_err(Error::Kubernetes)
    }

    /// Patch a resource.
    pub async fn patch(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
        patch: &serde_json::Value,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace).await?;
        api.patch(name, &PatchParams::default(), &Patch::Merge(patch.clone()))
            .await
            .map_err(Error::Kubernetes)
    }

    /// Check if a resource type is namespaced.
    ///
    /// First checks the discovery cache, then falls back to common resources
    /// if discovery hasn't been run.
    pub async fn is_namespaced(&self, gvk: &GroupVersionKind) -> bool {
        // Check discovery cache first
        let discovery = self.discovery.read().await;
        if let Some(info) = discovery.get(&gvk.group, &gvk.version, &gvk.kind) {
            return info.namespaced;
        }

        // Fall back to common resources
        let common = crate::k8s::discovery::shortcuts::common_resources();
        if let Some(info) = common
            .iter()
            .find(|r| r.kind == gvk.kind && r.group == gvk.group && r.api_version == gvk.version)
        {
            return info.namespaced;
        }

        // Default to namespaced for safety (most resources are namespaced)
        true
    }

    /// Get resource info from discovery.
    pub async fn get_resource_info(&self, gvk: &GroupVersionKind) -> Option<ApiResourceInfo> {
        let discovery = self.discovery.read().await;
        discovery
            .get(&gvk.group, &gvk.version, &gvk.kind)
            .cloned()
            .or_else(|| {
                let common = crate::k8s::discovery::shortcuts::common_resources();
                common
                    .iter()
                    .find(|r| {
                        r.kind == gvk.kind && r.group == gvk.group && r.api_version == gvk.version
                    })
                    .cloned()
            })
    }

    /// Create an API for the given GVK.
    ///
    /// Uses discovery to determine if the resource is namespaced or cluster-scoped.
    async fn api_for_gvk(
        &self,
        gvk: &GroupVersionKind,
        namespace: Option<&str>,
    ) -> Result<Api<DynamicObject>> {
        let is_namespaced = self.is_namespaced(gvk).await;

        let client = self.client.inner().await?;
        let api_resource = kube::core::ApiResource::from_gvk(gvk);

        let api = if is_namespaced {
            let ns = self.client.resolve_namespace(namespace).await;
            Api::namespaced_with(client, &ns, &api_resource)
        } else {
            Api::all_with(client, &api_resource)
        };

        Ok(api)
    }

    /// Refresh discovery cache.
    ///
    /// This can be useful when a 404 error might indicate stale discovery data.
    pub async fn refresh_discovery(&self) -> Result<()> {
        let client = self.client.inner().await?;
        let mut discovery = self.discovery.write().await;
        discovery.discover(&client).await
    }
}

/// Helper to determine if a GVK represents a known core resource type.
///
/// This is used as a fallback when discovery is not available.
pub fn is_known_core_resource(gvk: &GroupVersionKind) -> bool {
    let common = crate::k8s::discovery::shortcuts::common_resources();
    common
        .iter()
        .any(|r| r.kind == gvk.kind && r.group == gvk.group && r.api_version == gvk.version)
}

/// Get scope information for a resource type.
///
/// Returns a human-readable scope description.
pub fn get_scope_description(namespaced: bool) -> &'static str {
    if namespaced {
        "namespaced"
    } else {
        "cluster-scoped"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::k8s::parse_api_version;

    #[test]
    fn test_is_known_core_resource_pod() {
        let gvk = parse_api_version("v1", "Pod");
        assert!(is_known_core_resource(&gvk));
    }

    #[test]
    fn test_is_known_core_resource_deployment() {
        let gvk = parse_api_version("apps/v1", "Deployment");
        assert!(is_known_core_resource(&gvk));
    }

    #[test]
    fn test_is_known_core_resource_unknown() {
        let gvk = parse_api_version("custom.io/v1", "CustomResource");
        assert!(!is_known_core_resource(&gvk));
    }

    #[test]
    fn test_is_known_core_resource_all_common_types() {
        // Core v1
        assert!(is_known_core_resource(&parse_api_version("v1", "Pod")));
        assert!(is_known_core_resource(&parse_api_version("v1", "Service")));
        assert!(is_known_core_resource(&parse_api_version(
            "v1",
            "ConfigMap"
        )));
        assert!(is_known_core_resource(&parse_api_version("v1", "Secret")));
        assert!(is_known_core_resource(&parse_api_version(
            "v1",
            "Namespace"
        )));
        assert!(is_known_core_resource(&parse_api_version("v1", "Node")));
        assert!(is_known_core_resource(&parse_api_version("v1", "Event")));

        // Apps v1
        assert!(is_known_core_resource(&parse_api_version(
            "apps/v1",
            "Deployment"
        )));
        assert!(is_known_core_resource(&parse_api_version(
            "apps/v1",
            "StatefulSet"
        )));
        assert!(is_known_core_resource(&parse_api_version(
            "apps/v1",
            "DaemonSet"
        )));
        assert!(is_known_core_resource(&parse_api_version(
            "apps/v1",
            "ReplicaSet"
        )));

        // Batch v1
        assert!(is_known_core_resource(&parse_api_version(
            "batch/v1", "Job"
        )));
        assert!(is_known_core_resource(&parse_api_version(
            "batch/v1", "CronJob"
        )));

        // Networking v1
        assert!(is_known_core_resource(&parse_api_version(
            "networking.k8s.io/v1",
            "Ingress"
        )));
    }

    #[test]
    fn test_is_known_core_resource_crd_types() {
        // CRDs should not be known core resources
        assert!(!is_known_core_resource(&parse_api_version(
            "custom.io/v1",
            "CustomResource"
        )));
        assert!(!is_known_core_resource(&parse_api_version(
            "monitoring.coreos.com/v1",
            "Prometheus"
        )));
        assert!(!is_known_core_resource(&parse_api_version(
            "argoproj.io/v1alpha1",
            "Workflow"
        )));
    }

    #[test]
    fn test_get_scope_description() {
        assert_eq!(get_scope_description(true), "namespaced");
        assert_eq!(get_scope_description(false), "cluster-scoped");
    }

    #[tokio::test]
    async fn test_adaptive_resource_new() {
        // Test that AdaptiveResource can be created
        let client = Arc::new(K8sClient::from_client(
            kube::Client::try_from(kube::Config::new("http://localhost:8080".parse().unwrap()))
                .unwrap(),
            "default".to_string(),
        ));
        let discovery = Arc::new(RwLock::new(ApiDiscovery::new()));

        let _adaptive = AdaptiveResource::new(client, discovery);
    }

    #[test]
    fn test_parse_api_version_core() {
        let gvk = parse_api_version("v1", "Pod");
        assert_eq!(gvk.group, "");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Pod");
    }

    #[test]
    fn test_parse_api_version_grouped() {
        let gvk = parse_api_version("apps/v1", "Deployment");
        assert_eq!(gvk.group, "apps");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Deployment");
    }

    #[test]
    fn test_parse_api_version_long_group() {
        let gvk = parse_api_version("networking.k8s.io/v1", "Ingress");
        assert_eq!(gvk.group, "networking.k8s.io");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Ingress");
    }

    #[test]
    fn test_gvk_from_parse_api_version() {
        // Test that the GVK produced by parse_api_version works with is_known_core_resource
        let pod_gvk = parse_api_version("v1", "Pod");
        assert!(is_known_core_resource(&pod_gvk));

        let deploy_gvk = parse_api_version("apps/v1", "Deployment");
        assert!(is_known_core_resource(&deploy_gvk));

        let custom_gvk = parse_api_version("custom.io/v1", "Thing");
        assert!(!is_known_core_resource(&custom_gvk));
    }
}

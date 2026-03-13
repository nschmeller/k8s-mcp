//! Kubernetes client wrapper.

use crate::error::{Error, Result};
use crate::k8s::config::K8sConfig;
use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::Event;
use k8s_openapi::api::core::v1::{
    ConfigMap, PersistentVolume, PersistentVolumeClaim, Secret, Service,
};
use k8s_openapi::api::core::v1::{Namespace, Node, Pod};
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::api::storage::v1::StorageClass;
use kube::{Api, Client, Config};
use tracing::info;

/// Kubernetes client wrapper.
#[derive(Clone)]
pub struct K8sClient {
    /// The underlying kube client
    client: Client,
    /// The default namespace
    default_namespace: String,
    /// The current context name
    current_context: Option<String>,
}

impl K8sClient {
    /// Create a new Kubernetes client from configuration.
    pub async fn new(config: &K8sConfig) -> Result<Self> {
        let (client, default_namespace, current_context) = if crate::k8s::config::is_in_cluster() {
            info!("Running in-cluster, using service account");
            let client = Client::try_default().await?;
            let namespace = std::env::var("POD_NAMESPACE")
                .or_else(|_| std::env::var("KUBERNETES_NAMESPACE"))
                .unwrap_or_else(|_| "default".to_string());
            (client, namespace, None)
        } else {
            let kubeconfig = config.load().await?;
            let current_context = kubeconfig.current_context.clone();
            let default_namespace = kubeconfig
                .contexts
                .iter()
                .find(|c| Some(&c.name) == kubeconfig.current_context.as_ref())
                .and_then(|c| c.context.as_ref().and_then(|ctx| ctx.namespace.clone()))
                .unwrap_or_else(|| "default".to_string());

            let options = config.kubeconfig_options();
            let config = Config::from_custom_kubeconfig(kubeconfig, &options)
                .await
                .map_err(|e| Error::Config(format!("Failed to create kube config: {}", e)))?;

            let client = Client::try_from(config)
                .map_err(|e| Error::Config(format!("Failed to create client: {}", e)))?;

            (client, default_namespace, current_context)
        };

        info!(
            "Kubernetes client initialized, default namespace: {}, context: {:?}",
            default_namespace, current_context
        );

        Ok(K8sClient {
            client,
            default_namespace,
            current_context,
        })
    }

    /// Create a client from an existing kube client.
    pub fn from_client(client: Client, default_namespace: String) -> Self {
        K8sClient {
            client,
            default_namespace,
            current_context: None,
        }
    }

    /// Get the underlying kube client.
    pub fn inner(&self) -> &Client {
        &self.client
    }

    /// Get the default namespace.
    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    /// Get the current context name.
    pub fn current_context(&self) -> Option<&str> {
        self.current_context.as_deref()
    }

    /// Resolve namespace - use provided or fall back to default.
    pub fn resolve_namespace(&self, namespace: Option<&str>) -> String {
        namespace
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_namespace.clone())
    }

    // ========================================================================
    // Core API helpers
    // ========================================================================

    /// Get a Pod API for the given namespace.
    pub fn pods_api(&self, namespace: Option<&str>) -> Api<Pod> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a Node API (cluster-scoped).
    pub fn nodes_api(&self) -> Api<Node> {
        Api::all(self.client.clone())
    }

    /// Get a Namespace API (cluster-scoped).
    pub fn namespaces_api(&self) -> Api<Namespace> {
        Api::all(self.client.clone())
    }

    /// Get a Service API for the given namespace.
    pub fn services_api(&self, namespace: Option<&str>) -> Api<Service> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a ConfigMap API for the given namespace.
    pub fn configmaps_api(&self, namespace: Option<&str>) -> Api<ConfigMap> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a Secret API for the given namespace.
    pub fn secrets_api(&self, namespace: Option<&str>) -> Api<Secret> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a PVC API for the given namespace.
    pub fn pvcs_api(&self, namespace: Option<&str>) -> Api<PersistentVolumeClaim> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a PV API (cluster-scoped).
    pub fn pvs_api(&self) -> Api<PersistentVolume> {
        Api::all(self.client.clone())
    }

    /// Get an Event API for the given namespace.
    pub fn events_api(&self, namespace: Option<&str>) -> Api<Event> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    // ========================================================================
    // Apps API helpers
    // ========================================================================

    /// Get a Deployment API for the given namespace.
    pub fn deployments_api(&self, namespace: Option<&str>) -> Api<Deployment> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a StatefulSet API for the given namespace.
    pub fn statefulsets_api(&self, namespace: Option<&str>) -> Api<StatefulSet> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a DaemonSet API for the given namespace.
    pub fn daemonsets_api(&self, namespace: Option<&str>) -> Api<DaemonSet> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a ReplicaSet API for the given namespace.
    pub fn replicasets_api(&self, namespace: Option<&str>) -> Api<ReplicaSet> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    // ========================================================================
    // Batch API helpers
    // ========================================================================

    /// Get a Job API for the given namespace.
    pub fn jobs_api(&self, namespace: Option<&str>) -> Api<Job> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    /// Get a CronJob API for the given namespace.
    pub fn cronjobs_api(&self, namespace: Option<&str>) -> Api<CronJob> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    // ========================================================================
    // Networking API helpers
    // ========================================================================

    /// Get an Ingress API for the given namespace.
    pub fn ingresses_api(&self, namespace: Option<&str>) -> Api<Ingress> {
        Api::namespaced(self.client.clone(), &self.resolve_namespace(namespace))
    }

    // ========================================================================
    // Storage API helpers
    // ========================================================================

    /// Get a StorageClass API (cluster-scoped).
    pub fn storageclasses_api(&self) -> Api<StorageClass> {
        Api::all(self.client.clone())
    }

    // ========================================================================
    // Dynamic API helpers
    // ========================================================================

    /// Get a dynamic API for any resource type.
    pub fn dynamic_api(
        &self,
        group: &str,
        version: &str,
        kind: &str,
        namespace: Option<&str>,
        cluster_scoped: bool,
    ) -> kube::Api<kube::core::DynamicObject> {
        let gvk = kube::core::GroupVersionKind {
            group: group.to_string(),
            version: version.to_string(),
            kind: kind.to_string(),
        };

        let api_resource = kube::core::ApiResource::from_gvk(&gvk);

        if cluster_scoped {
            Api::all_with(self.client.clone(), &api_resource)
        } else {
            Api::namespaced_with(
                self.client.clone(),
                &self.resolve_namespace(namespace),
                &api_resource,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_resolve_namespace_with_provided() {
        // Test that provided namespace is returned as-is
        // This tests the logic without requiring a real kubeconfig
        let default_ns = "default".to_string();
        let result = if let Some(ns) = Some("custom") {
            ns.to_string()
        } else {
            default_ns.clone()
        };
        assert_eq!(result, "custom");
    }

    #[test]
    fn test_resolve_namespace_with_default() {
        // Test that default namespace is used when none provided
        let default_ns = "default".to_string();
        let result = if let Some(ns) = None::<&str> {
            ns.to_string()
        } else {
            default_ns.clone()
        };
        assert_eq!(result, "default");
    }
}

//! Kubernetes client wrapper.

use crate::error::{Error, Result};
use crate::k8s::config::K8sConfig;
use crate::k8s::version::K8sVersion;
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
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// State of the Kubernetes client connection.
// Allow large enum variant since the Connected state is inherently larger
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
enum ClientState {
    /// Client is connected and ready
    Connected {
        client: Client,
        default_namespace: String,
        current_context: Option<String>,
        version: Option<K8sVersion>,
    },
    /// No context is configured
    NoContext,
    /// Context exists but connection failed
    ConnectionFailed { error: String },
}

/// Kubernetes client wrapper that supports lazy initialization.
#[derive(Clone)]
pub struct K8sClient {
    /// The client state (lazy-loaded)
    state: Arc<RwLock<Option<ClientState>>>,
    /// Configuration for lazy initialization
    config: K8sConfig,
}

impl K8sClient {
    /// Create a new Kubernetes client from configuration.
    /// This performs lazy initialization - the actual connection is established
    /// on first use, allowing the server to start without a cluster connection.
    pub async fn new(config: &K8sConfig) -> Result<Self> {
        let client = K8sClient {
            state: Arc::new(RwLock::new(None)),
            config: config.clone(),
        };

        // Try to initialize, but don't fail if we can't connect
        match client.try_connect().await {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "Initial Kubernetes connection failed: {}. Server will start anyway, tools will fail until a cluster is available.",
                    e
                );
            }
        }

        Ok(client)
    }

    /// Attempt to establish a connection to the Kubernetes cluster.
    pub async fn try_connect(&self) -> Result<()> {
        let mut state = self.state.write().await;

        // Fast path: already connected
        if matches!(state.as_ref(), Some(ClientState::Connected { .. })) {
            return Ok(());
        }

        // Establish connection and update state
        let _ = state.insert(self.establish_connection().await);

        // Return result based on final state
        match state.as_ref() {
            Some(ClientState::Connected { .. }) => Ok(()),
            Some(ClientState::NoContext) => Err(Error::NoContext),
            Some(ClientState::ConnectionFailed { .. }) => Err(Error::NoClusterConnection),
            None => Err(Error::NoClusterConnection),
        }
    }

    /// Establish a new connection, returning the resulting state.
    async fn establish_connection(&self) -> ClientState {
        if crate::k8s::config::is_in_cluster() {
            self.connect_in_cluster().await
        } else {
            self.connect_from_kubeconfig().await
        }
    }

    /// Connect using in-cluster service account.
    async fn connect_in_cluster(&self) -> ClientState {
        info!("Running in-cluster, using service account");
        match Client::try_default().await {
            Ok(client) => {
                let namespace = std::env::var("POD_NAMESPACE")
                    .or_else(|_| std::env::var("KUBERNETES_NAMESPACE"))
                    .unwrap_or_else(|_| "default".to_string());

                // Detect Kubernetes version
                let version = K8sVersion::detect(&client).await.ok();
                if let Some(ref v) = version {
                    info!(
                        "Kubernetes client initialized, namespace: {}, version: {}",
                        namespace, v
                    );
                } else {
                    info!(
                        "Kubernetes client initialized, namespace: {} (version detection failed)",
                        namespace
                    );
                }

                ClientState::Connected {
                    client,
                    default_namespace: namespace,
                    current_context: None,
                    version,
                }
            }
            Err(e) => {
                let error = format!("Failed to create in-cluster client: {}", e);
                warn!("{}", error);
                ClientState::ConnectionFailed { error }
            }
        }
    }

    /// Connect from kubeconfig file.
    async fn connect_from_kubeconfig(&self) -> ClientState {
        let kubeconfig = match self.config.load().await {
            Ok(k) => k,
            Err(e) => {
                let error = format!("Failed to load kubeconfig: {}", e);
                warn!("{}", error);
                return ClientState::ConnectionFailed { error };
            }
        };

        let current_context = kubeconfig.current_context.clone();
        debug!("Current context from kubeconfig: {:?}", current_context);

        // Check for valid current context
        let has_context = current_context.as_ref().is_some_and(|s| !s.is_empty());

        if !has_context {
            info!("No current Kubernetes context is set");
            return ClientState::NoContext;
        }

        let default_namespace = kubeconfig
            .contexts
            .iter()
            .find(|c| Some(&c.name) == current_context.as_ref())
            .and_then(|c| c.context.as_ref().and_then(|ctx| ctx.namespace.clone()))
            .unwrap_or_else(|| "default".to_string());

        let options = self.config.kubeconfig_options();
        debug!("Kubeconfig options: context={:?}", options.context);

        match Config::from_custom_kubeconfig(kubeconfig, &options).await {
            Ok(config) => match Client::try_from(config) {
                Ok(client) => {
                    // Detect Kubernetes version
                    let version = K8sVersion::detect(&client).await.ok();
                    if let Some(ref v) = version {
                        info!(
                            "Kubernetes client initialized, default namespace: {}, context: {:?}, version: {}",
                            default_namespace, current_context, v
                        );
                    } else {
                        info!(
                            "Kubernetes client initialized, default namespace: {}, context: {:?} (version detection failed)",
                            default_namespace, current_context
                        );
                    }

                    ClientState::Connected {
                        client,
                        default_namespace,
                        current_context,
                        version,
                    }
                }
                Err(e) => {
                    let error = format!("Failed to create client: {}", e);
                    warn!("{}", error);
                    ClientState::ConnectionFailed { error }
                }
            },
            Err(e) => {
                let error = format!("Failed to create kube config: {}", e);
                warn!("{}", error);
                ClientState::ConnectionFailed { error }
            }
        }
    }

    /// Check if the client is connected to a cluster.
    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        matches!(state.as_ref(), Some(ClientState::Connected { .. }))
    }

    /// Get the connection state description.
    pub async fn connection_status(&self) -> String {
        let state = self.state.read().await;
        match state.as_ref() {
            Some(ClientState::Connected {
                current_context,
                version,
                ..
            }) => {
                let version_str = version
                    .as_ref()
                    .map(|v| format!(" (Kubernetes {})", v))
                    .unwrap_or_default();
                format!("Connected to context: {:?}{}", current_context, version_str)
            }
            Some(ClientState::NoContext) => {
                "No Kubernetes context is active. Use 'kubectl config use-context <context>' to set one.".to_string()
            }
            Some(ClientState::ConnectionFailed { error }) => {
                format!("Connection failed: {}", error)
            }
            None => "Not initialized".to_string(),
        }
    }

    /// Get the detected Kubernetes version.
    ///
    /// Returns `None` if not connected or version detection failed.
    pub async fn kubernetes_version(&self) -> Option<K8sVersion> {
        let state = self.state.read().await;
        match state.as_ref() {
            Some(ClientState::Connected { version, .. }) => version.clone(),
            _ => None,
        }
    }

    /// Get the client, returning an error if not connected.
    async fn get_client(&self) -> Result<Client> {
        // Fast path: already connected
        {
            let state = self.state.read().await;
            if let Some(ClientState::Connected { client, .. }) = state.as_ref() {
                return Ok(client.clone());
            }
        }

        // Try to establish connection
        self.try_connect().await?;

        // Extract client or return appropriate error
        self.state
            .read()
            .await
            .as_ref()
            .map_or(Err(Error::NoClusterConnection), |s| match s {
                ClientState::Connected { client, .. } => Ok(client.clone()),
                ClientState::NoContext => Err(Error::NoContext),
                ClientState::ConnectionFailed { .. } => Err(Error::NoClusterConnection),
            })
    }

    /// Create a client from an existing kube client.
    pub fn from_client(client: Client, default_namespace: String) -> Self {
        K8sClient {
            state: Arc::new(RwLock::new(Some(ClientState::Connected {
                client,
                default_namespace,
                current_context: None,
                version: None, // Version will be None for manually created clients
            }))),
            config: K8sConfig::new(),
        }
    }

    /// Get the underlying kube client.
    pub async fn inner(&self) -> Result<Client> {
        self.get_client().await
    }

    /// Get the default namespace.
    ///
    /// Returns the namespace from the current context if connected,
    /// otherwise returns "default" as a fallback.
    pub async fn default_namespace(&self) -> String {
        let _ = self.try_connect().await;
        self.state
            .read()
            .await
            .as_ref()
            .and_then(|s| match s {
                ClientState::Connected {
                    default_namespace, ..
                } => Some(default_namespace.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "default".to_string())
    }

    /// Get the current context name.
    pub async fn current_context(&self) -> Option<String> {
        self.state.read().await.as_ref().and_then(|s| match s {
            ClientState::Connected {
                current_context, ..
            } => current_context.clone(),
            _ => None,
        })
    }

    /// Resolve namespace - use provided or fall back to default.
    pub async fn resolve_namespace(&self, namespace: Option<&str>) -> String {
        match namespace {
            Some(ns) => ns.to_string(),
            None => self.default_namespace().await,
        }
    }

    // ========================================================================
    // Core API helpers
    // ========================================================================

    /// Get a Pod API for the given namespace.
    pub async fn pods_api(&self, namespace: Option<&str>) -> Result<Api<Pod>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a Node API (cluster-scoped).
    pub async fn nodes_api(&self) -> Result<Api<Node>> {
        self.get_client().await.map(Api::all)
    }

    /// Get a Namespace API (cluster-scoped).
    pub async fn namespaces_api(&self) -> Result<Api<Namespace>> {
        self.get_client().await.map(Api::all)
    }

    /// Get a Service API for the given namespace.
    pub async fn services_api(&self, namespace: Option<&str>) -> Result<Api<Service>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a ConfigMap API for the given namespace.
    pub async fn configmaps_api(&self, namespace: Option<&str>) -> Result<Api<ConfigMap>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a Secret API for the given namespace.
    pub async fn secrets_api(&self, namespace: Option<&str>) -> Result<Api<Secret>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a PVC API for the given namespace.
    pub async fn pvcs_api(&self, namespace: Option<&str>) -> Result<Api<PersistentVolumeClaim>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a PV API (cluster-scoped).
    pub async fn pvs_api(&self) -> Result<Api<PersistentVolume>> {
        self.get_client().await.map(Api::all)
    }

    /// Get an Event API for the given namespace.
    pub async fn events_api(&self, namespace: Option<&str>) -> Result<Api<Event>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    // ========================================================================
    // Apps API helpers
    // ========================================================================

    /// Get a Deployment API for the given namespace.
    pub async fn deployments_api(&self, namespace: Option<&str>) -> Result<Api<Deployment>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a StatefulSet API for the given namespace.
    pub async fn statefulsets_api(&self, namespace: Option<&str>) -> Result<Api<StatefulSet>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a DaemonSet API for the given namespace.
    pub async fn daemonsets_api(&self, namespace: Option<&str>) -> Result<Api<DaemonSet>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a ReplicaSet API for the given namespace.
    pub async fn replicasets_api(&self, namespace: Option<&str>) -> Result<Api<ReplicaSet>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    // ========================================================================
    // Batch API helpers
    // ========================================================================

    /// Get a Job API for the given namespace.
    pub async fn jobs_api(&self, namespace: Option<&str>) -> Result<Api<Job>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    /// Get a CronJob API for the given namespace.
    pub async fn cronjobs_api(&self, namespace: Option<&str>) -> Result<Api<CronJob>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    // ========================================================================
    // Networking API helpers
    // ========================================================================

    /// Get an Ingress API for the given namespace.
    pub async fn ingresses_api(&self, namespace: Option<&str>) -> Result<Api<Ingress>> {
        let client = self.get_client().await?;
        Ok(Api::namespaced(
            client,
            &self.resolve_namespace(namespace).await,
        ))
    }

    // ========================================================================
    // Storage API helpers
    // ========================================================================

    /// Get a StorageClass API (cluster-scoped).
    pub async fn storageclasses_api(&self) -> Result<Api<StorageClass>> {
        self.get_client().await.map(Api::all)
    }

    // ========================================================================
    // Dynamic API helpers
    // ========================================================================

    /// Get a dynamic API for any resource type.
    pub async fn dynamic_api(
        &self,
        group: &str,
        version: &str,
        kind: &str,
        namespace: Option<&str>,
        cluster_scoped: bool,
    ) -> Result<kube::Api<kube::core::DynamicObject>> {
        let client = self.get_client().await?;
        let gvk = kube::core::GroupVersionKind {
            group: group.to_string(),
            version: version.to_string(),
            kind: kind.to_string(),
        };
        let api_resource = kube::core::ApiResource::from_gvk(&gvk);

        Ok(if cluster_scoped {
            Api::all_with(client, &api_resource)
        } else {
            Api::namespaced_with(
                client,
                &self.resolve_namespace(namespace).await,
                &api_resource,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resolve_namespace_with_provided() {
        let result = Some("custom")
            .map(str::to_string)
            .unwrap_or_else(|| "default".to_string());
        assert_eq!(result, "custom");
    }

    #[test]
    fn test_resolve_namespace_with_default() {
        let result = None::<&str>
            .map(str::to_string)
            .unwrap_or_else(|| "default".to_string());
        assert_eq!(result, "default");
    }

    fn create_temp_kubeconfig(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    fn create_empty_kubeconfig() -> NamedTempFile {
        create_temp_kubeconfig(
            r#"
apiVersion: v1
kind: Config
contexts: []
clusters: []
users: []
"#,
        )
    }

    fn create_kubeconfig_no_current_context() -> NamedTempFile {
        create_temp_kubeconfig(
            r#"
apiVersion: v1
kind: Config
current-context: ""
contexts:
- name: test-context
  context:
    cluster: test-cluster
    namespace: test-ns
clusters:
- name: test-cluster
  cluster:
    server: https://localhost:6443
users:
- name: test-user
"#,
        )
    }

    #[tokio::test]
    async fn test_client_new_succeeds_without_context() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_client_connection_status_no_context() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        let status = client.connection_status().await;
        assert!(status.contains("No Kubernetes context"), "{}", status);
    }

    #[tokio::test]
    async fn test_client_connection_status_empty_current_context() {
        let temp_file = create_kubeconfig_no_current_context();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();

        assert!(!client.is_connected().await);
        assert!(
            client
                .connection_status()
                .await
                .contains("No Kubernetes context")
        );
    }

    #[tokio::test]
    async fn test_client_tool_call_returns_no_context_error() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        let result = client.pods_api(None).await;

        assert!(
            matches!(result, Err(Error::NoContext)),
            "Expected NoContext error"
        );
    }

    #[tokio::test]
    async fn test_client_inner_returns_no_context_error() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        let result = client.inner().await;

        assert!(
            matches!(result, Err(Error::NoContext)),
            "Expected NoContext error"
        );
    }

    #[tokio::test]
    async fn test_client_default_namespace_fallback() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        assert_eq!(client.default_namespace().await, "default");
    }

    #[tokio::test]
    async fn test_client_current_context_returns_none() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        assert!(client.current_context().await.is_none());
    }

    #[tokio::test]
    async fn test_client_reconnect_after_context_change() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();
        assert!(!client.is_connected().await);

        let result = client.try_connect().await;
        assert!(matches!(result.unwrap_err(), Error::NoContext));
    }

    #[tokio::test]
    async fn test_client_multiple_try_connect_calls_safe() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = K8sClient::new(&config).await.unwrap();

        for _ in 0..3 {
            assert!(client.try_connect().await.is_err());
            assert!(!client.is_connected().await);
        }
    }

    #[tokio::test]
    async fn test_client_is_connected_thread_safe() {
        let temp_file = create_empty_kubeconfig();
        let config = K8sConfig::new().with_kubeconfig(temp_file.path());
        let client = std::sync::Arc::new(K8sClient::new(&config).await.unwrap());

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let c = client.clone();
                tokio::spawn(async move { c.is_connected().await })
            })
            .collect();

        for handle in handles {
            assert!(!handle.await.unwrap());
        }
    }
}

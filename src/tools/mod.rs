//! MCP Tool implementations for Kubernetes operations.

pub mod api_resources;
pub mod context;
pub mod core;
pub mod events;
pub mod logs;
pub mod registry;
pub mod top;
pub mod version;

pub use api_resources::{ApiResourcesListTool, ApiVersionsTool};
pub use context::{ConfigurationViewTool, ContextCurrentTool, ContextsListTool};
pub use core::{
    DeleteResourceTool, DeploymentsDeleteTool, DeploymentsGetTool, DeploymentsListTool,
    GetResourceTool, ListResourcesTool, NamespacesDeleteTool, NamespacesGetTool,
    NamespacesListTool, NodesGetTool, NodesListTool, PodsDeleteTool, PodsGetTool, PodsListTool,
    ServicesGetTool, ServicesListTool,
};
pub use events::EventsListTool;
pub use logs::PodsLogsTool;
pub use registry::{ToolHandler, ToolRegistry};
pub use top::{TopNodesTool, TopPodsTool};
pub use version::KubernetesVersionTool;

use crate::k8s::{ApiDiscovery, K8sClient, K8sConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Register all tools with the registry.
pub fn register_all_tools(
    registry: &mut ToolRegistry,
    client: K8sClient,
    config: K8sConfig,
    discovery: Arc<RwLock<ApiDiscovery>>,
) {
    let client = Arc::new(client);

    // Core get tools
    registry.register(GetResourceTool::new(client.clone(), discovery.clone()));
    registry.register(PodsGetTool::new(client.clone()));
    registry.register(DeploymentsGetTool::new(client.clone()));
    registry.register(ServicesGetTool::new(client.clone()));
    registry.register(NodesGetTool::new(client.clone()));
    registry.register(NamespacesGetTool::new(client.clone()));

    // Core list tools
    registry.register(ListResourcesTool::new(client.clone(), discovery.clone()));
    registry.register(PodsListTool::new(client.clone()));
    registry.register(DeploymentsListTool::new(client.clone()));
    registry.register(ServicesListTool::new(client.clone()));
    registry.register(NodesListTool::new(client.clone()));
    registry.register(NamespacesListTool::new(client.clone()));

    // Core delete tools
    registry.register(DeleteResourceTool::new(client.clone(), discovery.clone()));
    registry.register(PodsDeleteTool::new(client.clone()));
    registry.register(DeploymentsDeleteTool::new(client.clone()));
    registry.register(NamespacesDeleteTool::new(client.clone()));

    // Logs tool
    registry.register(PodsLogsTool::new(client.clone()));

    // Events tool
    registry.register(EventsListTool::new(client.clone()));

    // Top tools
    registry.register(TopNodesTool::new(client.clone()));
    registry.register(TopPodsTool::new(client.clone()));

    // Context tools
    registry.register(ContextsListTool::new(config.clone()));
    registry.register(ContextCurrentTool::new(client.clone(), config.clone()));
    registry.register(ConfigurationViewTool::new(config));

    // API discovery tools
    registry.register(ApiResourcesListTool::new(client.clone(), discovery.clone()));
    registry.register(ApiVersionsTool::new(client.clone(), discovery.clone()));

    // Version tool
    registry.register(KubernetesVersionTool::new(client));
}

//! Kubernetes client and utilities.
//!
//! This module provides the Kubernetes client wrapper, API discovery,
//! dynamic resource handling, version detection, and metrics support.
//!
//! # Version Detection
//!
//! The [`K8sVersion`] struct provides runtime Kubernetes version detection:
//!
//! ```no_run
//! use k8s_mcp::k8s::K8sVersion;
//! use kube::Client;
//!
//! async fn example(client: &Client) {
//!     let version = K8sVersion::detect(client).await.unwrap();
//!     println!("Kubernetes version: {}", version.git_version);
//!
//!     if version.is_at_least(1, 25) {
//!         println!("Cluster supports PodSecurity admission");
//!     }
//! }
//! ```
//!
//! # Adaptive Resource Operations
//!
//! The [`AdaptiveResource`] struct provides dynamic resource operations
//! that work with any Kubernetes resource type:
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

pub mod adaptive;
pub mod client;
pub mod config;
pub mod discovery;
pub mod metrics;
pub mod resources;
pub mod version;

pub use adaptive::AdaptiveResource;
pub use client::K8sClient;
pub use config::{ContextInfo, K8sConfig};
pub use discovery::{ApiDiscovery, ApiResourceInfo};
pub use metrics::{MetricsClient, NodeMetrics, PodMetrics};
pub use resources::{DynamicResource, PatchStrategy, parse_api_version, parse_gvk};
pub use version::K8sVersion;

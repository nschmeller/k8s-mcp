//! Kubernetes client and utilities.
//!
//! This module provides the Kubernetes client wrapper, API discovery,
//! dynamic resource handling, and metrics support.

pub mod client;
pub mod config;
pub mod discovery;
pub mod metrics;
pub mod resources;

pub use client::K8sClient;
pub use config::{ContextInfo, K8sConfig};
pub use discovery::{ApiDiscovery, ApiResourceInfo};
pub use metrics::{MetricsClient, NodeMetrics, PodMetrics};
pub use resources::{parse_api_version, parse_gvk, DynamicResource, PatchStrategy};

//! Kubernetes metrics support (top nodes/pods).

use crate::error::{Error, Result};
use crate::k8s::client::K8sClient;
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{Api, ResourceExt, api::ListParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Metrics client for retrieving resource usage.
pub struct MetricsClient {
    client: K8sClient,
}

impl MetricsClient {
    /// Create a new metrics client.
    pub fn new(client: K8sClient) -> Self {
        MetricsClient { client }
    }

    /// Get node metrics.
    pub async fn top_nodes(&self) -> Result<Vec<NodeMetrics>> {
        debug!("Fetching node metrics");

        // Get node list
        let nodes_api: Api<Node> = Api::all(self.client.inner().clone());
        let nodes = nodes_api
            .list(&ListParams::default())
            .await
            .map_err(Error::Kubernetes)?;

        // Try to get metrics from metrics-server
        let node_metrics = match self.fetch_node_metrics().await {
            Ok(metrics) => Some(metrics),
            Err(e) => {
                warn!("Could not fetch node metrics from metrics-server: {}", e);
                None
            }
        };

        // Combine node info with metrics
        let mut result = Vec::new();
        for node in nodes.items {
            let name = node.name_any();

            // Parse allocatable resources
            let allocatable = node
                .status
                .as_ref()
                .and_then(|s| s.allocatable.as_ref())
                .cloned()
                .unwrap_or_default();

            let cpu_allocatable = parse_cpu(
                &allocatable
                    .get("cpu")
                    .map(|s| s.0.clone())
                    .unwrap_or_default(),
            );
            let memory_allocatable = parse_memory(
                &allocatable
                    .get("memory")
                    .map(|s| s.0.clone())
                    .unwrap_or_default(),
            );

            // Get metrics if available
            let (cpu_usage, memory_usage, cpu_percent, memory_percent) =
                if let Some(ref metrics) = node_metrics {
                    if let Some(m) = metrics.get(&name) {
                        let cpu_usage =
                            parse_cpu(&m.usage.get("cpu").map(|s| s.0.clone()).unwrap_or_default());
                        let memory_usage = parse_memory(
                            &m.usage
                                .get("memory")
                                .map(|s| s.0.clone())
                                .unwrap_or_default(),
                        );

                        let cpu_percent = if cpu_allocatable > 0 {
                            (cpu_usage as f64 / cpu_allocatable as f64 * 100.0) as u32
                        } else {
                            0
                        };

                        let memory_percent = if memory_allocatable > 0 {
                            (memory_usage as f64 / memory_allocatable as f64 * 100.0) as u32
                        } else {
                            0
                        };

                        (
                            Some(cpu_usage),
                            Some(memory_usage),
                            Some(cpu_percent),
                            Some(memory_percent),
                        )
                    } else {
                        (None, None, None, None)
                    }
                } else {
                    (None, None, None, None)
                };

            result.push(NodeMetrics {
                name: name.clone(),
                cpu_allocatable,
                memory_allocatable,
                cpu_usage,
                memory_usage,
                cpu_percent,
                memory_percent,
            });
        }

        Ok(result)
    }

    /// Get pod metrics.
    pub async fn top_pods(
        &self,
        namespace: Option<&str>,
        label_selector: Option<&str>,
    ) -> Result<Vec<PodMetrics>> {
        debug!("Fetching pod metrics");

        // Get pod list
        let pods_api: Api<Pod> = if let Some(ns) = namespace {
            Api::namespaced(self.client.inner().clone(), ns)
        } else {
            Api::all(self.client.inner().clone())
        };

        let mut list_params = ListParams::default();
        if let Some(selector) = label_selector {
            list_params = list_params.labels(selector);
        }

        let pods = pods_api
            .list(&list_params)
            .await
            .map_err(Error::Kubernetes)?;

        // Try to get metrics from metrics-server
        let pod_metrics = match self.fetch_pod_metrics(namespace).await {
            Ok(metrics) => Some(metrics),
            Err(e) => {
                warn!("Could not fetch pod metrics from metrics-server: {}", e);
                None
            }
        };

        // Combine pod info with metrics
        let mut result = Vec::new();
        for pod in pods.items {
            let name = pod.name_any();
            let ns = pod.namespace().unwrap_or_default();

            // Get metrics if available
            let (cpu_usage, memory_usage) = if let Some(ref metrics) = pod_metrics {
                if let Some(m) = metrics.get(&format!("{}/{}", ns, name)) {
                    // Sum CPU values from all containers
                    let cpu: u64 = m
                        .containers
                        .iter()
                        .map(|c| {
                            parse_cpu(&c.usage.get("cpu").map(|s| s.0.clone()).unwrap_or_default())
                        })
                        .sum();

                    // Sum memory values from all containers
                    let memory: u64 = m
                        .containers
                        .iter()
                        .map(|c| {
                            parse_memory(
                                &c.usage
                                    .get("memory")
                                    .map(|s| s.0.clone())
                                    .unwrap_or_default(),
                            )
                        })
                        .sum();

                    (Some(cpu), Some(memory))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            result.push(PodMetrics {
                name,
                namespace: ns,
                cpu_usage,
                memory_usage,
            });
        }

        Ok(result)
    }

    /// Fetch node metrics from metrics-server.
    async fn fetch_node_metrics(&self) -> Result<HashMap<String, NodeMetricsRaw>> {
        // Use the metrics API
        let client = self.client.inner().clone();

        // Try to access metrics.k8s.io/v1beta1
        let url = "/apis/metrics.k8s.io/v1beta1/nodes";

        let request = http::Request::builder()
            .uri(url)
            .body(vec![])
            .map_err(|e| Error::MetricsUnavailable(e.to_string()))?;

        let response = client
            .request::<NodeMetricsList>(request)
            .await
            .map_err(|e| Error::MetricsUnavailable(e.to_string()))?;

        let mut metrics = HashMap::new();
        for item in response.items {
            if let Some(name) = item.metadata.name.clone() {
                metrics.insert(name, item);
            }
        }

        Ok(metrics)
    }

    /// Fetch pod metrics from metrics-server.
    async fn fetch_pod_metrics(
        &self,
        namespace: Option<&str>,
    ) -> Result<HashMap<String, PodMetricsRaw>> {
        let client = self.client.inner().clone();

        let url = if let Some(ns) = namespace {
            format!("/apis/metrics.k8s.io/v1beta1/namespaces/{}/pods", ns)
        } else {
            "/apis/metrics.k8s.io/v1beta1/pods".to_string()
        };

        let request = http::Request::builder()
            .uri(url)
            .body(vec![])
            .map_err(|e| Error::MetricsUnavailable(e.to_string()))?;

        let response = client
            .request::<PodMetricsList>(request)
            .await
            .map_err(|e| Error::MetricsUnavailable(e.to_string()))?;

        let mut metrics = HashMap::new();
        for item in response.items {
            if let (Some(namespace), Some(name)) = (&item.metadata.namespace, &item.metadata.name) {
                let key = format!("{}/{}", namespace, name);
                metrics.insert(key, item);
            }
        }

        Ok(metrics)
    }
}

/// Node metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node name.
    pub name: String,
    /// CPU allocatable in millicores.
    pub cpu_allocatable: u64,
    /// Memory allocatable in bytes.
    pub memory_allocatable: u64,
    /// CPU usage in millicores (if available).
    pub cpu_usage: Option<u64>,
    /// Memory usage in bytes (if available).
    pub memory_usage: Option<u64>,
    /// CPU usage percentage (if available).
    pub cpu_percent: Option<u32>,
    /// Memory usage percentage (if available).
    pub memory_percent: Option<u32>,
}

/// Pod metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodMetrics {
    /// Pod name.
    pub name: String,
    /// Pod namespace.
    pub namespace: String,
    /// CPU usage in millicores (if available).
    pub cpu_usage: Option<u64>,
    /// Memory usage in bytes (if available).
    pub memory_usage: Option<u64>,
}

/// Raw node metrics from metrics-server.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeMetricsList {
    items: Vec<NodeMetricsRaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeMetricsRaw {
    metadata: kube::core::ObjectMeta,
    usage: HashMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>,
}

/// Raw pod metrics from metrics-server.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PodMetricsList {
    items: Vec<PodMetricsRaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PodMetricsRaw {
    metadata: kube::core::ObjectMeta,
    containers: Vec<ContainerMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContainerMetrics {
    name: String,
    usage: HashMap<String, k8s_openapi::apimachinery::pkg::api::resource::Quantity>,
}

/// Parse CPU quantity to millicores.
fn parse_cpu(s: &str) -> u64 {
    if s.is_empty() {
        return 0;
    }

    // Handle "n" (nanocores)
    if s.ends_with('n') {
        let value: u64 = s.trim_end_matches('n').parse().unwrap_or(0);
        return value / 1_000_000;
    }

    // Handle "u" (microcores)
    if s.ends_with('u') {
        let value: u64 = s.trim_end_matches('u').parse().unwrap_or(0);
        return value / 1_000;
    }

    // Handle "m" (millicores)
    if s.ends_with('m') {
        let value: u64 = s.trim_end_matches('m').parse().unwrap_or(0);
        return value;
    }

    // Plain number = cores
    let value: f64 = s.parse().unwrap_or(0.0);
    (value * 1000.0) as u64
}

/// Parse memory quantity to bytes.
fn parse_memory(s: &str) -> u64 {
    if s.is_empty() {
        return 0;
    }

    // Handle "Ki" (kibibytes)
    if s.ends_with("Ki") {
        let value: u64 = s.trim_end_matches("Ki").parse().unwrap_or(0);
        return value * 1024;
    }

    // Handle "Mi" (mebibytes)
    if s.ends_with("Mi") {
        let value: u64 = s.trim_end_matches("Mi").parse().unwrap_or(0);
        return value * 1024 * 1024;
    }

    // Handle "Gi" (gibibytes)
    if s.ends_with("Gi") {
        let value: u64 = s.trim_end_matches("Gi").parse().unwrap_or(0);
        return value * 1024 * 1024 * 1024;
    }

    // Handle "K" (kilobytes)
    if s.ends_with('K') {
        let value: u64 = s.trim_end_matches('K').parse().unwrap_or(0);
        return value * 1000;
    }

    // Handle "M" (megabytes)
    if s.ends_with('M') {
        let value: u64 = s.trim_end_matches('M').parse().unwrap_or(0);
        return value * 1000 * 1000;
    }

    // Handle "G" (gigabytes)
    if s.ends_with('G') {
        let value: u64 = s.trim_end_matches('G').parse().unwrap_or(0);
        return value * 1000 * 1000 * 1000;
    }

    // Plain number = bytes
    s.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu() {
        assert_eq!(parse_cpu("100m"), 100);
        assert_eq!(parse_cpu("1"), 1000);
        assert_eq!(parse_cpu("500m"), 500);
        assert_eq!(parse_cpu("1000000n"), 1); // 1,000,000 nanocores = 1 millicore
        assert_eq!(parse_cpu("1000u"), 1); // 1000 microcores = 1 millicore
        assert_eq!(parse_cpu(""), 0);
    }

    #[test]
    fn test_parse_memory() {
        assert_eq!(parse_memory("1Ki"), 1024);
        assert_eq!(parse_memory("1Mi"), 1024 * 1024);
        assert_eq!(parse_memory("1Gi"), 1024 * 1024 * 1024);
        assert_eq!(parse_memory("1000"), 1000);
        assert_eq!(parse_memory(""), 0);
    }
}

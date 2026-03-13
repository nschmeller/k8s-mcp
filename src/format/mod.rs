//! Output formatting for Kubernetes resources.
//!
//! This module provides formatting utilities for displaying Kubernetes
//! resources in various output formats (table, JSON, YAML).
//!
//! # Example
//!
//! ```
//! use k8s_mcp::format::OutputFormat;
//!
//! // Parse from string
//! let format = OutputFormat::from("json");
//! assert_eq!(format, OutputFormat::Json);
//!
//! // Default is table
//! let default = OutputFormat::default();
//! assert_eq!(default, OutputFormat::Table);
//! ```

mod table;

use k8s_openapi::api::apps::v1::{DaemonSet, Deployment, ReplicaSet, StatefulSet};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::api::core::v1::{
    ConfigMap, Event, Namespace, Node, PersistentVolume, PersistentVolumeClaim, Pod, Secret,
    Service,
};
use k8s_openapi::api::networking::v1::Ingress;
pub use table::{
    format_configmaps_table, format_cronjobs_table, format_daemonsets_table,
    format_deployments_table, format_events_table, format_ingresses_table, format_jobs_table,
    format_namespaces_table, format_nodes_table, format_pods_table, format_pvcs_table,
    format_pvs_table, format_replicasets_table, format_secrets_table, format_services_table,
    format_statefulsets_table,
};

/// Output format options.
///
/// Specifies how Kubernetes resources should be formatted for display.
///
/// # Example
///
/// ```
/// use k8s_mcp::format::OutputFormat;
///
/// let format = OutputFormat::from("yaml");
/// assert_eq!(format, OutputFormat::Yaml);
///
/// // Unknown formats default to table
/// let unknown = OutputFormat::from("xml");
/// assert_eq!(unknown, OutputFormat::Table);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
}

impl From<&str> for OutputFormat {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "yaml" => OutputFormat::Yaml,
            _ => OutputFormat::Table,
        }
    }
}

impl OutputFormat {
    /// Format a list of pods.
    pub fn format_pods(&self, pods: &[Pod]) -> String {
        match self {
            OutputFormat::Table => format_pods_table(pods),
            OutputFormat::Json => format_as_json(pods),
            OutputFormat::Yaml => format_as_yaml(pods),
        }
    }

    /// Format a list of deployments.
    pub fn format_deployments(&self, deployments: &[Deployment]) -> String {
        match self {
            OutputFormat::Table => format_deployments_table(deployments),
            OutputFormat::Json => format_as_json(deployments),
            OutputFormat::Yaml => format_as_yaml(deployments),
        }
    }

    /// Format a list of services.
    pub fn format_services(&self, services: &[Service]) -> String {
        match self {
            OutputFormat::Table => format_services_table(services),
            OutputFormat::Json => format_as_json(services),
            OutputFormat::Yaml => format_as_yaml(services),
        }
    }

    /// Format a list of configmaps.
    pub fn format_configmaps(&self, configmaps: &[ConfigMap]) -> String {
        match self {
            OutputFormat::Table => format_configmaps_table(configmaps),
            OutputFormat::Json => format_as_json(configmaps),
            OutputFormat::Yaml => format_as_yaml(configmaps),
        }
    }

    /// Format a list of secrets.
    pub fn format_secrets(&self, secrets: &[Secret]) -> String {
        match self {
            OutputFormat::Table => format_secrets_table(secrets),
            OutputFormat::Json => format_as_json(secrets),
            OutputFormat::Yaml => format_as_yaml(secrets),
        }
    }

    /// Format a list of namespaces.
    pub fn format_namespaces(&self, namespaces: &[Namespace]) -> String {
        match self {
            OutputFormat::Table => format_namespaces_table(namespaces),
            OutputFormat::Json => format_as_json(namespaces),
            OutputFormat::Yaml => format_as_yaml(namespaces),
        }
    }

    /// Format a list of nodes.
    pub fn format_nodes(&self, nodes: &[Node]) -> String {
        match self {
            OutputFormat::Table => format_nodes_table(nodes),
            OutputFormat::Json => format_as_json(nodes),
            OutputFormat::Yaml => format_as_yaml(nodes),
        }
    }

    /// Format a list of statefulsets.
    pub fn format_statefulsets(&self, statefulsets: &[StatefulSet]) -> String {
        match self {
            OutputFormat::Table => format_statefulsets_table(statefulsets),
            OutputFormat::Json => format_as_json(statefulsets),
            OutputFormat::Yaml => format_as_yaml(statefulsets),
        }
    }

    /// Format a list of daemonsets.
    pub fn format_daemonsets(&self, daemonsets: &[DaemonSet]) -> String {
        match self {
            OutputFormat::Table => format_daemonsets_table(daemonsets),
            OutputFormat::Json => format_as_json(daemonsets),
            OutputFormat::Yaml => format_as_yaml(daemonsets),
        }
    }

    /// Format a list of replicasets.
    pub fn format_replicasets(&self, replicasets: &[ReplicaSet]) -> String {
        match self {
            OutputFormat::Table => format_replicasets_table(replicasets),
            OutputFormat::Json => format_as_json(replicasets),
            OutputFormat::Yaml => format_as_yaml(replicasets),
        }
    }

    /// Format a list of jobs.
    pub fn format_jobs(&self, jobs: &[Job]) -> String {
        match self {
            OutputFormat::Table => format_jobs_table(jobs),
            OutputFormat::Json => format_as_json(jobs),
            OutputFormat::Yaml => format_as_yaml(jobs),
        }
    }

    /// Format a list of cronjobs.
    pub fn format_cronjobs(&self, cronjobs: &[CronJob]) -> String {
        match self {
            OutputFormat::Table => format_cronjobs_table(cronjobs),
            OutputFormat::Json => format_as_json(cronjobs),
            OutputFormat::Yaml => format_as_yaml(cronjobs),
        }
    }

    /// Format a list of ingresses.
    pub fn format_ingresses(&self, ingresses: &[Ingress]) -> String {
        match self {
            OutputFormat::Table => format_ingresses_table(ingresses),
            OutputFormat::Json => format_as_json(ingresses),
            OutputFormat::Yaml => format_as_yaml(ingresses),
        }
    }

    /// Format a list of PVCs.
    pub fn format_pvcs(&self, pvcs: &[PersistentVolumeClaim]) -> String {
        match self {
            OutputFormat::Table => format_pvcs_table(pvcs),
            OutputFormat::Json => format_as_json(pvcs),
            OutputFormat::Yaml => format_as_yaml(pvcs),
        }
    }

    /// Format a list of PVs.
    pub fn format_pvs(&self, pvs: &[PersistentVolume]) -> String {
        match self {
            OutputFormat::Table => format_pvs_table(pvs),
            OutputFormat::Json => format_as_json(pvs),
            OutputFormat::Yaml => format_as_yaml(pvs),
        }
    }

    /// Format a list of events.
    pub fn format_events(&self, events: &[Event]) -> String {
        match self {
            OutputFormat::Table => format_events_table(events),
            OutputFormat::Json => format_as_json(events),
            OutputFormat::Yaml => format_as_yaml(events),
        }
    }
}

/// Format as JSON.
fn format_as_json<T: serde::Serialize>(items: &[T]) -> String {
    serde_json::to_string_pretty(items).unwrap_or_else(|e| format!("Error formatting JSON: {}", e))
}

/// Format as YAML.
fn format_as_yaml<T: serde::Serialize>(items: &[T]) -> String {
    serde_yaml::to_string(items).unwrap_or_else(|e| format!("Error formatting YAML: {}", e))
}

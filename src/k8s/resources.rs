//! Dynamic resource handling for Kubernetes.
//!
//! This module provides utilities for working with any Kubernetes resource
//! dynamically, without compile-time type information.
//!
//! # Example
//!
//! ```
//! use k8s_mcp::k8s::{parse_api_version, parse_gvk};
//!
//! // Parse API version and kind
//! let gvk = parse_api_version("apps/v1", "Deployment");
//! assert_eq!(gvk.group, "apps");
//! assert_eq!(gvk.version, "v1");
//! assert_eq!(gvk.kind, "Deployment");
//!
//! // Parse GVK from components
//! let gvk = parse_gvk("batch", "v1", "Job");
//! assert_eq!(gvk.group, "batch");
//! ```

use crate::error::{Error, Result};
use crate::k8s::client::K8sClient;
use either::Either;
use kube::core::response::StatusSummary;
use kube::{
    api::{
        Api, DeleteParams, DynamicObject, ListParams, ObjectList, Patch, PatchParams, PostParams,
    },
    core::{GroupVersionKind, Status},
};
use serde_json::Value;

/// Dynamic resource operations.
///
/// Provides CRUD operations for any Kubernetes resource type.
pub struct DynamicResource {
    client: K8sClient,
}

impl DynamicResource {
    /// Create a new dynamic resource handler.
    pub fn new(client: K8sClient) -> Self {
        DynamicResource { client }
    }

    /// Get a resource by name.
    pub async fn get(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace)?;
        api.get(name).await.map_err(Error::Kubernetes)
    }

    /// List resources.
    pub async fn list(
        &self,
        gvk: &GroupVersionKind,
        namespace: Option<&str>,
        list_params: ListParams,
    ) -> Result<ObjectList<DynamicObject>> {
        let api = self.api_for_gvk(gvk, namespace)?;
        api.list(&list_params).await.map_err(Error::Kubernetes)
    }

    /// Create a resource.
    pub async fn create(
        &self,
        gvk: &GroupVersionKind,
        namespace: Option<&str>,
        resource: DynamicObject,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace)?;
        api.create(&PostParams::default(), &resource)
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
        let api = self.api_for_gvk(gvk, namespace)?;
        api.replace(name, &PostParams::default(), &resource)
            .await
            .map_err(Error::Kubernetes)
    }

    /// Patch a resource.
    pub async fn patch(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
        patch: &Value,
        patch_type: PatchStrategy,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace)?;

        let patch = match patch_type {
            PatchStrategy::Merge => Patch::Merge(patch.clone()),
            PatchStrategy::Strategic => Patch::Strategic(patch.clone()),
            PatchStrategy::Json => {
                // Convert Value to json_patch::Patch
                let json_patch: json_patch::Patch = serde_json::from_value(patch.clone())
                    .map_err(|e| Error::Protocol(format!("Invalid JSON patch: {}", e)))?;
                Patch::Json(json_patch)
            }
        };

        api.patch(name, &PatchParams::default(), &patch)
            .await
            .map_err(Error::Kubernetes)
    }

    /// Delete a resource.
    pub async fn delete(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
        grace_period_seconds: Option<u32>,
    ) -> Result<Status> {
        let api = self.api_for_gvk(gvk, namespace)?;

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
                            status: Some(StatusSummary::Success),
                            message: format!("Resource {} deleted", name),
                            ..Default::default()
                        }
                    }
                    Either::Right(status) => status,
                }
            })
    }

    /// Server-side apply a resource.
    pub async fn apply(
        &self,
        gvk: &GroupVersionKind,
        name: &str,
        namespace: Option<&str>,
        resource: &Value,
        field_manager: &str,
    ) -> Result<DynamicObject> {
        let api = self.api_for_gvk(gvk, namespace)?;

        let patch_params = PatchParams {
            field_manager: Some(field_manager.to_string()),
            ..Default::default()
        };

        api.patch(name, &patch_params, &Patch::Apply(resource))
            .await
            .map_err(Error::Kubernetes)
    }

    /// Create an API for the given GVK.
    fn api_for_gvk(
        &self,
        gvk: &GroupVersionKind,
        namespace: Option<&str>,
    ) -> Result<Api<DynamicObject>> {
        let api_resource = kube::core::ApiResource::from_gvk(gvk);

        // Determine if this is a cluster-scoped resource
        // For simplicity, we'll use namespace to determine scope
        let is_namespaced = namespace.is_some();

        let api = if is_namespaced {
            Api::namespaced_with(
                self.client.inner().clone(),
                &self.client.resolve_namespace(namespace),
                &api_resource,
            )
        } else {
            Api::all_with(self.client.inner().clone(), &api_resource)
        };

        Ok(api)
    }
}

/// Patch strategy.
#[derive(Debug, Clone, Copy, Default)]
pub enum PatchStrategy {
    /// JSON Merge Patch
    #[default]
    Merge,
    /// Strategic Merge Patch (deprecated for CRDs)
    Strategic,
    /// JSON Patch
    Json,
}

/// Helper to parse a GVK from strings.
pub fn parse_gvk(group: &str, version: &str, kind: &str) -> GroupVersionKind {
    GroupVersionKind {
        group: group.to_string(),
        version: version.to_string(),
        kind: kind.to_string(),
    }
}

/// Helper to parse a GVK from an API version string and kind.
pub fn parse_api_version(api_version: &str, kind: &str) -> GroupVersionKind {
    let (group, version) = if let Some(pos) = api_version.find('/') {
        (
            api_version[..pos].to_string(),
            api_version[pos + 1..].to_string(),
        )
    } else {
        (String::new(), api_version.to_string())
    };

    GroupVersionKind {
        group,
        version,
        kind: kind.to_string(),
    }
}

/// Helper to create a DynamicObject from a JSON value.
pub fn dynamic_object_from_json(json: Value) -> Result<DynamicObject> {
    serde_json::from_value(json)
        .map_err(|e| Error::Protocol(format!("Failed to parse resource: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_api_version() {
        let gvk = parse_api_version("apps/v1", "Deployment");
        assert_eq!(gvk.group, "apps");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Deployment");

        let gvk = parse_api_version("v1", "Pod");
        assert_eq!(gvk.group, "");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Pod");
    }

    #[test]
    fn test_parse_gvk() {
        let gvk = parse_gvk("batch", "v1", "Job");
        assert_eq!(gvk.group, "batch");
        assert_eq!(gvk.version, "v1");
        assert_eq!(gvk.kind, "Job");
    }
}

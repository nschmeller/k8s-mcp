//! Kind cluster management for integration tests.
//!
//! This module provides utilities to ensure a kind cluster is available
//! and selected as the current kubectl context before running tests.
//! The cluster is automatically torn down after all tests complete.

use std::process::Command;
use std::sync::Once;

const KIND_CLUSTER_NAME: &str = "k8s-mcp-test";

static INIT: Once = Once::new();

/// Ensure a kind cluster is running and selected as current context.
/// This is idempotent and thread-safe - only creates the cluster once.
pub fn ensure_kind_cluster() {
    INIT.call_once(|| {
        // Check if kind is installed
        if !is_kind_installed() {
            panic!("kind is not installed. Install with: go install sigs.k8s.io/kind@latest");
        }

        // Check if our test cluster exists
        if !kind_cluster_exists() {
            println!("\nCreating kind cluster '{}'...", KIND_CLUSTER_NAME);
            create_kind_cluster();
        }

        // Switch to kind context
        switch_to_kind_context();
    });
}

fn is_kind_installed() -> bool {
    Command::new("kind")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn kind_cluster_exists() -> bool {
    let output = Command::new("kind")
        .args(["get", "clusters"])
        .output()
        .expect("Failed to execute kind");

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.trim() == KIND_CLUSTER_NAME)
}

fn create_kind_cluster() {
    let status = Command::new("kind")
        .args(["create", "cluster", "--name", KIND_CLUSTER_NAME])
        .status()
        .expect("Failed to create kind cluster");

    if !status.success() {
        panic!("Failed to create kind cluster '{}'", KIND_CLUSTER_NAME);
    }

    println!("Created kind cluster '{}'", KIND_CLUSTER_NAME);
}

fn switch_to_kind_context() {
    let context_name = format!("kind-{}", KIND_CLUSTER_NAME);

    let status = Command::new("kubectl")
        .args(["config", "use-context", &context_name])
        .status()
        .expect("Failed to switch kubectl context");

    if !status.success() {
        panic!("Failed to switch to context '{}'", context_name);
    }

    println!("Switched to context '{}'", context_name);
}

/// Delete the kind cluster.
pub fn delete_kind_cluster() {
    let status = Command::new("kind")
        .args(["delete", "cluster", "--name", KIND_CLUSTER_NAME])
        .status()
        .expect("Failed to delete kind cluster");

    if status.success() {
        println!("\nDeleted kind cluster '{}'", KIND_CLUSTER_NAME);
    }
}

/// Global teardown to clean up the kind cluster after all tests.
/// This is called automatically via the #[ctor] crate.
#[cfg(test)]
mod teardown {
    use ctor::dtor;

    #[dtor]
    fn cleanup() {
        // Only delete if we created it (cluster exists)
        if super::kind_cluster_exists() {
            super::delete_kind_cluster();
        }
    }
}

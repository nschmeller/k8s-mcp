//! Error types for the Kubernetes MCP server.
//!
//! This module provides a comprehensive error type that covers all possible
//! error conditions in the k8s-mcp server.
//!
//! # Example
//!
//! ```
//! use k8s_mcp::error::{Error, Result};
//!
//! fn might_fail() -> Result<()> {
//!     Err(Error::Config("test error".to_string()))
//! }
//!
//! let result = might_fail();
//! assert!(result.is_err());
//! ```

use thiserror::Error;

/// Main error type for the k8s-mcp server.
///
/// This enum covers all possible error conditions including:
/// - MCP protocol errors
/// - JSON-RPC errors with standard codes
/// - Kubernetes API errors
/// - Configuration errors
/// - IO errors
///
/// # Example
///
/// ```
/// use k8s_mcp::error::Error;
///
/// let err = Error::json_rpc_invalid_params("Missing parameter");
/// assert_eq!(err.json_rpc_code(), -32602);
/// ```
#[derive(Error, Debug)]
pub enum Error {
    /// MCP protocol errors
    #[error("MCP protocol error: {0}")]
    Protocol(String),

    /// JSON-RPC errors
    #[error("JSON-RPC error: code={code}, message={message}")]
    JsonRpc {
        code: i32,
        message: String,
        data: Option<serde_json::Value>,
    },

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// YAML serialization/deserialization errors
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Kubernetes client errors
    #[error("Kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),

    /// Kubernetes API error with status
    #[error("Kubernetes API error: {message}")]
    KubernetesApi {
        status: Box<kube::core::Status>,
        message: String,
    },

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Tool execution errors
    #[error("Tool error: {0}")]
    Tool(String),

    /// Read-only mode violation
    #[error(
        "Operation not permitted in read-only mode. Start with --read-write to enable mutations."
    )]
    ReadOnlyMode,

    /// Resource not found
    #[error("Resource not found: {kind}/{name} in namespace {namespace:?}")]
    ResourceNotFound {
        kind: String,
        name: String,
        namespace: Option<String>,
    },

    /// Invalid parameter errors
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Watch stream errors
    #[error("Watch error: {0}")]
    Watch(String),

    /// Exec errors
    #[error("Exec error: {0}")]
    Exec(String),

    /// Port forward errors
    #[error("Port forward error: {0}")]
    PortForward(String),

    /// Metrics not available
    #[error("Metrics not available: {0}")]
    MetricsUnavailable(String),

    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),

    /// No Kubernetes context configured
    #[error(
        "No Kubernetes context is currently active. Use 'kubectl config use-context <context>' to set one, or specify a context via --context or K8S_CONTEXT environment variable."
    )]
    NoContext,

    /// No Kubernetes cluster connection
    #[error(
        "No Kubernetes cluster connection available. Ensure a context is active and the cluster is reachable."
    )]
    NoClusterConnection,

    /// Kubernetes version mismatch
    #[error("Kubernetes version mismatch: server is {current}, but feature requires {required}")]
    VersionMismatch {
        /// Current cluster version
        current: String,
        /// Required minimum version
        required: String,
    },

    /// Feature not available in current Kubernetes version
    #[error(
        "Feature '{feature}' requires Kubernetes v{required_major}.{required_minor}+, cluster is v{current_major}.{current_minor}"
    )]
    FeatureNotAvailable {
        /// Feature name
        feature: String,
        /// Required major version
        required_major: u32,
        /// Required minor version
        required_minor: u32,
        /// Current major version
        current_major: u32,
        /// Current minor version
        current_minor: u32,
    },

    /// Resource scope mismatch
    #[error(
        "Resource scope mismatch: {kind} is {expected_scope}, but {actual_scope} scope was requested"
    )]
    ResourceScopeMismatch {
        /// Resource kind
        kind: String,
        /// Expected scope
        expected_scope: String,
        /// Actual scope provided
        actual_scope: String,
    },
}

/// Result type alias for k8s-mcp operations.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a JSON-RPC error with standard error codes.
    pub fn json_rpc_invalid_request(message: impl Into<String>) -> Self {
        Error::JsonRpc {
            code: -32600,
            message: message.into(),
            data: None,
        }
    }

    /// Create a JSON-RPC method not found error.
    pub fn json_rpc_method_not_found(method: impl Into<String>) -> Self {
        Error::JsonRpc {
            code: -32601,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    /// Create a JSON-RPC invalid params error.
    pub fn json_rpc_invalid_params(message: impl Into<String>) -> Self {
        Error::JsonRpc {
            code: -32602,
            message: message.into(),
            data: None,
        }
    }

    /// Create a JSON-RPC internal error.
    pub fn json_rpc_internal(message: impl Into<String>) -> Self {
        Error::JsonRpc {
            code: -32603,
            message: message.into(),
            data: None,
        }
    }

    /// Get the JSON-RPC error code if this is a JSON-RPC error.
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            Error::JsonRpc { code, .. } => *code,
            _ => -32603, // Internal error
        }
    }

    /// Check if this error should be reported as a JSON-RPC error.
    pub fn is_json_rpc_error(&self) -> bool {
        matches!(self, Error::JsonRpc { .. })
    }
}

impl From<kube::core::Status> for Error {
    fn from(status: kube::core::Status) -> Self {
        let message = status.message.clone();
        Error::KubernetesApi {
            status: Box::new(status),
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_mismatch_error() {
        let error = Error::VersionMismatch {
            current: "v1.28.0".to_string(),
            required: "v1.30.0".to_string(),
        };

        let message = error.to_string();
        assert!(message.contains("v1.28.0"));
        assert!(message.contains("v1.30.0"));
        assert!(message.contains("version mismatch"));
    }

    #[test]
    fn test_feature_not_available_error() {
        let error = Error::FeatureNotAvailable {
            feature: "PodSecurityPolicies".to_string(),
            required_major: 1,
            required_minor: 25,
            current_major: 1,
            current_minor: 22,
        };

        let message = error.to_string();
        assert!(message.contains("PodSecurityPolicies"));
        assert!(message.contains("v1.25"));
        assert!(message.contains("v1.22"));
    }

    #[test]
    fn test_resource_scope_mismatch_error() {
        let error = Error::ResourceScopeMismatch {
            kind: "Namespace".to_string(),
            expected_scope: "cluster-scoped".to_string(),
            actual_scope: "namespaced".to_string(),
        };

        let message = error.to_string();
        assert!(message.contains("Namespace"));
        assert!(message.contains("cluster-scoped"));
        assert!(message.contains("namespaced"));
    }

    #[test]
    fn test_error_display() {
        // Test that all error variants implement Display correctly
        let errors: Vec<Error> = vec![
            Error::Protocol("test".to_string()),
            Error::Config("test config".to_string()),
            Error::Tool("test tool".to_string()),
            Error::ReadOnlyMode,
            Error::NoContext,
            Error::NoClusterConnection,
            Error::VersionMismatch {
                current: "v1".to_string(),
                required: "v2".to_string(),
            },
            Error::FeatureNotAvailable {
                feature: "test".to_string(),
                required_major: 1,
                required_minor: 30,
                current_major: 1,
                current_minor: 28,
            },
            Error::ResourceScopeMismatch {
                kind: "Test".to_string(),
                expected_scope: "cluster".to_string(),
                actual_scope: "namespace".to_string(),
            },
        ];

        for error in errors {
            // Just ensure to_string() doesn't panic
            let _ = error.to_string();
        }
    }

    #[test]
    fn test_json_rpc_code_for_non_json_rpc_errors() {
        // Non-JSON-RPC errors should return internal error code
        assert_eq!(Error::NoContext.json_rpc_code(), -32603);
        assert_eq!(Error::ReadOnlyMode.json_rpc_code(), -32603);
        assert_eq!(
            Error::VersionMismatch {
                current: "v1".to_string(),
                required: "v2".to_string()
            }
            .json_rpc_code(),
            -32603
        );
    }

    #[test]
    fn test_is_json_rpc_error() {
        assert!(Error::json_rpc_invalid_params("test").is_json_rpc_error());
        assert!(!Error::NoContext.is_json_rpc_error());
        assert!(!Error::ReadOnlyMode.is_json_rpc_error());
    }
}

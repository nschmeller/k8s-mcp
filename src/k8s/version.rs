//! Kubernetes version detection.
//!
//! This module provides version detection and comparison functionality
//! for Kubernetes clusters.
//!
//! # Example
//!
//! ```no_run
//! use k8s_mcp::k8s::version::K8sVersion;
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

use crate::error::{Error, Result};
use kube::Client;
use serde::Deserialize;
use std::fmt;
use tracing::info;

/// Kubernetes version information.
///
/// Contains detailed version information about a Kubernetes cluster
/// obtained from the `/version` endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct K8sVersion {
    /// Major version number (e.g., 1 for v1.32.0)
    pub major: u32,
    /// Minor version number (e.g., 32 for v1.32.0)
    pub minor: u32,
    /// Full git version string (e.g., "v1.32.0+abc123")
    pub git_version: String,
    /// Platform identifier (e.g., "linux/amd64")
    pub platform: String,
    /// Git commit hash
    pub git_commit: String,
    /// Build date
    pub build_date: String,
    /// Go version used to build Kubernetes
    pub go_version: String,
    /// Compiler used
    pub compiler: String,
}

impl K8sVersion {
    /// Detect the Kubernetes version from a connected client.
    ///
    /// Queries the `/version` endpoint to retrieve cluster version information.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use k8s_mcp::k8s::version::K8sVersion;
    /// use kube::Client;
    ///
    /// async fn detect_version(client: &Client) {
    ///     let version = K8sVersion::detect(client).await.unwrap();
    ///     println!("Cluster version: {}", version);
    /// }
    /// ```
    pub async fn detect(client: &Client) -> Result<Self> {
        info!("Detecting Kubernetes version...");

        let request = http::Request::builder()
            .uri("/version")
            .body(vec![])
            .map_err(|e| Error::Protocol(format!("Failed to build version request: {}", e)))?;

        let version_info: VersionInfo = client
            .request(request)
            .await
            .map_err(|e| Error::Protocol(format!("Failed to get Kubernetes version: {}", e)))?;

        let version = K8sVersion::from_version_info(&version_info)?;
        info!("Detected Kubernetes version: {}", version.git_version);

        Ok(version)
    }

    /// Parse version from the raw VersionInfo response.
    fn from_version_info(info: &VersionInfo) -> Result<Self> {
        let major = info
            .major
            .parse::<u32>()
            .map_err(|e| Error::Protocol(format!("Invalid major version: {}", e)))?;

        let minor = info
            .minor
            .parse::<u32>()
            .map_err(|e| Error::Protocol(format!("Invalid minor version: {}", e)))?;

        Ok(K8sVersion {
            major,
            minor,
            git_version: info.git_version.clone(),
            platform: info.platform.clone(),
            git_commit: info.git_commit.clone(),
            build_date: info.build_date.clone(),
            go_version: info.go_version.clone(),
            compiler: info.compiler.clone(),
        })
    }

    /// Check if the cluster version is at least the specified version.
    ///
    /// # Example
    ///
    /// ```
    /// use k8s_mcp::k8s::version::K8sVersion;
    ///
    /// let version = K8sVersion {
    ///     major: 1,
    ///     minor: 32,
    ///     git_version: "v1.32.0".to_string(),
    ///     platform: "linux/amd64".to_string(),
    ///     git_commit: String::new(),
    ///     build_date: String::new(),
    ///     go_version: String::new(),
    ///     compiler: String::new(),
    /// };
    ///
    /// assert!(version.is_at_least(1, 30));
    /// assert!(version.is_at_least(1, 32));
    /// assert!(!version.is_at_least(1, 33));
    /// ```
    pub fn is_at_least(&self, major: u32, minor: u32) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }

    /// Check if the cluster version is less than the specified version.
    pub fn is_less_than(&self, major: u32, minor: u32) -> bool {
        self.major < major || (self.major == major && self.minor < minor)
    }

    /// Get a short version string (e.g., "1.32").
    pub fn short_version(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Parse git version string to extract major and minor.
    ///
    /// Handles formats like:
    /// - "v1.32.0" -> (1, 32)
    /// - "v1.32.0+abc123" -> (1, 32)
    /// - "1.32.0" -> (1, 32)
    pub fn parse_git_version(git_version: &str) -> Option<(u32, u32)> {
        let version = git_version.trim_start_matches('v');

        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let major = parts[0].parse::<u32>().ok()?;
        // Handle minor version that might have extra suffixes like "32+abc123"
        let minor_str = parts[1].split('+').next().unwrap_or(parts[1]);
        let minor = minor_str.parse::<u32>().ok()?;

        Some((major, minor))
    }
}

impl fmt::Display for K8sVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.git_version)
    }
}

/// Raw version info from Kubernetes /version endpoint.
#[derive(Debug, Clone, Deserialize)]
struct VersionInfo {
    major: String,
    minor: String,
    git_version: String,
    git_commit: String,
    build_date: String,
    go_version: String,
    compiler: String,
    platform: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_version(major: u32, minor: u32) -> K8sVersion {
        K8sVersion {
            major,
            minor,
            git_version: format!("v{}.{}.0", major, minor),
            platform: "linux/amd64".to_string(),
            git_commit: "abc123".to_string(),
            build_date: "2024-01-01T00:00:00Z".to_string(),
            go_version: "go1.21.0".to_string(),
            compiler: "gc".to_string(),
        }
    }

    #[test]
    fn test_is_at_least() {
        let v = test_version(1, 32);

        // Same major, lower minor
        assert!(v.is_at_least(1, 30));
        assert!(v.is_at_least(1, 31));
        assert!(v.is_at_least(1, 32));

        // Same major, higher minor
        assert!(!v.is_at_least(1, 33));
        assert!(!v.is_at_least(1, 34));

        // Lower major
        assert!(v.is_at_least(0, 99));

        // Higher major
        assert!(!v.is_at_least(2, 0));
    }

    #[test]
    fn test_is_at_least_boundary_cases() {
        // Test version 1.0
        let v1_0 = test_version(1, 0);
        assert!(v1_0.is_at_least(1, 0));
        assert!(!v1_0.is_at_least(1, 1));
        assert!(v1_0.is_at_least(0, 999));

        // Test version 2.0
        let v2_0 = test_version(2, 0);
        assert!(v2_0.is_at_least(1, 99));
        assert!(v2_0.is_at_least(2, 0));
        assert!(!v2_0.is_at_least(2, 1));

        // Test high minor version
        let v1_99 = test_version(1, 99);
        assert!(v1_99.is_at_least(1, 98));
        assert!(v1_99.is_at_least(1, 99));
        assert!(!v1_99.is_at_least(2, 0));
    }

    #[test]
    fn test_is_less_than() {
        let v = test_version(1, 32);

        assert!(!v.is_less_than(1, 30));
        assert!(!v.is_less_than(1, 32));
        assert!(v.is_less_than(1, 33));
        assert!(v.is_less_than(2, 0));
    }

    #[test]
    fn test_is_less_than_boundary_cases() {
        let v1_0 = test_version(1, 0);
        assert!(!v1_0.is_less_than(1, 0));
        assert!(v1_0.is_less_than(1, 1));
        assert!(!v1_0.is_less_than(0, 99));

        let v2_5 = test_version(2, 5);
        assert!(v2_5.is_less_than(3, 0));
        assert!(!v2_5.is_less_than(2, 5));
        assert!(v2_5.is_less_than(2, 6));
    }

    #[test]
    fn test_short_version() {
        let v = test_version(1, 32);
        assert_eq!(v.short_version(), "1.32");

        let v2 = test_version(2, 0);
        assert_eq!(v2.short_version(), "2.0");

        let v3 = test_version(1, 0);
        assert_eq!(v3.short_version(), "1.0");
    }

    #[test]
    fn test_parse_git_version_standard_formats() {
        // Standard semver formats
        assert_eq!(K8sVersion::parse_git_version("v1.32.0"), Some((1, 32)));
        assert_eq!(K8sVersion::parse_git_version("v1.0.0"), Some((1, 0)));
        assert_eq!(K8sVersion::parse_git_version("v2.1.3"), Some((2, 1)));
        assert_eq!(K8sVersion::parse_git_version("1.32.0"), Some((1, 32)));
    }

    #[test]
    fn test_parse_git_version_with_build_metadata() {
        // With build metadata after +
        assert_eq!(
            K8sVersion::parse_git_version("v1.32.0+abc123"),
            Some((1, 32))
        );
        assert_eq!(
            K8sVersion::parse_git_version("v1.25.3+eks-123"),
            Some((1, 25))
        );
        assert_eq!(
            K8sVersion::parse_git_version("v1.30.0+gitabc"),
            Some((1, 30))
        );
    }

    #[test]
    fn test_parse_git_version_with_prerelease() {
        // With prerelease suffixes
        assert_eq!(
            K8sVersion::parse_git_version("v1.25.3-eks-123"),
            Some((1, 25))
        );
        assert_eq!(K8sVersion::parse_git_version("v1.28.0-rc.1"), Some((1, 28)));
        assert_eq!(
            K8sVersion::parse_git_version("v1.30.0-alpha"),
            Some((1, 30))
        );
    }

    #[test]
    fn test_parse_git_version_cloud_provider_formats() {
        // Cloud provider specific formats
        assert_eq!(
            K8sVersion::parse_git_version("v1.27.3-eks-abc123"),
            Some((1, 27))
        );
        assert_eq!(
            K8sVersion::parse_git_version("v1.28.4-gke.500"),
            Some((1, 28))
        );
        assert_eq!(
            K8sVersion::parse_git_version("v1.29.0-0.okd"),
            Some((1, 29))
        );
    }

    #[test]
    fn test_parse_git_version_invalid() {
        // Invalid cases
        assert_eq!(K8sVersion::parse_git_version("invalid"), None);
        assert_eq!(K8sVersion::parse_git_version("v1"), None);
        assert_eq!(K8sVersion::parse_git_version(""), None);
        assert_eq!(K8sVersion::parse_git_version("v"), None);
        assert_eq!(K8sVersion::parse_git_version("abc.def.ghi"), None);
        assert_eq!(K8sVersion::parse_git_version("vX.Y.Z"), None);
    }

    #[test]
    fn test_display() {
        let v = test_version(1, 32);
        assert_eq!(format!("{}", v), "v1.32.0");

        let v2 = test_version(2, 0);
        assert_eq!(format!("{}", v2), "v2.0.0");
    }

    #[test]
    fn test_version_equality() {
        let v1 = test_version(1, 32);
        let v2 = test_version(1, 32);
        let v3 = test_version(1, 33);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_version_clone() {
        let v1 = test_version(1, 32);
        let v2 = v1.clone();

        assert_eq!(v1, v2);
        assert_eq!(v1.major, v2.major);
        assert_eq!(v1.minor, v2.minor);
        assert_eq!(v1.git_version, v2.git_version);
    }

    #[test]
    fn test_version_debug() {
        let v = test_version(1, 32);
        let debug_str = format!("{:?}", v);

        assert!(debug_str.contains("major: 1"));
        assert!(debug_str.contains("minor: 32"));
        assert!(debug_str.contains("v1.32.0"));
    }

    #[test]
    fn test_from_version_info() {
        let info = VersionInfo {
            major: "1".to_string(),
            minor: "32".to_string(),
            git_version: "v1.32.0".to_string(),
            git_commit: "abc123".to_string(),
            build_date: "2024-01-01T00:00:00Z".to_string(),
            go_version: "go1.21.0".to_string(),
            compiler: "gc".to_string(),
            platform: "linux/amd64".to_string(),
        };

        let version = K8sVersion::from_version_info(&info).unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 32);
        assert_eq!(version.git_version, "v1.32.0");
        assert_eq!(version.platform, "linux/amd64");
        assert_eq!(version.git_commit, "abc123");
        assert_eq!(version.go_version, "go1.21.0");
    }

    #[test]
    fn test_from_version_info_various_versions() {
        // Test various version combinations
        let test_cases = [
            ("1", "25", 1, 25),
            ("1", "30", 1, 30),
            ("2", "0", 2, 0),
            ("1", "0", 1, 0),
        ];

        for (major, minor, expected_major, expected_minor) in test_cases {
            let info = VersionInfo {
                major: major.to_string(),
                minor: minor.to_string(),
                git_version: format!("v{}.{}.0", major, minor),
                git_commit: String::new(),
                build_date: String::new(),
                go_version: String::new(),
                compiler: String::new(),
                platform: String::new(),
            };

            let version = K8sVersion::from_version_info(&info).unwrap();
            assert_eq!(version.major, expected_major);
            assert_eq!(version.minor, expected_minor);
        }
    }

    #[test]
    fn test_from_version_info_invalid_major() {
        let info = VersionInfo {
            major: "invalid".to_string(),
            minor: "32".to_string(),
            git_version: "v1.32.0".to_string(),
            git_commit: String::new(),
            build_date: String::new(),
            go_version: String::new(),
            compiler: String::new(),
            platform: String::new(),
        };

        assert!(K8sVersion::from_version_info(&info).is_err());
    }

    #[test]
    fn test_from_version_info_invalid_minor() {
        let info = VersionInfo {
            major: "1".to_string(),
            minor: "invalid".to_string(),
            git_version: "v1.32.0".to_string(),
            git_commit: String::new(),
            build_date: String::new(),
            go_version: String::new(),
            compiler: String::new(),
            platform: String::new(),
        };

        assert!(K8sVersion::from_version_info(&info).is_err());
    }

    #[test]
    fn test_from_version_info_empty_strings() {
        let info = VersionInfo {
            major: "".to_string(),
            minor: "32".to_string(),
            git_version: "v1.32.0".to_string(),
            git_commit: String::new(),
            build_date: String::new(),
            go_version: String::new(),
            compiler: String::new(),
            platform: String::new(),
        };

        assert!(K8sVersion::from_version_info(&info).is_err());
    }

    #[test]
    fn test_version_comparison_chaining() {
        // Test that comparison methods work correctly together
        let v = test_version(1, 28);

        // v1.28 should satisfy requirements for 1.25+
        assert!(v.is_at_least(1, 25));

        // But not for 1.30+
        assert!(!v.is_at_least(1, 30));

        // It should be less than 1.30
        assert!(v.is_less_than(1, 30));

        // But not less than 1.28 or 1.27
        assert!(!v.is_less_than(1, 28));
        assert!(!v.is_less_than(1, 27));
    }
}

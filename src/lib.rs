//! Kubernetes MCP Server - Feature complete as kubectl.
//!
//! This library provides a Model Context Protocol (MCP) server for Kubernetes
//! that enables AI assistants to interact with Kubernetes clusters through
//! a standardized protocol.
//!
//! # Features
//!
//! - Full MCP protocol support (stdio transport)
//! - Read-only mode by default for safety
//! - Dynamic API discovery
//! - Comprehensive output formatting (table, json, yaml)
//!
//! # Example
//!
//! ```no_run
//! use k8s_mcp::{k8s::K8sClient, mcp::run_server, tools::ToolRegistry};
//!
//! #[tokio::main]
//! async fn main() -> k8s_mcp::Result<()> {
//!     let client = K8sClient::new(&Default::default()).await?;
//!     let registry = ToolRegistry::with_client(client);
//!     run_server(registry, false).await
//! }
//! ```

pub mod error;
pub mod format;
pub mod k8s;
pub mod mcp;
pub mod tools;

pub use error::{Error, Result};
pub use mcp::{run_server, McpServer};
pub use tools::ToolRegistry;

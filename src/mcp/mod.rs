//! MCP (Model Context Protocol) implementation.
//!
//! This module provides the MCP server implementation for the Kubernetes MCP server.

pub mod protocol;
pub mod server;
pub mod transport;

pub use protocol::*;
pub use server::{run_server, McpServer};
pub use transport::{StdioTransport, SyncStdioTransport, Transport};

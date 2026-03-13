#![doc = include_str!("../README.md")]

pub mod error;
pub mod format;
pub mod k8s;
pub mod mcp;
pub mod tools;

pub use error::{Error, Result};
pub use mcp::{run_server, McpServer};
pub use tools::ToolRegistry;

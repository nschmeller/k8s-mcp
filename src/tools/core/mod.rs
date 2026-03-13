//! Core CRUD tools for Kubernetes resources.

pub mod delete;
pub mod get;
pub mod list;

pub use delete::{DeleteResourceTool, DeploymentsDeleteTool, NamespacesDeleteTool, PodsDeleteTool};
pub use get::{
    DeploymentsGetTool, GetResourceTool, NamespacesGetTool, NodesGetTool, PodsGetTool,
    ServicesGetTool,
};
pub use list::{
    DeploymentsListTool, ListResourcesTool, NamespacesListTool, NodesListTool, PodsListTool,
    ServicesListTool,
};

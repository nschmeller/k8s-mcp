//! Kubernetes MCP Server - Feature complete as kubectl.
//!
//! This is the main entry point for the k8s-mcp server.

use clap::Parser;
use k8s_mcp::{
    k8s::{ApiDiscovery, K8sClient, K8sConfig},
    mcp::run_server,
    tools::register_all_tools,
    ToolRegistry,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Kubernetes MCP Server - Feature complete as kubectl.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable read-write mode (allows mutations)
    #[arg(short, long, env = "K8S_MCP_READ_WRITE")]
    read_write: bool,

    /// Path to kubeconfig file
    #[arg(short, long, env = "KUBECONFIG")]
    kubeconfig: Option<String>,

    /// Kubernetes context to use
    #[arg(short, long, env = "K8S_CONTEXT")]
    context: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, env = "K8S_MCP_LOG_LEVEL", default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize logging
    let level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_writer(std::io::stderr) // Write logs to stderr, stdout is for MCP
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting k8s-mcp server (read_write={})", args.read_write);

    // Build Kubernetes config
    let mut config = K8sConfig::new();
    if let Some(kubeconfig) = args.kubeconfig {
        config = config.with_kubeconfig(kubeconfig);
    }
    if let Some(context) = args.context {
        config = config.with_context(context);
    }

    // Initialize Kubernetes client
    let client = K8sClient::new(&config).await?;

    // Initialize API discovery
    let discovery = Arc::new(RwLock::new(ApiDiscovery::new()));

    // Create tool registry and register all tools
    let mut registry = ToolRegistry::new();
    register_all_tools(&mut registry, client, config, discovery);

    info!("Registered {} tools", registry.len());

    // Run the MCP server
    run_server(registry, args.read_write).await?;

    Ok(())
}

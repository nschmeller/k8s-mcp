//! MCP Server implementation.

use crate::error::{Error, Result};
use crate::mcp::protocol::*;
use crate::mcp::transport::Transport;
use crate::tools::registry::ToolRegistry;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// MCP Server state
pub struct McpServer {
    tool_registry: Arc<RwLock<ToolRegistry>>,
    server_info: Implementation,
    read_write: bool,
    initialized: bool,
}

impl McpServer {
    /// Create a new MCP server.
    pub fn new(tool_registry: ToolRegistry, read_write: bool) -> Self {
        McpServer {
            tool_registry: Arc::new(RwLock::new(tool_registry)),
            server_info: Implementation {
                name: "k8s-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            read_write,
            initialized: false,
        }
    }

    /// Handle an incoming JSON-RPC request.
    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> Option<String> {
        debug!(
            "Handling request: method={}, id={:?}",
            request.method, request.id
        );

        // Clone the id before the match since we need it later
        let request_id = request.id.clone();
        let method = request.method.clone();

        let response = match method.as_str() {
            // Lifecycle methods
            "initialize" => self.handle_initialize(request).await,
            "notifications/initialized" => {
                self.initialized = true;
                info!("Client initialized");
                return None; // Notification, no response
            }

            // Ping
            "ping" => self.handle_ping(request).await,

            // Tools
            "tools/list" => self.handle_tools_list(request).await,
            "tools/call" => self.handle_tools_call(request).await,

            // Resources (not implemented yet)
            "resources/list" => self.handle_resources_list(request).await,
            "resources/read" => self.handle_resources_read(request).await,

            // Prompts (not implemented yet)
            "prompts/list" => self.handle_prompts_list(request).await,

            // Logging
            "logging/setLevel" => self.handle_set_level(request).await,

            // Unknown method
            _ => {
                warn!("Unknown method: {}", method);
                Err(Error::json_rpc_method_not_found(&method))
            }
        };

        match response {
            Ok(result) => {
                let response = JsonRpcResponse::new(request_id, result);
                Some(serde_json::to_string(&response).unwrap_or_else(|e| {
                    error!("Failed to serialize response: {}", e);
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#.to_string()
                }))
            }
            Err(e) => {
                let error_response =
                    JsonRpcError::new(request_id, e.json_rpc_code(), e.to_string(), None);
                Some(serde_json::to_string(&error_response).unwrap_or_else(|e| {
                    error!("Failed to serialize error response: {}", e);
                    r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"Internal error"}}"#.to_string()
                }))
            }
        }
    }

    /// Handle initialize request.
    async fn handle_initialize(&mut self, request: JsonRpcRequest) -> Result<serde_json::Value> {
        let params: InitializeParams = request
            .params
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_else(|| InitializeParams {
                protocol_version: "2024-11-05".to_string(),
                capabilities: ClientCapabilities::default(),
                client_info: None,
            });

        info!("Initialize request from client: {:?}", params.client_info);

        // Validate protocol version
        if params.protocol_version != "2024-11-05" {
            warn!(
                "Client requested protocol version {}, we support 2024-11-05",
                params.protocol_version
            );
        }

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(false),
                }),
                resources: Some(ResourcesCapability {
                    subscribe: Some(false),
                    list_changed: Some(false),
                }),
                logging: Some(()),
                ..Default::default()
            },
            server_info: self.server_info.clone(),
            instructions: Some(format!(
                "Kubernetes MCP Server - Read-only mode: {}. Use tools to interact with your Kubernetes cluster.",
                if self.read_write {
                    "disabled"
                } else {
                    "enabled"
                }
            )),
        };

        Ok(serde_json::to_value(result)?)
    }

    /// Handle ping request.
    async fn handle_ping(&mut self, _request: JsonRpcRequest) -> Result<serde_json::Value> {
        debug!("Ping received");
        Ok(json!({}))
    }

    /// Handle tools/list request.
    async fn handle_tools_list(&mut self, _request: JsonRpcRequest) -> Result<serde_json::Value> {
        debug!("Tools list requested");
        let registry = self.tool_registry.read().await;
        let tools = registry.list_tools();
        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };
        Ok(serde_json::to_value(result)?)
    }

    /// Handle tools/call request.
    async fn handle_tools_call(&mut self, request: JsonRpcRequest) -> Result<serde_json::Value> {
        let params_value = request
            .params
            .ok_or_else(|| Error::json_rpc_invalid_params("Missing params"))?;

        let params: CallToolParams = serde_json::from_value(params_value)
            .map_err(|e: serde_json::Error| Error::json_rpc_invalid_params(e.to_string()))?;

        debug!("Tool call: {}", params.name);

        // Check if tool requires write access
        let registry = self.tool_registry.read().await;
        if registry.is_write_tool(&params.name) && !self.read_write {
            return Ok(serde_json::to_value(CallToolResult::error(
                "Operation not permitted in read-only mode. Start with --read-write to enable mutations.",
            ))?);
        }

        let result = registry
            .call_tool(&params.name, params.arguments.unwrap_or_default())
            .await?;

        Ok(serde_json::to_value(result)?)
    }

    /// Handle resources/list request.
    async fn handle_resources_list(
        &mut self,
        _request: JsonRpcRequest,
    ) -> Result<serde_json::Value> {
        debug!("Resources list requested");
        let result = ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        };
        Ok(serde_json::to_value(result)?)
    }

    /// Handle resources/read request.
    async fn handle_resources_read(
        &mut self,
        _request: JsonRpcRequest,
    ) -> Result<serde_json::Value> {
        Err(Error::json_rpc_method_not_found("resources/read"))
    }

    /// Handle prompts/list request.
    async fn handle_prompts_list(&mut self, _request: JsonRpcRequest) -> Result<serde_json::Value> {
        debug!("Prompts list requested");
        Ok(json!({"prompts": []}))
    }

    /// Handle logging/setLevel request.
    async fn handle_set_level(&mut self, request: JsonRpcRequest) -> Result<serde_json::Value> {
        let params_value = request
            .params
            .ok_or_else(|| Error::json_rpc_invalid_params("Missing params"))?;

        let params: SetLevelParams = serde_json::from_value(params_value)
            .map_err(|e: serde_json::Error| Error::json_rpc_invalid_params(e.to_string()))?;

        info!("Logging level set to: {:?}", params.level);
        Ok(json!({}))
    }

    /// Run the server with the given transport.
    pub async fn run<T: Transport>(&mut self, mut transport: T) -> Result<()> {
        info!("Starting MCP server");

        loop {
            match transport.receive().await? {
                Some(message) => {
                    // Parse the request
                    let request: JsonRpcRequest = match serde_json::from_str(&message) {
                        Ok(r) => r,
                        Err(e) => {
                            error!("Failed to parse request: {}", e);
                            let error = JsonRpcError::new(
                                None,
                                -32700,
                                format!("Parse error: {}", e),
                                None,
                            );
                            transport.send(&serde_json::to_string(&error)?).await?;
                            continue;
                        }
                    };

                    // Handle the request
                    if let Some(response) = self.handle_request(request).await {
                        transport.send(&response).await?;
                    }
                }
                None => {
                    // EOF, client disconnected
                    info!("Client disconnected");
                    break;
                }
            }
        }

        Ok(())
    }
}

/// Run the MCP server with stdio transport.
pub async fn run_server(tool_registry: ToolRegistry, read_write: bool) -> Result<()> {
    let transport = crate::mcp::transport::StdioTransport::new();
    let mut server = McpServer::new(tool_registry, read_write);
    server.run(transport).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolRegistry;

    /// Create a basic server for testing.
    fn create_test_server() -> McpServer {
        let registry = ToolRegistry::new();
        McpServer::new(registry, false)
    }

    /// Create a server with write mode enabled.
    #[allow(dead_code)]
    fn create_test_server_write() -> McpServer {
        let registry = ToolRegistry::new();
        McpServer::new(registry, true)
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocol_version": "2024-11-05",
                "capabilities": {}
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(response_json["jsonrpc"], "2.0");
        assert_eq!(response_json["id"], 1);
        assert!(response_json["result"]["capabilities"]["tools"].is_object());
    }

    #[tokio::test]
    async fn test_handle_initialize_with_client_info() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocol_version": "2024-11-05",
                "capabilities": {},
                "client_info": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["server_info"]["name"].is_string());
    }

    #[tokio::test]
    async fn test_handle_ping() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "ping".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(response_json["result"], json!({}));
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["tools"].is_array());
    }

    #[tokio::test]
    async fn test_handle_tools_call_unknown_tool() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "unknown_tool",
                "arguments": {}
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["error"].is_object());
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["error"].is_object());
        assert_eq!(response_json["error"]["code"], -32601); // Method not found
    }

    #[tokio::test]
    async fn test_handle_resources_list() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "resources/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["resources"].is_array());
        assert!(
            response_json["result"]["resources"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_handle_resources_read_not_implemented() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "resources/read".to_string(),
            params: Some(json!({"uri": "test://resource"})),
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["error"].is_object());
    }

    #[tokio::test]
    async fn test_handle_prompts_list() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "prompts/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["result"]["prompts"].is_array());
    }

    #[tokio::test]
    async fn test_handle_logging_set_level() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "logging/setLevel".to_string(),
            params: Some(json!({
                "level": "debug"
            })),
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(response_json["result"], json!({}));
    }

    #[tokio::test]
    async fn test_handle_request_with_string_id() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::String("test-id-123".to_string())),
            method: "ping".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert_eq!(response_json["id"], "test-id-123");
    }

    #[tokio::test]
    async fn test_handle_request_missing_params_for_tools_call() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "tools/call".to_string(),
            params: None, // Missing params
        };

        let response = server.handle_request(request).await;
        assert!(response.is_some());

        let response_json: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
        assert!(response_json["error"].is_object());
        assert_eq!(response_json["error"]["code"], -32602); // Invalid params
    }

    #[tokio::test]
    async fn test_server_info_in_response() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocol_version": "2024-11-05",
                "capabilities": {}
            })),
        };

        let response = server.handle_request(request).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();

        assert_eq!(response_json["result"]["server_info"]["name"], "k8s-mcp");
        assert!(response_json["result"]["server_info"]["version"].is_string());
    }

    #[tokio::test]
    async fn test_instructions_in_response() {
        let mut server = create_test_server();

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(RequestId::Number(1)),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocol_version": "2024-11-05",
                "capabilities": {}
            })),
        };

        let response = server.handle_request(request).await.unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response).unwrap();

        assert!(response_json["result"]["instructions"].is_string());
        // Should mention read-only mode
        assert!(
            response_json["result"]["instructions"]
                .as_str()
                .unwrap()
                .contains("Read-only")
        );
    }
}

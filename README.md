# k8s-mcp

A Kubernetes MCP (Model Context Protocol) server for Claude Code. This server provides Claude with tools to interact with Kubernetes clusters, similar to kubectl.

## Features

- **Read-only by default**: Safe to use in production environments
- **Read-write mode**: Optional flag to enable mutations (create, update, delete)
- **Multiple output formats**: Table, JSON, and YAML output
- **Context management**: List and switch between Kubernetes contexts
- **Resource operations**: Get, list, and delete Kubernetes resources
- **Metrics support**: View resource consumption with `top` commands
- **Log streaming**: Fetch logs from pods and containers

## Installation

### From Source

```bash
git clone https://github.com/nschmeller/k8s-mcp.git
cd k8s-mcp
cargo build --release
```

The binary will be at `target/release/k8s-mcp`.

### Prerequisites

- Rust 1.81 or later
- A valid kubeconfig file (default location: `~/.kube/config`)
- Access to a Kubernetes cluster

## Usage

### Command Line Options

```
k8s-mcp [OPTIONS]

Options:
  -r, --read-write               Enable read-write mode (allows mutations)
  -k, --kubeconfig <KUBECONFIG>  Path to kubeconfig file
  -c, --context <CONTEXT>        Kubernetes context to use
  -l, --log-level <LOG_LEVEL>    Log level (trace, debug, info, warn, error) [default: info]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `K8S_MCP_READ_WRITE` | Set to `true` to enable read-write mode |
| `KUBECONFIG` | Path to kubeconfig file |
| `K8S_CONTEXT` | Kubernetes context to use |
| `K8S_MCP_LOG_LEVEL` | Log level (default: `info`) |

## Configuring with Claude Code

### Option 1: Add as a stdio MCP server (Recommended)

Add k8s-mcp to Claude Code using the CLI:

```bash
# Read-only mode (default, safe for production)
claude mcp add --transport stdio k8s-mcp -- /path/to/k8s-mcp

# With a specific kubeconfig
claude mcp add --transport stdio --env KUBECONFIG=/path/to/kubeconfig k8s-mcp -- /path/to/k8s-mcp

# With read-write mode enabled
claude mcp add --transport stdio --env K8S_MCP_READ_WRITE=true k8s-mcp -- /path/to/k8s-mcp

# With a specific Kubernetes context
claude mcp add --transport stdio --env K8S_CONTEXT=my-context k8s-mcp -- /path/to/k8s-mcp
```

Replace `/path/to/k8s-mcp` with the actual path to the binary. For example:

```bash
# If built from source in ~/projects/k8s-mcp
claude mcp add --transport stdio k8s-mcp -- ~/projects/k8s-mcp/target/release/k8s-mcp

# If installed system-wide
claude mcp add --transport stdio k8s-mcp -- /usr/local/bin/k8s-mcp
```

### Option 2: Configure via `.mcp.json` (Project-level)

Create a `.mcp.json` file in your project root:

```json
{
  "mcpServers": {
    "k8s-mcp": {
      "command": "/path/to/k8s-mcp",
      "args": [],
      "env": {
        "KUBECONFIG": "${HOME}/.kube/config"
      }
    }
  }
}
```

For read-write mode:

```json
{
  "mcpServers": {
    "k8s-mcp": {
      "command": "/path/to/k8s-mcp",
      "args": ["--read-write"],
      "env": {}
    }
  }
}
```

### Option 3: User-level configuration

Add to `~/.claude.json` under the `mcpServers` key:

```json
{
  "mcpServers": {
    "k8s-mcp": {
      "command": "/path/to/k8s-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

## Available Tools

### Resource Operations

| Tool | Description |
|------|-------------|
| `pods_list` | List pods in a namespace |
| `pods_get` | Get a specific pod |
| `pods_delete` | Delete a pod (write mode required) |
| `pods_log` | Get logs from a pod |
| `deployments_list` | List deployments |
| `deployments_get` | Get a specific deployment |
| `deployments_delete` | Delete a deployment (write mode required) |
| `services_list` | List services |
| `services_get` | Get a specific service |
| `nodes_list` | List nodes |
| `nodes_get` | Get a specific node |
| `namespaces_list` | List namespaces |
| `namespaces_get` | Get a specific namespace |
| `namespaces_delete` | Delete a namespace (write mode required) |
| `events_list` | List events |
| `resources_list` | List any resource type |
| `resources_get` | Get any resource by name |
| `resources_delete` | Delete any resource (write mode required) |

### Metrics

| Tool | Description |
|------|-------------|
| `top_nodes` | Show node resource usage |
| `top_pods` | Show pod resource usage |

### Context & Configuration

| Tool | Description |
|------|-------------|
| `contexts_list` | List available Kubernetes contexts |
| `context_current` | Get the current context |
| `configuration_view` | View kubeconfig contents |
| `api_resources_list` | List available API resources |
| `api_versions` | List available API versions |

## Example Usage in Claude Code

Once configured, you can ask Claude to interact with your Kubernetes cluster:

```
List all pods in the default namespace
```

```
Show me the logs from the nginx pod
```

```
What's the resource usage across all nodes?
```

```
Get the details of the deployment named "api-server" in the "production" namespace
```

```
List all events in the last hour
```

```
Switch to the production context and show me all deployments
```

## Security Considerations

### Read-Only Mode (Default)

By default, k8s-mcp runs in read-only mode. This prevents any mutations to your cluster:
- No resource creation
- No resource updates
- No resource deletions

This is safe for production environments and for users who only need to inspect cluster state.

### Read-Write Mode

When started with `--read-write` or `K8S_MCP_READ_WRITE=true`, the server allows mutations. Use with caution:
- Resources can be deleted
- ConfigMaps and Secrets can be modified
- Namespaces can be created and deleted

**Recommendation**: Only enable read-write mode in development environments or when you trust all users with cluster access.

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests (requires kind)
cargo test --test integration_tests

# All tests
cargo test
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## License

Apache-2.0
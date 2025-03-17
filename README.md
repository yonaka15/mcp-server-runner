# MCP Server Runner

> **Note**: This project is currently under active development and in WIP (Work In Progress) status. Features and APIs may change significantly.

A WebSocket server implementation for running [Model Context Protocol](https://github.com/modelcontextprotocol) (MCP) servers. This application enables MCP servers to be accessed via WebSocket connections, facilitating integration with web applications and other network-enabled clients.

## Development Status

- 🚧 **Work In Progress**: This software is in active development
- ⚠️ **API Stability**: APIs and features may change without notice
- 🧪 **Testing**: Currently undergoing testing and refinement
- 📝 **Documentation**: Documentation is being actively updated

## Overview

MCP Server Runner acts as a bridge between WebSocket clients and MCP server implementations. It:

- Launches an MCP server process
- Manages WebSocket connections
- Handles bidirectional communication between clients and the MCP server
- Supports graceful shutdown and error handling

## Features

- WebSocket server implementation with single-client support
- Process management for MCP server instances
- Bidirectional message passing between client and server
- Graceful shutdown handling
- Comprehensive error logging
- Cross-platform support (Unix/Windows)

## Prerequisites

- Rust 1.70 or higher
- An MCP server implementation executable

## Configuration

### Environment Variables

The application can be configured through environment variables:

```env
PROGRAM=        # Path to the MCP server executable (required if no config file)
ARGS=           # Comma-separated list of arguments for the MCP server
HOST=0.0.0.0    # Host address to bind to (default: 0.0.0.0)
PORT=8080       # Port to listen on (default: 8080)
CONFIG_FILE=    # Path to JSON configuration file
```

Additional environment variables will be passed through to the MCP server process.

### JSON Configuration

Alternatively, you can provide a JSON configuration file:

```json
{
  "servers": {
    "filesystem": {
      "command": "npx",
      "args": [
        "-y",
        "@modelcontextprotocol/server-filesystem",
        "/path/to/workspace"
      ]
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "your_token_here"
      }
    }
  },
  "default_server": "filesystem",
  "host": "0.0.0.0",
  "port": 8080
}
```

You can specify the configuration file in two ways:

1. As a command-line argument: `mcp-server-runner config.json`
2. Using the `CONFIG_FILE` environment variable: `CONFIG_FILE=config.json mcp-server-runner`

The JSON configuration allows you to define multiple server configurations and select one as the default.

### Configuration Priority

1. Command-line specified config file
2. `CONFIG_FILE` environment variable
3. Environment variables (`PROGRAM`, `ARGS`, etc.)
4. Default values

## Usage

1. Using environment variables:

   ```bash
   export PROGRAM=npx
   export ARGS=-y,@modelcontextprotocol/server-github
   export PORT=8080
   export GITHUB_PERSONAL_ACCESS_TOKEN=github_pat_***
   cargo run
   ```

2. Using a configuration file:

   ```bash
   # Either specify the config file as an argument
   cargo run config.json

   # Or use the CONFIG_FILE environment variable
   CONFIG_FILE=config.json cargo run
   ```

3. Connect to the WebSocket server:
   ```javascript
   const ws = new WebSocket("ws://localhost:8080");
   ```

## Docker Support

A Dockerfile and docker-compose.yml are provided for containerized deployment:

```bash
docker-compose up --build
```

## Development

Build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run with debug logging:

```bash
RUST_LOG=debug cargo run
```

## Architecture

The application follows a modular architecture:

- `main.rs`: Application entry point and server setup
- `config/`: Configuration loading and management
- `process/`: Process management and I/O handling
- `websocket/`: WebSocket connection management
- `state.rs`: Global state management
- `shutdown.rs`: Graceful shutdown handling

## Error Handling

- Standard error output from the MCP server is logged but not forwarded to clients
- WebSocket connection errors are handled gracefully
- Process errors are logged with detailed information

## Limitations

- Supports only one client connection at a time
- Does not support WebSocket SSL/TLS (use a reverse proxy for secure connections)
- No built-in authentication mechanism

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Additional Resources

- [Model Context Protocol Specification](https://github.com/modelcontextprotocol/specification)
- [WebSocket Protocol (RFC 6455)](https://tools.ietf.org/html/rfc6455)

# MCP Server Runner

A WebSocket server implementation for running Model Context Protocol (MCP) servers. This application enables MCP servers to be accessed via WebSocket connections, facilitating integration with web applications and other network-enabled clients.

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

The application is configured through environment variables:

```env
PROGRAM=        # Path to the MCP server executable (required)
ARGS=           # Comma-separated list of arguments for the MCP server
HOST=0.0.0.0    # Host address to bind to (default: 0.0.0.0)
PORT=8080       # Port to listen on (default: 8080)
```

Additional environment variables will be passed through to the MCP server process.

## Usage

1. Set up the environment variables:

   ```bash
   export PROGRAM=/path/to/mcp-server
   export ARGS=arg1,arg2
   export PORT=8080
   ```

2. Run the server:

   ```bash
   cargo run
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


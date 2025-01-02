# RustOpus API Gateway

A high-performance API Gateway written in Rust, inspired by KrakenD.

## Features

- High-performance HTTP routing and proxying
- Backend service discovery and load balancing
- Request/response transformation pipeline
- Plugin system for extensibility
- Metrics and telemetry
- Circuit breaker pattern
- Rate limiting
- Health checks

## Requirements

- Rust 1.70 or later
- Cargo

## Installation

Clone the repository and build:

```bash
git clone https://github.com/raultoto/rustopus.git
cd rustopus
cargo build --release
```

## Configuration

Create a `config.yaml` file (see example in repository) or set the `CONFIG_PATH` environment variable to point to your configuration file.

Example configuration:

```yaml
server:
  listen_addr: "127.0.0.1:8080"
  workers: 4

proxy:
  connection_timeout: "30s"
  request_timeout: "30s"
  max_connections: 1000
  enable_http2: true

backends:
  - name: "api1"
    url: "http://localhost:3000"
    weight: 1
    timeout: "5s"
    retry_attempts: 3
    health_check:
      enabled: true
      interval: "10s"
      path: "/health"
      timeout: "2s"
```

## Running

```bash
# Using default config.yaml
cargo run --release

# Using custom config path
CONFIG_PATH=/path/to/config.yaml cargo run --release
```

## Architecture

RustOpus follows a hexagonal architecture pattern with the following components:

- Router: HTTP request routing and middleware pipeline
- Proxy: Backend service communication
- Pipeline: Request/response transformation
- Plugins: Extensible functionality
- Metrics: Performance monitoring
- Telemetry: Distributed tracing

## Performance

RustOpus is designed for high performance:

- Async I/O with Tokio
- Zero-copy where possible
- Connection pooling
- Minimal allocations
- Efficient routing

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test
```

### Adding a New Plugin

1. Implement the `Plugin` trait
2. Register the plugin in the `PluginManager`
3. Add configuration in `config.yaml`

Example:

```rust
use async_trait::async_trait;

#[async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    async fn on_request(&self, request: Request<Body>) -> Result<Request<Body>> {
        // Transform request
        Ok(request)
    }

    async fn on_response(&self, response: Response<Body>) -> Result<Response<Body>> {
        // Transform response
        Ok(response)
    }
}
```


## License

This project is licensed under the MIT License - see the LICENSE file for details. 
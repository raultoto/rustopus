use super::Config;
use anyhow::Result;
use std::time::Duration;

pub fn validate_config(config: &Config) -> Result<()> {
    validate_server_config(&config.server)?;
    validate_logging_config(&config.logging)?;
    validate_metrics_config(&config.metrics)?;
    validate_security_config(&config.security)?;
    validate_plugins_config(&config.plugins)?;
    validate_endpoints_config(&config.endpoints)?;
    Ok(())
}

fn validate_server_config(config: &super::types::ServerConfig) -> Result<()> {
    if config.port == 0 {
        return Err(anyhow::anyhow!("Server port cannot be 0"));
    }

    if config.workers == 0 {
        return Err(anyhow::anyhow!("Number of workers cannot be 0"));
    }

    if config.timeout < Duration::from_secs(1) {
        return Err(anyhow::anyhow!("Server timeout must be at least 1 second"));
    }

    if config.max_request_size == 0 {
        return Err(anyhow::anyhow!("Max request size cannot be 0"));
    }

    Ok(())
}

fn validate_logging_config(config: &super::types::LoggingConfig) -> Result<()> {
    match config.level.to_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => {}
        _ => return Err(anyhow::anyhow!("Invalid log level")),
    }

    match config.format.to_lowercase().as_str() {
        "json" | "text" => {}
        _ => return Err(anyhow::anyhow!("Invalid log format")),
    }

    if let Some(path) = &config.file_output {
        if path.is_empty() {
            return Err(anyhow::anyhow!("Log file path cannot be empty"));
        }
    }

    Ok(())
}

fn validate_metrics_config(config: &super::types::MetricsConfig) -> Result<()> {
    if config.enabled && config.port == 0 {
        return Err(anyhow::anyhow!("Metrics port cannot be 0 when metrics are enabled"));
    }

    if config.path.is_empty() {
        return Err(anyhow::anyhow!("Metrics path cannot be empty"));
    }

    if !config.path.starts_with('/') {
        return Err(anyhow::anyhow!("Metrics path must start with '/'"));
    }

    Ok(())
}

fn validate_security_config(config: &super::types::SecurityConfig) -> Result<()> {
    if config.cors.enabled {
        if config.cors.allowed_origins.is_empty() {
            return Err(anyhow::anyhow!("CORS allowed origins cannot be empty when CORS is enabled"));
        }
        if config.cors.allowed_methods.is_empty() {
            return Err(anyhow::anyhow!("CORS allowed methods cannot be empty when CORS is enabled"));
        }
    }

    if config.rate_limit.enabled {
        if config.rate_limit.requests_per_second == 0 {
            return Err(anyhow::anyhow!("Rate limit requests per second cannot be 0"));
        }
    }

    if config.auth.enabled {
        if config.auth.jwt_secret.is_none() {
            return Err(anyhow::anyhow!("JWT secret must be provided when auth is enabled"));
        }
    }

    Ok(())
}

fn validate_plugins_config(config: &super::types::PluginsConfig) -> Result<()> {
    if config.enabled {
        if config.directory.is_none() {
            return Err(anyhow::anyhow!("Plugin directory must be specified when plugins are enabled"));
        }
    }
    Ok(())
}

fn validate_endpoints_config(endpoints: &[super::types::EndpointConfig]) -> Result<()> {
    for endpoint in endpoints {
        if endpoint.path.is_empty() {
            return Err(anyhow::anyhow!("Endpoint path cannot be empty"));
        }

        if !endpoint.path.starts_with('/') {
            return Err(anyhow::anyhow!("Endpoint path must start with '/'"));
        }

        if endpoint.backend.is_empty() {
            return Err(anyhow::anyhow!("Endpoint must have at least one backend"));
        }

        // Validate protocol compatibility
        for backend in &endpoint.backend {
            match (&endpoint.protocol, &backend.protocol) {
                (super::types::GatewayProtocol::Rest, super::types::BackendProtocol::WebSocket) => {
                    return Err(anyhow::anyhow!("REST gateway cannot proxy to WebSocket backend"));
                }
                (super::types::GatewayProtocol::WebSocket, super::types::BackendProtocol::Rest) => {
                    return Err(anyhow::anyhow!("WebSocket gateway cannot proxy to REST backend"));
                }
                _ => {}
            }

            if backend.url.is_empty() {
                return Err(anyhow::anyhow!("Backend URL cannot be empty"));
            }

            if let Some(circuit_breaker) = &backend.circuit_breaker {
                if circuit_breaker.threshold == 0 {
                    return Err(anyhow::anyhow!("Circuit breaker threshold cannot be 0"));
                }
                if circuit_breaker.min_requests == 0 {
                    return Err(anyhow::anyhow!("Circuit breaker minimum requests cannot be 0"));
                }
            }

            if let Some(retry) = &backend.retry {
                if retry.attempts == 0 {
                    return Err(anyhow::anyhow!("Retry attempts cannot be 0"));
                }
            }
        }

        // Validate guards if auth is required
        if endpoint.auth_required && endpoint.guards.is_empty() {
            return Err(anyhow::anyhow!("Auth required but no guards specified"));
        }
    }

    Ok(())
} 
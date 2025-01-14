use anyhow::Result;
use rustopus::{
    config::Config,
    core::Gateway,
    protocol::http::{HttpServer, HttpProtocol}
};
use tracing::{info, Level};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting RustOpus API Gateway...");

    // Load configuration
    let config = Config::load()?;
    
    // Print configuration details
    info!("Server configuration:");
    info!("  Host: {}", config.server.host);
    info!("  Port: {}", config.server.port);
    info!("  Workers: {}", config.server.workers);
    
    info!("Metrics configuration:");
    info!("  Enabled: {}", config.metrics.enabled);
    info!("  Port: {}", config.metrics.port);
    
    info!("Number of configured endpoints: {}", config.endpoints.len());

    // Create and start HTTP gateway
    let gateway = Gateway::new(
        "rustopus".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
        config.clone(),
    )?;

    let http: HttpServer = HttpServer::new(
        Arc::new(RwLock::new(HttpProtocol::new())),
        Arc::new(config),
    );
    
    // Start the gateway
    info!("Starting HTTP gateway.....");
    http.start().await?;

    Ok(())
} 
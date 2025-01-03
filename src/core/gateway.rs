use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{info, debug, error};
use crate::config::Config;
use crate::protocol::http::{
    HttpProtocol, HttpClient, HttpServer,
    middleware::{
        Middleware,
        LoggingMiddleware,
        MetricsMiddleware,
        AuthMiddleware,
        RateLimitMiddleware,
    },
};
use super::middleware::MiddlewareStack;
use super::routing::RouterRegistry;

pub struct Gateway {
    name: String,
    version: String,
    config: Arc<Config>,
    router_registry: Arc<RwLock<RouterRegistry>>,
    middleware_chain: Arc<RwLock<MiddlewareStack>>,
    http_protocol: Arc<RwLock<HttpProtocol>>,
}

impl Gateway {
    pub fn new(name: String, version: String, config: Config) -> Result<Self> {
        Ok(Self {
            name,
            version,
            config: Arc::new(config),
            router_registry: Arc::new(RwLock::new(RouterRegistry::new())),
            middleware_chain: Arc::new(RwLock::new(MiddlewareStack::new())),
            http_protocol: Arc::new(RwLock::new(HttpProtocol::new())),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    pub fn router_registry(&self) -> Arc<RwLock<RouterRegistry>> {
        self.router_registry.clone()
    }

    pub fn middleware_chain(&self) -> Arc<RwLock<MiddlewareStack>> {
        self.middleware_chain.clone()
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting gateway: {} v{}", self.name, self.version);

        // Initialize telemetry
        self.init_telemetry().await?;

        // Initialize security
        self.init_security().await?;

        // Initialize protocols
        self.init_protocols().await?;

        // Start protocol servers
        self.start_servers().await?;

        info!("Gateway started successfully");
        Ok(())
    }

    async fn init_telemetry(&self) -> Result<()> {
        debug!("Initializing telemetry");

        // Initialize metrics
        if self.config.metrics.enabled {
            let metrics_middleware = self.create_metrics_middleware();
            self.http_protocol.write().await.add_middleware(metrics_middleware);
        }

        // Initialize tracing
        if self.config.observability.tracing.enabled {
            let tracing_middleware = self.create_tracing_middleware();
            self.http_protocol.write().await.add_middleware(tracing_middleware);
        }

        Ok(())
    }

    async fn init_security(&self) -> Result<()> {
        debug!("Initializing security");

        // Initialize authentication
        if self.config.security.auth.enabled {
            let auth_middleware = self.create_auth_middleware();
            self.http_protocol.write().await.add_middleware(auth_middleware);
        }

        // Initialize rate limiting
        if self.config.security.rate_limit.enabled {
            let rate_limit_middleware = self.create_rate_limit_middleware();
            self.http_protocol.write().await.add_middleware(rate_limit_middleware);
        }

        Ok(())
    }

    async fn init_protocols(&self) -> Result<()> {
        debug!("Initializing protocols");

        let mut http = self.http_protocol.write().await;
        
        // Configure HTTP routes from config
        for endpoint in &self.config.endpoints {
            for backend in &endpoint.backend {
                let client = crate::protocol::http::HttpClient::new(vec![backend.clone()])?;
                http.router().add_route(&endpoint.path, endpoint.clone(), client)?;
            }
        }

        Ok(())
    }

    async fn start_servers(&self) -> Result<()> {
        debug!("Starting protocol servers");

        // Start HTTP server if configured
        if !self.config.endpoints.is_empty() {
            self.start_http_server().await?;
        }

        Ok(())
    }

    async fn start_http_server(&self) -> Result<()> {
        let server = HttpServer::new(
            self.http_protocol.clone(),
            self.config.clone(),
        );
        
        tokio::spawn(async move {
            if let Err(e) = server.start().await {
                error!(?e, "HTTP server error");
            }
        });

        Ok(())
    }

    fn create_metrics_middleware(&self) -> Middleware {
        Middleware::Metrics(MetricsMiddleware)
    }

    fn create_tracing_middleware(&self) -> Middleware {
        Middleware::Logging(LoggingMiddleware)
    }

    fn create_auth_middleware(&self) -> Middleware {
        let token = self.config.security.auth.jwt_secret.clone()
            .unwrap_or_else(|| "default-secret".to_string());
        Middleware::Auth(AuthMiddleware::new(token))
    }

    fn create_rate_limit_middleware(&self) -> Middleware {
        let config = &self.config.security.rate_limit;
        Middleware::RateLimit(RateLimitMiddleware::new(
            config.requests_per_second,
            config.burst,
        ))
    }
} 
pub mod client;
mod router;
pub mod middleware;
mod server;

pub use client::{HttpClient};
pub use router::HttpRouter;
pub use middleware::{Middleware, MiddlewareChain};
pub use server::HttpServer;

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use anyhow::Result;

pub type HttpContext = std::collections::HashMap<String, String>;

#[async_trait]
pub trait HttpHandler: Send + Sync + std::fmt::Debug + 'static {
    async fn handle(&self, request: Value) -> Result<Value>;
}

pub struct HttpProtocol {
    router: HttpRouter,
    middleware: MiddlewareChain,
}

impl HttpProtocol {
    pub fn new() -> Self {
        Self {
            router: HttpRouter::new(),
            middleware: MiddlewareChain::new(),
        }
    }

    pub fn add_middleware(&mut self, middleware: Middleware) {
        self.middleware.add(middleware);
    }

    pub fn router(&mut self) -> &mut HttpRouter {
        &mut self.router
    }

    pub fn router_ref(&self) -> &HttpRouter {
        &self.router
    }

    pub fn middleware(&self) -> &MiddlewareChain {
        &self.middleware
    }
}

impl Default for HttpProtocol {
    fn default() -> Self {
        Self::new()
    }
} 
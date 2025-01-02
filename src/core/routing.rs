use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use regex::Regex;
use crate::config::types::{EndpointConfig, GatewayProtocol};
use super::handler::{BoxedHandler, HandlerResult, Request, Response};

#[derive(Debug, thiserror::Error)]
pub enum RoutingError {
    #[error("Route not found")]
    NotFound,
    #[error("Method not allowed")]
    MethodNotAllowed,
    #[error("Protocol mismatch")]
    ProtocolMismatch,
}

pub struct Route {
    pub endpoint: Arc<EndpointConfig>,
    pub handler: BoxedHandler,
}

#[async_trait]
pub trait Router: Send + Sync + 'static {
    async fn add_route(&self, route: Route) -> HandlerResult<()>;
    async fn route(&self, req: Request) -> HandlerResult<Response>;
    fn protocol(&self) -> GatewayProtocol;
}

pub struct RouterRegistry {
    routers: HashMap<GatewayProtocol, Box<dyn Router>>,
}

impl RouterRegistry {
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
        }
    }

    pub fn register<R: Router>(&mut self, router: R) {
        let protocol = router.protocol();
        self.routers.insert(protocol, Box::new(router));
    }

    pub async fn route(&self, req: Request) -> HandlerResult<Response> {
        let protocol = req.protocol.parse::<GatewayProtocol>()
            .map_err(|_| RoutingError::ProtocolMismatch)?;

        let router = self.routers.get(&protocol)
            .ok_or(RoutingError::ProtocolMismatch)?;

        router.route(req).await
    }
} 
use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post, put, delete},
    extract::{State, Json, OriginalUri},
    response::IntoResponse,
    http::StatusCode,
    body::Body,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{info, debug, error};
use anyhow::{Result, Context};

use super::{HttpProtocol, HttpContext, HttpHandler, middleware::Middleware};
use crate::config::types::Config;

pub struct HttpServer {
    protocol: Arc<RwLock<HttpProtocol>>,
    config: Arc<Config>,
}

#[derive(Clone)]
struct ServerState {
    protocol: Arc<RwLock<HttpProtocol>>,
}

impl HttpServer {
    pub fn new(protocol: Arc<RwLock<HttpProtocol>>, config: Arc<Config>) -> Self {
        Self { protocol, config }
    }

    pub async fn start(&self) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.server.port));
        let state = ServerState {
            protocol: self.protocol.clone(),
        };

        let mut app = Router::new()
            .route("/health", get(health_check));

        // Add configured routes based on their methods
        for endpoint in &self.config.endpoints {
            let path = endpoint.path.clone();
            match endpoint.method.to_uppercase().as_str() {
                "GET" => app = app.route(&path, get(handle_request)),
                "POST" => app = app.route(&path, post(handle_request)),
                "PUT" => app = app.route(&path, put(handle_request)),
                "DELETE" => app = app.route(&path, delete(handle_request)),
                _ => continue,
            };
        }

        let app = app.with_state(state);

        info!("Starting HTTP server on {}", addr);
        axum::serve(
            tokio::net::TcpListener::bind(&addr).await?,
            app.into_make_service(),
        )
        .await
        .context("Failed to start HTTP server")?;

        Ok(())
    }
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

async fn handle_request(
    State(state): State<ServerState>,
    OriginalUri(uri): OriginalUri,
    payload: Option<Json<Value>>,
) -> Result<Json<Value>, StatusCode> {
    let protocol_guard = state.protocol.read().await;
    let path = uri.path();
    let (route, params) = protocol_guard
        .router_ref()
        .match_route(path)
        .ok_or(StatusCode::NOT_FOUND)?;
    let route = route.clone();
    let middlewares: Vec<_> = protocol_guard.middleware().iter().collect();

    let mut context = HttpContext::new();
    for (k, v) in params {
        context.insert(k, v);
    }
    
    // Pre-process
    let payload_value = payload.map(|p| p.0).unwrap_or(Value::Null);
    for middleware in &middlewares {
        if let Err(e) = middleware.pre_process(&payload_value, &mut context).await {
            error!(?e, "Middleware pre-processing failed");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Execute handler
    let response = route
        .handler
        .handle(payload_value)
        .await
        .map_err(|e| {
            error!(?e, "Request handler failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Post-process
    for middleware in middlewares.iter().rev() {
        if let Err(e) = middleware.post_process(&response, &mut context).await {
            error!(?e, "Middleware post-processing failed");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_check() {
        let state = ServerState {
            protocol: Arc::new(RwLock::new(HttpProtocol::new())),
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
} 
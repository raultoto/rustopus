use std::sync::Arc;
use anyhow::{Result, Context};
use reqwest::{Client, ClientBuilder};
use serde_json::Value;
use tracing::{info, error, instrument};
use crate::config::types::{BackendConfig, CircuitBreakerConfig, RetryConfig};
use async_trait::async_trait;
use super::HttpHandler;

#[derive(Debug, Clone)]
struct BackendInfo {
    method: String,
    url: String,
}

#[derive(Debug)]
pub struct HttpClient {
    client: Client,
    backends: Vec<BackendConfig>,
    current_backend: usize,
}

impl HttpClient {
    pub fn new(backends: Vec<BackendConfig>) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(30));
            
        if let Some(retry) = backends.first().and_then(|b| b.retry.as_ref()) {
            builder = configure_retry(builder, retry);
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            backends,
            current_backend: 0,
        })
    }

    #[instrument(skip(self, payload))]
    async fn make_request(&mut self, payload: Value) -> Result<Value> {
        let mut last_error = None;
        let client = self.client.clone();
        let backends = self.backends.clone();
        let total_backends = backends.len();
        let start_backend = self.current_backend;
        self.current_backend = (start_backend + 1) % total_backends;

        for i in 0..total_backends {
            let backend_idx = (start_backend + i) % total_backends;
            let backend = &backends[backend_idx];
            let method = backend.method.as_ref().unwrap_or(&"GET".to_string()).to_string();
            
            info!(backend_url = %backend.url, "Attempting request to backend");
            
            let mut request = client
                .request(
                    reqwest::Method::from_bytes(method.as_bytes())?,
                    &backend.url
                );

            // Only add JSON body for non-GET requests
            if method != "GET" {
                request = request.json(&payload);
            }
            
            match request
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        return response.json().await.context("Failed to parse backend response");
                    }
                    last_error = Some(anyhow::anyhow!("Backend returned status: {}", response.status()));
                }
                Err(e) => {
                    error!(error = ?e, "Backend request failed");
                    last_error = Some(e.into());
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All backends failed")))
    }
}

#[async_trait]
impl HttpHandler for HttpClient {
    async fn handle(&self, payload: Value) -> Result<Value> {
        // Clone self to allow mutation of current_backend
        let mut client = Self {
            client: self.client.clone(),
            backends: self.backends.clone(),
            current_backend: self.current_backend,
        };
        
        client.make_request(payload).await
    }
}

fn configure_retry(builder: ClientBuilder, config: &RetryConfig) -> ClientBuilder {
    // Implement retry configuration
    builder
} 
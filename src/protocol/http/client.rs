use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, Context};
use async_trait::async_trait;
use reqwest::{Client as ReqwestClient, ClientBuilder};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error, instrument};
use crate::config::types::{BackendConfig, CircuitBreakerConfig, RetryConfig};
use tokio::time::sleep;
use super::HttpHandler;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: ReqwestClient,
    config: BackendConfig,
}

#[async_trait]
pub trait HttpBackend: Send + Sync + 'static {
    async fn send_request<T, R>(&self, payload: &T) -> Result<R>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned + Send;
}

#[async_trait]
impl HttpHandler for HttpClient {
    async fn handle(&self, request: Value) -> Result<Value> {
        self.send_request(&request).await
    }
}

impl HttpClient {
    pub fn new(config: BackendConfig) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(config.timeout.unwrap_or_else(|| Duration::from_secs(30)))
            .pool_idle_timeout(Some(Duration::from_secs(30)))
            .pool_max_idle_per_host(32);

        let client = builder.build().context("Failed to build HTTP client")?;

        Ok(Self { client, config })
    }

    #[instrument(skip(self, payload), fields(url = %self.config.url))]
    async fn execute_with_retry<T, R>(&self, payload: &T) -> Result<R>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let retry_config = self.config.retry.clone().unwrap_or_else(|| RetryConfig {
            attempts: 3,
            backoff: Duration::from_millis(100),
        });

        let mut attempt = 0;
        let mut last_error = None;

        while attempt < retry_config.attempts {
            match self.execute_request(payload).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    error!(error = ?e, attempt = attempt, "Request failed");
                    last_error = Some(e);
                    attempt += 1;
                    if attempt < retry_config.attempts {
                        sleep(retry_config.backoff).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    async fn execute_request<T, R>(&self, payload: &T) -> Result<R>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let method = self.config.method.as_deref().unwrap_or("POST");
        let request = self.client
            .request(method.parse()?, &self.config.url)
            .json(payload)
            .build()?;

        debug!(url = %self.config.url, method = %method, "Sending request");
        
        let response = self.client
            .execute(request)
            .await
            .context("Failed to execute request")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Backend request failed with status: {}",
                response.status()
            ));
        }

        response.json().await.context("Failed to deserialize response")
    }
}

#[async_trait]
impl HttpBackend for HttpClient {
    async fn send_request<T, R>(&self, payload: &T) -> Result<R>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        self.execute_with_retry(payload).await
    }
} 
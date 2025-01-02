use std::future::Future;
use std::pin::Pin;
use bytes::Bytes;
use http::{HeaderMap, Method, Uri, Version};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use anyhow::Result;

pub type HandlerResult<T> = Result<T>;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub protocol: String,
    pub extensions: http::Extensions,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: http::StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
    pub extensions: http::Extensions,
}

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn handle(&self, req: Request) -> HandlerResult<Response>;
}

pub type BoxedHandler = Box<dyn Handler>;
pub type HandlerFuture = Pin<Box<dyn Future<Output = HandlerResult<Response>> + Send>>; 
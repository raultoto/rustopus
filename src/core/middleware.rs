use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use async_trait::async_trait;
use super::handler::{HandlerResult, Request, Response};

pub type Next = Pin<Box<dyn Future<Output = HandlerResult<Response>> + Send>>;

#[async_trait]
pub trait Middleware: Send + Sync + 'static {
    async fn handle(&self, req: Request, next: Next) -> HandlerResult<Response>;
}

pub struct MiddlewareStack {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M>(&mut self, middleware: M)
    where
        M: Middleware + 'static,
    {
        self.middlewares.push(Arc::new(middleware));
    }

    pub async fn execute(&self, req: Request, final_handler: Next) -> HandlerResult<Response> {
        let mut chain = self.middlewares.iter().rev();
        let mut next = final_handler;

        while let Some(middleware) = chain.next() {
            let middleware = middleware.clone();
            let req = req.clone();
            let current_next = next;
            next = Box::pin(async move {
                middleware.handle(req, current_next).await
            });
        }

        next.await
    }
}

impl Clone for MiddlewareStack {
    fn clone(&self) -> Self {
        Self {
            middlewares: self.middlewares.clone(),
        }
    }
} 
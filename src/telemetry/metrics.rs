use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct Metrics {
    request_count: Arc<AtomicU64>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }
} 
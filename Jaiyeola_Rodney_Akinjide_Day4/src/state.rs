//! Shared application state.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

use crate::middleware::LeakyBucket;
use crate::models::Post;

pub type SharedState = Arc<AppState>;

pub struct AppState {
    pub posts: RwLock<HashMap<u64, Post>>,
    next_id: AtomicU64,
    pub limiter: Mutex<LeakyBucket>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            posts: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            limiter: Mutex::new(LeakyBucket::new(5.0, 1.0)),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

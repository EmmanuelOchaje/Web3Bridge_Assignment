use crate::models::Post;
use crate::middleware::LeakyBucket;
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

// ========================================
// APPLICATION STATE
// ========================================

pub struct AppState {
    // RwLock: many readers, one writer
    // Use .read().await for GET operations
    // Use .write().await for POST/DELETE operations
    pub posts: RwLock<HashMap<u64, Post>>,

    // Mutex: one request updates limiter at a time
    pub limiter: Mutex<LeakyBucket>,

    // Mutex: ensures concurrent requests generate unique IDs
    pub next_id: Mutex<u64>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            posts: RwLock::new(HashMap::new()),
            limiter: Mutex::new(LeakyBucket::new(
                5.0,  // capacity
                1.0,  // leaks per second
            )),
            next_id: Mutex::new(1),
        }
    }
}

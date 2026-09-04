use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use tokio::sync::{Mutex, RwLock};

use crate::middleware::LeakyBucket;
use crate::models::Post;

pub struct AppState {
    // RwLock:
    // many readers
    // one writer
    pub posts: RwLock<HashMap<u64, Post>>,

    // AtomicU64:
    // concurrent requests can each
    // safely claim a unique id
    pub next_id: AtomicU64,

    // Mutex:
    // one request updates
    // limiter at a time
    pub limiter: Mutex<LeakyBucket>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            posts: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            limiter: Mutex::new(LeakyBucket::new(
                5.0, // capacity
                1.0, // leaks per second
            )),
        }
    }
}

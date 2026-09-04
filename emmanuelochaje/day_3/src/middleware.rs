use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

// ========================================
// LEAKY BUCKET
// ========================================

pub struct LeakyBucket {
    capacity: f64,
    water: f64,
    leak_rate: f64,
    last_check: Instant,
}

impl LeakyBucket {
    pub fn new(capacity: f64, leak_rate: f64) -> Self {
        Self {
            capacity,
            water: 0.0,
            leak_rate,
            last_check: Instant::now(),
        }
    }

    pub fn allow(&mut self) -> bool {
        // How much time passed?
        let elapsed = self.last_check.elapsed().as_secs_f64();

        // How much water leaked?
        let leaked = elapsed * self.leak_rate;

        // Remove leaked water
        self.water = (self.water - leaked).max(0.0);

        // Update our clock
        self.last_check = Instant::now();

        // Is bucket full?
        if self.water + 1.0 > self.capacity {
            return false;
        }

        // Add this request
        self.water += 1.0;

        true
    }
}

// ========================================
// RATE LIMIT MIDDLEWARE
// ========================================

pub async fn rate_limit(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let allowed = {
        let mut limiter = state.limiter.lock().await;
        limiter.allow()
    };

    if !allowed {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please slow down.",
        )
            .into_response();
    }

    next.run(request).await
}

// ========================================
// LOGGING MIDDLEWARE
// ========================================

pub async fn logging(request: Request, next: Next) -> Response {
    println!("Incoming request: {} {}", request.method(), request.uri().path());

    next.run(request).await
}

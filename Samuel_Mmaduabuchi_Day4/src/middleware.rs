use std::{sync::Arc, time::Instant};

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::state::AppState;

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

    fn allow(&mut self) -> bool {
        let elapsed = self.last_check.elapsed().as_secs_f64();
        let leaked = elapsed * self.leak_rate;

        self.water = (self.water - leaked).max(0.0);
        self.last_check = Instant::now();

        if self.water + 1.0 > self.capacity {
            return false;
        }

        self.water += 1.0;
        true
    }
}

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

pub async fn logging(request: Request, next: Next) -> Response {
    println!(
        "Incoming request: {} {}",
        request.method(),
        request.uri().path()
    );

    next.run(request).await
}

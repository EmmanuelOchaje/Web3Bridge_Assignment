//! Day 4 — Axum Leaky Bucket Rate Limiter, split into modules.
//!
//! `main.rs` only creates the state, builds the router, attaches the
//! middleware and starts the server. Everything else lives in its own module.

mod handlers;
mod middleware;
mod models;
mod routes;
mod state;

use std::sync::Arc;

use crate::routes::create_router;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind to 127.0.0.1:3000");

    println!("Rate limiter API running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("server error");
}

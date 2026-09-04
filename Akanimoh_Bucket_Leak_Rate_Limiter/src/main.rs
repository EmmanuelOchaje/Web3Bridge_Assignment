mod handlers;
mod middleware;
mod models;
mod routes;
mod state;

use axum::middleware as axum_middleware;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create application state
    let state = Arc::new(state::AppState::new());

    // Create router with routes
    let app = routes::create_router(state.clone())
        // Add logging middleware first (outermost)
        .layer(axum_middleware::from_fn(middleware::logging))
        // Add rate limiter middleware
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::rate_limit,
        ));

    // Bind to address
    let listener = tokio::net::TcpListener::bind("127.0.0.1:4000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:4000");

    // Start server
    axum::serve(listener, app).await.unwrap();
}

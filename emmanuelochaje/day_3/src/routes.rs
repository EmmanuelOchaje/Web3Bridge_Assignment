use std::sync::Arc;

use axum::{middleware as axum_middleware, routing::get, Router};

use crate::handlers;
use crate::middleware::{logging, rate_limit};
use crate::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(handlers::home))
        .route(
            "/posts",
            get(handlers::get_posts).post(handlers::create_post),
        )
        .route(
            "/posts/:id",
            get(handlers::get_post).delete(handlers::delete_post),
        )
        // Request
        //   -> Logging Middleware
        //   -> Rate Limiter Middleware
        //   -> Route Handler
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            rate_limit,
        ))
        .layer(axum_middleware::from_fn(logging))
        .with_state(state)
}

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::handlers::{create_post, delete_post, get_post, get_posts, home};
use crate::state::AppState;

// ========================================
// APPLICATION ROUTES
// ========================================

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(home))
        .route("/posts", get(get_posts).post(create_post))
        .route("/posts/:id", get(get_post).delete(delete_post))
        .with_state(state)
}

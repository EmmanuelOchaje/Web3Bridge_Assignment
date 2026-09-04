use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    handlers::{create_post, delete_post, get_post, get_posts},
    state::AppState,
};

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/posts", get(get_posts).post(create_post))
        .route("/posts/{id}", get(get_post).delete(delete_post))
}

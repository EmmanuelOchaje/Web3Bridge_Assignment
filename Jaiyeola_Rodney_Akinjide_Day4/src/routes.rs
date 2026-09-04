use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::get,
    Router,
};

use crate::handlers::{create_post, delete_post, get_post, list_posts};
use crate::middleware::{logging, rate_limit};
use crate::state::SharedState;


pub fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/:id", get(get_post).delete(delete_post))
        .layer(from_fn_with_state(state.clone(), rate_limit))
        .layer(from_fn(logging))
        .with_state(state)
}

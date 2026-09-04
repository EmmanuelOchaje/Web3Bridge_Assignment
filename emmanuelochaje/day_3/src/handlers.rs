use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::{CreatePost, ErrorResponse, Post};
use crate::state::AppState;

pub async fn home() -> &'static str {
    "Rate Limiter API"
}

// ========================================
// GET /posts
// ========================================

pub async fn get_posts(State(state): State<Arc<AppState>>) -> Json<Vec<Post>> {
    let posts = state.posts.read().await;

    let mut list: Vec<Post> = posts.values().cloned().collect();
    list.sort_by_key(|post| post.id);

    Json(list)
}

// ========================================
// GET /posts/:id
// ========================================

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Post>, (StatusCode, Json<ErrorResponse>)> {
    let posts = state.posts.read().await;

    match posts.get(&id) {
        Some(post) => Ok(Json(post.clone())),
        None => Err(not_found()),
    }
}

// ========================================
// POST /posts
// ========================================

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePost>,
) -> (StatusCode, Json<Post>) {
    let id = state.next_id.fetch_add(1, Ordering::SeqCst);

    let post = Post {
        id,
        title: payload.title,
        content: payload.content,
    };

    let mut posts = state.posts.write().await;
    posts.insert(id, post.clone());

    (StatusCode::CREATED, Json(post))
}

// ========================================
// DELETE /posts/:id
// ========================================

pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut posts = state.posts.write().await;

    match posts.remove(&id) {
        Some(_) => Ok(StatusCode::OK),
        None => Err(not_found()),
    }
}

fn not_found() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Post not found".to_string(),
        }),
    )
}

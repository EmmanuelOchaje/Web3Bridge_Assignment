use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::models::{CreatePost, ErrorResponse, Post};
use crate::state::AppState;

// ========================================
// HOME
// ========================================

pub async fn home() -> &'static str {
    "Rate Limiter API"
}

// ========================================
// GET ALL POSTS
// ========================================

pub async fn get_posts(State(state): State<Arc<AppState>>) -> Json<Vec<Post>> {
    let posts = state.posts.read().await;
    let posts_vec: Vec<Post> = posts.values().cloned().collect();
    Json(posts_vec)
}

// ========================================
// GET POST BY ID
// ========================================

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Post>, (StatusCode, Json<ErrorResponse>)> {
    let posts = state.posts.read().await;

    match posts.get(&id) {
        Some(post) => Ok(Json(post.clone())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Post not found".to_string(),
            }),
        )),
    }
}

// ========================================
// CREATE POST
// ========================================

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePost>,
) -> (StatusCode, Json<Post>) {
    // Get the next ID and increment it
    let id = {
        let mut next_id = state.next_id.lock().await;
        let current_id = *next_id;
        *next_id += 1;
        current_id
    };

    let post = Post {
        id,
        title: payload.title,
        content: payload.content,
    };

    // Store the post
    {
        let mut posts = state.posts.write().await;
        posts.insert(id, post.clone());
    }

    (StatusCode::CREATED, Json(post))
}

// ========================================
// DELETE POST
// ========================================

pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let mut posts = state.posts.write().await;

    match posts.remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Post not found".to_string(),
            }),
        )),
    }
}

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use crate::{
    models::{CreatePost, ErrorResponse, Post},
    state::AppState,
};

type ApiError = (StatusCode, Json<ErrorResponse>);

pub async fn get_posts(State(state): State<Arc<AppState>>) -> Json<Vec<Post>> {
    let posts = state.posts.read().await;
    let mut posts: Vec<Post> = posts.values().cloned().collect();
    posts.sort_by_key(|post| post.id);

    Json(posts)
}

pub async fn get_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Post>, ApiError> {
    let posts = state.posts.read().await;

    posts.get(&id).cloned().map(Json).ok_or_else(post_not_found)
}

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreatePost>,
) -> (StatusCode, Json<Post>) {
    let post = Post {
        id: state.next_post_id(),
        title: payload.title,
        content: payload.content,
    };

    let mut posts = state.posts.write().await;
    posts.insert(post.id, post.clone());

    (StatusCode::CREATED, Json(post))
}

pub async fn delete_post(
    State(state): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<Json<Post>, ApiError> {
    let mut posts = state.posts.write().await;

    posts.remove(&id).map(Json).ok_or_else(post_not_found)
}

fn post_not_found() -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Post not found".to_owned(),
        }),
    )
}

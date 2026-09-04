//! Route handler functions.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::{CreatePost, ErrorResponse, MessageResponse, Post};
use crate::state::SharedState;

type ApiError = (StatusCode, Json<ErrorResponse>);

fn post_not_found() -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "Post not found".to_string(),
        }),
    )
}

/// `GET /posts` — return every post, ordered by id.
pub async fn list_posts(State(state): State<SharedState>) -> Json<Vec<Post>> {
    let posts = state.posts.read().await;
    let mut all: Vec<Post> = posts.values().cloned().collect();
    all.sort_by_key(|post| post.id);
    Json(all)
}

pub async fn get_post(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<Json<Post>, ApiError> {
    let posts = state.posts.read().await;
    posts
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(post_not_found)
}

pub async fn create_post(
    State(state): State<SharedState>,
    Json(payload): Json<CreatePost>,
) -> (StatusCode, Json<Post>) {
    let post = Post {
        id: state.next_id(),
        title: payload.title,
        content: payload.content,
    };

    let mut posts = state.posts.write().await;
    posts.insert(post.id, post.clone());

    (StatusCode::CREATED, Json(post))
}

/// `DELETE /posts/:id` — remove a post or `404`.
pub async fn delete_post(
    State(state): State<SharedState>,
    Path(id): Path<u64>,
) -> Result<(StatusCode, Json<MessageResponse>), ApiError> {
    let mut posts = state.posts.write().await;
    if posts.remove(&id).is_some() {
        Ok((
            StatusCode::OK,
            Json(MessageResponse {
                message: format!("Post {id} deleted"),
            }),
        ))
    } else {
        Err(post_not_found())
    }
}

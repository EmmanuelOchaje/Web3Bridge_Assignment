use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

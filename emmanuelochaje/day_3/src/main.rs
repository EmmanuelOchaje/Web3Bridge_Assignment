mod handlers;
mod middleware;
mod models;
mod routes;
mod state;

use std::sync::Arc;

use state::AppState;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::new());

    let app = routes::create_router(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

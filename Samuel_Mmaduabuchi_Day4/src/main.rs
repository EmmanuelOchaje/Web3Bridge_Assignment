mod handlers;
mod middleware;
mod models;
mod routes;
mod state;

use std::{io, sync::Arc};

use axum::middleware as axum_middleware;
use middleware::{logging, rate_limit};
use state::AppState;

const ADDRESS: &str = "127.0.0.1:3000";

#[tokio::main]
async fn main() -> io::Result<()> {
    let state = Arc::new(AppState::new());

    let app = routes::create_routes()
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            rate_limit,
        ))
        .layer(axum_middleware::from_fn(logging))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(ADDRESS).await?;
    println!("Server running on http://{ADDRESS}");

    axum::serve(listener, app).await
}

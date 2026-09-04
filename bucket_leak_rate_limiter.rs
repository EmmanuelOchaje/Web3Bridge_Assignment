use axum::{
    extract::{
        Request,
        State,
    },

    http::StatusCode,

    middleware::{
        self,
        Next,
    },

    response::{
        IntoResponse,
        Response,
    },

    routing::{
        get,
        post,
    },

    Json,
    Router,
};

use serde::{
    Deserialize,
    Serialize,
};

use std::{
    sync::Arc,
    time::Instant,
};

use tokio::sync::{
    Mutex,
    RwLock,
};


// ========================================
// POST
// ========================================

#[derive(Debug, Clone, Serialize)]
struct Post {
    id: usize,
    message: String,
}


// ========================================
// POST REQUEST BODY
// ========================================

#[derive(Debug, Deserialize)]
struct CreatePost {
    message: String,
}


// ========================================
// LEAKY BUCKET
// ========================================

struct LeakyBucket {
    capacity: f64,
    water: f64,
    leak_rate: f64,
    last_check: Instant,
}


impl LeakyBucket {

    fn new(
        capacity: f64,
        leak_rate: f64,
    ) -> Self {

        Self {
            capacity,

            water: 0.0,

            leak_rate,

            last_check: Instant::now(),
        }
    }


    fn allow(&mut self) -> bool {

        // How much time passed?
        let elapsed =
            self.last_check.elapsed().as_secs_f64();


        // How much water leaked?
        let leaked =
            elapsed * self.leak_rate;


        // Remove leaked water
        self.water =
            (self.water - leaked).max(0.0);


        // Update our clock
        self.last_check =
            Instant::now();


        // Is bucket full?
        if self.water + 1.0 > self.capacity {

            return false;
        }


        // Add this request
        self.water += 1.0;


        true
    }
}


// ========================================
// APPLICATION STATE
// ========================================

struct AppState {

    // RwLock:
    // many readers
    // one writer
    posts: RwLock<Vec<Post>>,


    // Mutex:
    // one request updates
    // limiter at a time
    limiter: Mutex<LeakyBucket>,
}


// ========================================
// MAIN
// ========================================

#[tokio::main]
async fn main() {

    let state = Arc::new(
        AppState {

            posts: RwLock::new(
                Vec::new()
            ),

            limiter: Mutex::new(
                LeakyBucket::new(
                    5.0, // capacity
                    1.0, // leaks per second
                )
            ),
        }
    );


    let app = Router::new()

        .route(
            "/",
            get(home)
        )

        .route(
            "/posts",
            get(get_posts)
                .post(create_post)
        )

        // NEW:
        // Every request passes through
        // rate_limit first.
        .layer(
            middleware::from_fn_with_state(
                state.clone(),
                rate_limit,
            )
        )

        .with_state(state);


    let listener =
        tokio::net::TcpListener::bind(
            "127.0.0.1:3000"
        )
        .await
        .unwrap();


    println!(
        "Server running on http://127.0.0.1:3000"
    );


    axum::serve(
        listener,
        app,
    )
    .await
    .unwrap();
}


// ========================================
// RATE LIMIT MIDDLEWARE
// ========================================

async fn rate_limit(

    State(state):
        State<Arc<AppState>>,

    request: Request,

    next: Next,

) -> Response {

    let allowed = {

        let mut limiter =
            state.limiter.lock().await;

        limiter.allow()
    };


    if !allowed {

        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please slow down."
        )
        .into_response();
    }


    next.run(request).await
}


// ========================================
// HANDLERS
// ========================================

async fn home()
    -> &'static str
{
    "Rate Limiter API"
}


// ========================================
// GET POSTS
// ========================================

async fn get_posts(

    State(state):
        State<Arc<AppState>>,

) -> Json<Vec<Post>> {

    let posts =
        state.posts.read().await;

    Json(posts.clone())
}


// ========================================
// CREATE POST
// ========================================

async fn create_post(

    State(state):
        State<Arc<AppState>>,

    Json(payload):
        Json<CreatePost>,

) -> (StatusCode, Json<Post>) {

    let mut posts =
        state.posts.write().await;


    let post = Post {

        id: posts.len() + 1,

        message: payload.message,
    };


    posts.push(
        post.clone()
    );


    (
        StatusCode::CREATED,
        Json(post)
    )
}

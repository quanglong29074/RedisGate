use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{Path, State},
    response::{Sse, sse::Event},
};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use redis::{Client, AsyncCommands};
use tokio::sync::Mutex;
use futures::{Stream, StreamExt};
use async_stream::stream;

// Type alias for Redis connection
type RedisConn = Arc<Mutex<redis::aio::MultiplexedConnection>>;

// Success and Error response structures
#[derive(Serialize, Deserialize)]
struct SuccessResponse<T> {
    result: Option<T>,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

// Initialize Redis connection
async fn init_redis(url: &str) -> RedisConn {
    let client = Client::open(url).unwrap();
    let conn = client.get_multiplexed_async_connection().await.unwrap();
    Arc::new(Mutex::new(conn))
}

// Handler to respond with "Hello"
async fn hello_handler() -> Json<serde_json::Value> {
    Json(json!({ "result": "Hello" }))
}

// Publish handler to send a message to a Redis channel
async fn publish_handler(
    State(conn): State<RedisConn>,
    Path((channel, message)): Path<(String, String)>,
) -> Json<serde_json::Value> {
    let mut conn = conn.lock().await;
    let res: redis::RedisResult<()> = conn.publish(channel, message).await;
    match res {
        Ok(_) => Json(json!({ "result": "OK" })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

// Subscribe handler for a Redis channel, returns SSE stream
async fn subscribe_handler(Path(channel): Path<String>) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let client = Client::open("redis://127.0.0.1:6379/").unwrap();
    let mut pubsub_conn = client.get_async_pubsub().await.unwrap();

    // Subscribe to the channel
    pubsub_conn.subscribe(&channel).await.unwrap();

    // Create stream to emit incoming messages from Redis
    let mut messages = pubsub_conn.into_on_message();

    let s = stream! {
        while let Some(msg) = messages.next().await {
            let payload: String = msg.get_payload().unwrap_or_default();
            yield Ok(Event::default().data(payload));
        }
    };

    Sse::new(s)
}

// Main function to set up the server and routes
#[tokio::main]
async fn main() {
    println!("🚀 Server running");

    // Initialize Redis connection
    let redis_conn = init_redis("redis://127.0.0.1:6379/").await;
    println!("Redis connection established.");

    // Define the app and routes
    let app = Router::new()
        .route("/", get(hello_handler))
        .route("/publish/{channel}/{message}", post(publish_handler))
        .route("/subscribe/{channel}", get(subscribe_handler))
        .with_state(redis_conn);

    // Define server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running at http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Error starting server: {}", e);
    }
}

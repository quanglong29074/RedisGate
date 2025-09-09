use axum::{routing::{get, post}, Router};
use redis::Client;
use serde::{Serialize, Deserialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

mod handlers;
use handlers::keys::{set_key, get_key};

// Alias cho Redis connection
type RedisConn = Arc<Mutex<redis::aio::MultiplexedConnection>>;


/// Hàm khởi tạo Redis connection
async fn init_redis(url: &str) -> RedisConn {
    let client = Client::open(url).unwrap();
    let conn = client.get_multiplexed_async_connection().await.unwrap();
    Arc::new(Mutex::new(conn))
}

/// Hàm main khởi chạy server
#[tokio::main]
async fn main() {
    println!("🚀 Server starting...");

    // Redis connection
    let redis_conn = init_redis("redis://127.0.0.1:6379/").await;
    println!("✅ Redis connection established");

    // Router
    let app = Router::new()
        .route("/set/{key}/{value}", post(set_key))
        .route("/get/{key}", get(get_key))
        .with_state(redis_conn);

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🌐 Server running at http://{}", addr);


    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ Error starting server: {}", e);
    }
}

use axum::{routing::{get, post}, Router};
use redis::Client;
use serde::{Serialize, Deserialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use deadpool_redis::{Config as RedisConfig, Pool};

mod handlers;
use handlers::keys::{set_key, get_key};

// Alias 
type RedisPool = Pool;

/// Khởi tạo Redis connection pool
async fn init_redis_pool(url: &str) -> RedisPool {
    let cfg = RedisConfig::from_url(url);
    cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .unwrap()
}

/// Hàm main khởi chạy server
#[tokio::main]
async fn main() {
    println!("🚀 Server starting...");

    // Redis pool
    let redis_pool = init_redis_pool("redis://127.0.0.1:6379/").await;
    println!("✅ Redis pool established");

    // Router
    let app = Router::new()
        .route("/set/{key}/{value}", post(set_key))
        .route("/get/{key}", get(get_key))
        .with_state(redis_pool);

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("🌐 Server running at http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("❌ Error starting server: {}", e);
    }
}

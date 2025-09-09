// main.rs
use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use deadpool_redis::{Pool, Runtime};

mod handlers;
use handlers::keys::{get_key, set_key};
use handlers::health::health_check;

mod config;
use crate::config::{Config, RedisConfig};

// Alias for Redis pool type
type RedisPool = Pool;

/// Initialize Redis connection pool
async fn init_redis_pool(cfg: &RedisConfig) -> Pool {
    // Create pool configuration from Redis URL
    let mut cfg_pool = deadpool_redis::Config::from_url(cfg.url.as_str());

    // Customize pool settings if needed
    cfg_pool.pool = Some(deadpool_redis::PoolConfig {
        max_size: cfg.pool_size as usize,
        timeouts: Default::default(),
    });

    cfg_pool
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

/// Main function to start the server
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Server starting...");

    // Load configuration
    let cfg = Config::from_env()?;
    println!("⚙️ Loaded config: {:?}", cfg);

    // Initialize Redis pool
    let redis_pool = init_redis_pool(&cfg.redis).await;
    println!("✅ Redis pool established");

    // Build router with routes
    let app = Router::new()
        .route("/set/{key}/{value}", post(set_key))
        .route("/get/{key}", get(get_key))
        .route("/healthz", get(health_check))
        .with_state(redis_pool);

    // Define server address
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    println!("🌐 Server running at http://{}", addr);

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await;
    Ok(())
}

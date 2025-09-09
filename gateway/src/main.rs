// src/main.rs
use axum::{
    routing::{get, post},
    Router,
};
use deadpool_redis::{Pool, Runtime};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{fmt};

mod config;
use crate::config::{Config, RedisConfig};

mod handlers;
use handlers::health::{health_check, metrics_endpoint};
use handlers::keys::{get_key, set_key};

mod middleware;
use middleware::{logging::request_logger, metrics::metrics_middleware};

/// Alias for Redis connection pool
type RedisPool = Pool;

/// Initialize Redis connection pool
async fn init_redis_pool(cfg: &RedisConfig) -> RedisPool {
    let mut cfg_pool = deadpool_redis::Config::from_url(cfg.url.as_str());

    // Optional pool configuration
    cfg_pool.pool = Some(deadpool_redis::PoolConfig {
        max_size: cfg.pool_size as usize,
        timeouts: Default::default(),
    });

    cfg_pool
        .create_pool(Some(Runtime::Tokio1))
        .expect("Failed to create Redis pool")
}

/// Main entry point
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber
    fmt::init();

    println!("🚀 Server starting...");

    // Load configuration from default.toml and environment variables
    let cfg = Config::from_env()?;
    println!("⚙️ Loaded config: {:?}", cfg);

    // Initialize Redis connection pool
    let redis_pool = init_redis_pool(&cfg.redis).await;
    println!("✅ Redis pool established");

    // Initialize Prometheus metrics recorder
    let prometheus_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    // Build application router
    let app = Router::new()
        .route("/set/{key}/{value}", post(set_key))
        .route("/get/{key}", get(get_key))
        .route("/healthz", get(health_check))
        .route(
            "/metrics",
            get({
                let handle = prometheus_handle.clone();
                move || async move { metrics_endpoint(handle.clone()).await }
            }),
        )
        .with_state(redis_pool)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(request_logger))
                .layer(axum::middleware::from_fn(metrics_middleware)),
        );

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.server.port));
    println!("🌐 Server running at http://{}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await;

    Ok(())
}

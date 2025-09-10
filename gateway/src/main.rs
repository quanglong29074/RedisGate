// src/main.rs
use axum::{
    routing::{get, post},
    Router,
};
use deadpool_redis::{Pool, Runtime};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{fmt, prelude::*};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_subscriber::{filter::EnvFilter};

mod config;
mod handlers;
mod error;
mod redis;

use crate::config::{Config, RedisConfig};

use handlers::health::{health_check, metrics_endpoint};
use handlers::keys::{get_key, set_key};

mod middleware;
use middleware::{logging::request_logger, metrics::metrics_middleware};



/// Alias for Redis connection pool
type RedisPool = Pool;

/// Initialize Redis connection pool
async fn init_redis_pool(cfg: &RedisConfig) -> RedisPool {
    let mut cfg_pool = deadpool_redis::Config::from_url(&cfg.url);

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
    // Load configuration
    let cfg = Config::from_env()?;
    tracing::info!(?cfg, "⚙️ Loaded configuration");

    let log_level = cfg.logging.level.clone();
    let server_port = cfg.server.port;
    let redis_cfg = cfg.redis.clone();

    // Initialize tracing subscriber with env filter
    tracing_subscriber::registry()
        .with(
            EnvFilter::new(log_level),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 Server starting...");

    tracing::info!(?cfg, "⚙️ Loaded configuration");

    // Initialize Redis connection pool
    let redis_pool = init_redis_pool(&redis_cfg).await;
    tracing::info!("✅ Redis pool established");

    // Initialize Prometheus metrics recorder
    let metrics_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    let redis_pool_for_router = redis_pool.clone();
    let redis_pool_for_shutdown = redis_pool.clone();

    // Build application router
    let app = Router::new()
        .route("/set/{key}/{value}", post(set_key))
        .route("/get/{key}", get(get_key))
        .route("/healthz", get(health_check))
        .route(
            "/metrics",
            get({
                let handle = metrics_handle.clone();
                move || async move { metrics_endpoint(handle.clone()).await }
            }),
        )
        .with_state(redis_pool_for_router)
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(request_logger))
                .layer(axum::middleware::from_fn(metrics_middleware)),
        );

    // Server address
    let addr = SocketAddr::from(([0, 0, 0, 0], server_port));
    tracing::info!(%addr, "🌐 Server running");
    println!("🌐 Server running at http://{}", addr);

    // Start server with graceful shutdown
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("⚡ Dropping Redis pool...");
            drop(redis_pool_for_shutdown);
        })
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    #[cfg(not(unix))]
    ctrl_c.await;

    tracing::info!("⚡ Shutdown signal received");
}

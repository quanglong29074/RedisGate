use axum::{extract::Extension, response::Json, routing::get, Router};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{info, warn};

mod models;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Database migrations completed successfully");

    // Build application
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/version", get(version))
        .route("/stats", get(database_stats))
        .layer(Extension(Arc::new(pool)));

    // Start server
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to address");

    info!("Server starting on 0.0.0.0:8080");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn health_check(Extension(pool): Extension<Arc<PgPool>>) -> Json<serde_json::Value> {
    // Test database connection
    let db_status = match sqlx::query("SELECT 1 as status")
        .fetch_one(pool.as_ref())
        .await
    {
        Ok(row) => {
            let status: i32 = row.get("status");
            if status == 1 {
                "healthy"
            } else {
                "unhealthy"
            }
        }
        Err(e) => {
            warn!("Database health check failed: {}", e);
            "unhealthy"
        }
    };

    Json(json!({
        "status": "ok",
        "database": db_status,
        "timestamp": chrono::Utc::now()
    }))
}

async fn version() -> Json<serde_json::Value> {
    Json(json!({
        "name": "redisgate",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Cloud Redis on Kubernetes HTTP Gateway"
    }))
}

async fn database_stats(Extension(pool): Extension<Arc<PgPool>>) -> Json<serde_json::Value> {
    // Get table counts to demonstrate compile-time checked queries
    let user_count = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    let org_count = sqlx::query!("SELECT COUNT(*) as count FROM organizations")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    let redis_instance_count = sqlx::query!("SELECT COUNT(*) as count FROM redis_instances")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    let api_key_count = sqlx::query!("SELECT COUNT(*) as count FROM api_keys")
        .fetch_one(pool.as_ref())
        .await
        .map(|row| row.count.unwrap_or(0))
        .unwrap_or(0);

    Json(json!({
        "tables": {
            "users": user_count,
            "organizations": org_count,
            "redis_instances": redis_instance_count,
            "api_keys": api_key_count
        },
        "timestamp": chrono::Utc::now()
    }))
}

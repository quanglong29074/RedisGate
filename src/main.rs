use axum::{
    extract::Extension,
    middleware as axum_middleware,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod api_models;
mod auth;
mod handlers;
mod middleware;
mod models;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    info!("Database migrations completed successfully");

    // Create application state
    let app_state = Arc::new(middleware::AppState::new(pool.clone(), &jwt_secret));

    // Build application with all routes
    let app = Router::new()
        // Public routes (no authentication required)
        .route("/health", get(health_check))
        .route("/version", get(version))
        .route("/stats", get(database_stats))
        .route("/auth/register", post(handlers::auth::register))
        .route("/auth/login", post(handlers::auth::login))
        
        // Protected routes (authentication required)
        .nest("/api", 
            Router::new()
                .route("/organizations", post(handlers::organizations::create_organization))
                .route("/organizations", get(handlers::organizations::list_organizations))
                .route("/organizations/:org_id", get(handlers::organizations::get_organization))
                .route("/organizations/:org_id", put(handlers::organizations::update_organization))
                .route("/organizations/:org_id", delete(handlers::organizations::delete_organization))
                
                .route("/organizations/:org_id/api-keys", post(handlers::api_keys::create_api_key))
                .route("/organizations/:org_id/api-keys", get(handlers::api_keys::list_api_keys))
                .route("/organizations/:org_id/api-keys/:key_id", get(handlers::api_keys::get_api_key))
                .route("/organizations/:org_id/api-keys/:key_id", delete(handlers::api_keys::revoke_api_key))
                
                .route("/organizations/:org_id/redis-instances", post(handlers::redis_instances::create_redis_instance))
                .route("/organizations/:org_id/redis-instances", get(handlers::redis_instances::list_redis_instances))
                .route("/organizations/:org_id/redis-instances/:instance_id", get(handlers::redis_instances::get_redis_instance))
                .route("/organizations/:org_id/redis-instances/:instance_id", delete(handlers::redis_instances::delete_redis_instance))
                
                // Apply authentication middleware only to protected routes
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::auth_middleware,
                ))
        )
        .layer(CorsLayer::permissive())
        .with_state(app_state)
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

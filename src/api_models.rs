// API request and response models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SLUG_REGEX: Regex = Regex::new(r"^[a-z0-9-]+$").unwrap();
}

// User registration request
#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(length(min = 1, max = 50))]
    pub first_name: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub last_name: Option<String>,
}

// User login request
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

// User response
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

// Organization creation request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrganizationRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 50), regex(path = "*SLUG_REGEX"))]
    pub slug: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}

// Organization response
#[derive(Debug, Serialize)]
pub struct OrganizationResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub plan: String,
    pub max_redis_instances: i32,
    pub max_api_keys: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// API key creation request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub organization_id: Uuid,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

// API key response
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub organization_id: Uuid,
    pub scopes: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// API key creation response (includes full key)
#[derive(Debug, Serialize)]
pub struct ApiKeyCreationResponse {
    pub api_key: ApiKeyResponse,
    pub key: String, // Only returned on creation
}

// Redis instance creation request
#[derive(Debug, Deserialize, Validate)]
pub struct CreateRedisInstanceRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 1, max = 50), regex(path = "*SLUG_REGEX"))]
    pub slug: String,
    pub organization_id: Uuid,
    #[validate(range(min = 1048576, max = 17179869184i64))] // 1MB to 16GB
    pub max_memory: i64,
    pub redis_version: Option<String>,
    pub persistence_enabled: Option<bool>,
    pub backup_enabled: Option<bool>,
}

// Redis instance response
#[derive(Debug, Serialize)]
pub struct RedisInstanceResponse {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub organization_id: Uuid,
    pub api_key_id: Option<Uuid>,
    pub port: i32,
    pub domain: Option<String>,
    pub max_memory: i64,
    pub current_memory: i64,
    pub redis_version: String,
    pub namespace: String,
    pub status: String,
    pub health_status: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub connections_count: i32,
    pub max_connections: i32,
    pub persistence_enabled: bool,
    pub backup_enabled: bool,
    pub last_backup_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message),
            timestamp: Utc::now(),
        }
    }
}

// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
        }
    }
}

// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}
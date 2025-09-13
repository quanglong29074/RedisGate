// Database models for RedisGate
// These models correspond to the database tables created by migrations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: bool,
    pub is_verified: bool,
    pub verification_token: Option<String>,
    pub reset_password_token: Option<String>,
    pub reset_password_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Organization {
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

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub scopes: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub last_used_ip: Option<IpAddr>,
    pub is_active: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct RedisInstance {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub organization_id: Uuid,
    pub api_key_id: Uuid,

    // Network configuration
    pub port: i32,
    pub private_ip_address: Option<IpAddr>,
    pub public_ip_address: Option<IpAddr>,
    pub domain: Option<String>,

    // Redis configuration
    pub max_memory: i64,
    pub current_memory: i64,
    pub password_hash: Option<String>,
    pub redis_version: String,

    // Kubernetes configuration
    pub namespace: String,
    pub pod_name: Option<String>,
    pub service_name: Option<String>,

    // Instance status and metadata
    pub status: String,
    pub last_health_check_at: Option<DateTime<Utc>>,
    pub health_status: String,

    // Resource usage tracking
    pub cpu_usage_percent: rust_decimal::Decimal,
    pub memory_usage_percent: rust_decimal::Decimal,
    pub connections_count: i32,
    pub max_connections: i32,

    // Backup and persistence
    pub persistence_enabled: bool,
    pub backup_enabled: bool,
    pub last_backup_at: Option<DateTime<Utc>>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct OrganizationMembership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub invited_by: Option<Uuid>,
    pub invitation_token: Option<String>,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<IpAddr>,
    pub user_agent: Option<String>,
    pub api_key_id: Option<Uuid>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Database models for RedisGate
// These models correspond to the database tables created by migrations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use sqlx::types::BigDecimal;
use ipnetwork;

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_active: Option<bool>, // Allow nullable
    pub is_verified: Option<bool>, // Allow nullable
    pub verification_token: Option<String>,
    pub reset_password_token: Option<String>,
    pub reset_password_expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub owner_id: Uuid,
    pub is_active: Option<bool>,
    pub plan: Option<String>,
    pub max_redis_instances: Option<i32>,
    pub max_api_keys: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
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
    pub last_used_ip: Option<ipnetwork::IpNetwork>,
    pub is_active: Option<bool>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct RedisInstance {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub organization_id: Uuid,
    pub api_key_id: Uuid,

    // Network configuration
    pub port: Option<i32>,
    pub private_ip_address: Option<ipnetwork::IpNetwork>,
    pub public_ip_address: Option<ipnetwork::IpNetwork>,
    pub domain: Option<String>,

    // Redis configuration
    pub max_memory: Option<i64>,
    pub current_memory: Option<i64>,
    pub password_hash: Option<String>,
    pub redis_version: Option<String>,

    // Kubernetes configuration
    pub namespace: Option<String>,
    pub pod_name: Option<String>,
    pub service_name: Option<String>,

    // Instance status and metadata
    pub status: Option<String>,
    pub last_health_check_at: Option<DateTime<Utc>>,
    pub health_status: Option<String>,

    // Resource usage tracking
    pub cpu_usage_percent: Option<BigDecimal>,
    pub memory_usage_percent: Option<BigDecimal>,
    pub connections_count: Option<i32>,
    pub max_connections: Option<i32>,

    // Backup and persistence
    pub persistence_enabled: Option<bool>,
    pub backup_enabled: Option<bool>,
    pub last_backup_at: Option<DateTime<Utc>>,

    // Timestamps
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct OrganizationMembership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub role: Option<String>,
    pub permissions: Vec<String>,
    pub is_active: Option<bool>,
    pub invited_by: Option<Uuid>,
    pub invitation_token: Option<String>,
    pub invitation_expires_at: Option<DateTime<Utc>>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub action: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<ipnetwork::IpNetwork>,
    pub user_agent: Option<String>,
    pub api_key_id: Option<Uuid>,
    pub status: Option<String>,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

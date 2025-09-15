// Authentication utilities for RedisGate

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub email: String,
    pub org_id: Option<Uuid>,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyClaims {
    pub api_key_id: Uuid,
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub scopes: Vec<String>,
    pub key_prefix: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, email: String, org_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        Self {
            user_id,
            email,
            org_id,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }
}

impl ApiKeyClaims {
    pub fn new(
        api_key_id: Uuid,
        user_id: Uuid,
        organization_id: Uuid,
        scopes: Vec<String>,
        key_prefix: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        let exp = expires_at
            .unwrap_or_else(|| now + Duration::days(365)) // Default to 1 year if no expiry
            .timestamp();

        Self {
            api_key_id,
            user_id,
            organization_id,
            scopes,
            key_prefix,
            exp,
            iat: now.timestamp(),
        }
    }
}

#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    pub fn create_token(&self, claims: &Claims) -> Result<String, AuthError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|_| AuthError::TokenCreationFailed)
    }

    pub fn create_api_key_token(&self, claims: &ApiKeyClaims) -> Result<String, AuthError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|_| AuthError::TokenCreationFailed)
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, AuthError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)
    }

    pub fn verify_api_key_token(&self, token: &str) -> Result<TokenData<ApiKeyClaims>, AuthError> {
        decode::<ApiKeyClaims>(token, &self.decoding_key, &Validation::default())
            .map_err(|_| AuthError::InvalidToken)
    }
}

#[derive(Debug)]
pub enum AuthError {
    TokenCreationFailed,
    InvalidToken,
    TokenExpired,
    MissingToken,
    InvalidCredentials,
    UserNotFound,
    UserNotActive,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::TokenCreationFailed => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
            AuthError::UserNotActive => (StatusCode::UNAUTHORIZED, "User account is not active"),
        };

        (status, message).into_response()
    }
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;
    
    #[test]
    fn test_api_key_claims_creation() {
        let api_key_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let scopes = vec!["read".to_string(), "write".to_string()];
        let key_prefix = "rg_test123".to_string();
        let expires_at = Some(Utc::now() + chrono::Duration::days(30));

        let claims = ApiKeyClaims::new(
            api_key_id,
            user_id,
            organization_id,
            scopes.clone(),
            key_prefix.clone(),
            expires_at,
        );

        assert_eq!(claims.api_key_id, api_key_id);
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.organization_id, organization_id);
        assert_eq!(claims.scopes, scopes);
        assert_eq!(claims.key_prefix, key_prefix);
        assert_eq!(claims.exp, expires_at.unwrap().timestamp());
    }

    #[test]
    fn test_jwt_manager_api_key_token_cycle() {
        let secret = "test-secret-key-for-jwt";
        let jwt_manager = JwtManager::new(secret);

        let api_key_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let organization_id = Uuid::new_v4();
        let scopes = vec!["read".to_string()];
        let key_prefix = "rg_test123".to_string();

        let claims = ApiKeyClaims::new(
            api_key_id,
            user_id,
            organization_id,
            scopes.clone(),
            key_prefix.clone(),
            None, // No expiry
        );

        // Create token
        let token = jwt_manager.create_api_key_token(&claims).unwrap();
        assert!(!token.is_empty());
        assert!(token.contains('.'), "JWT token should contain dots");

        // Verify token
        let verified = jwt_manager.verify_api_key_token(&token).unwrap();
        assert_eq!(verified.claims.api_key_id, api_key_id);
        assert_eq!(verified.claims.user_id, user_id);
        assert_eq!(verified.claims.organization_id, organization_id);
        assert_eq!(verified.claims.scopes, scopes);
        assert_eq!(verified.claims.key_prefix, key_prefix);
    }

    #[test] 
    fn test_invalid_token_verification() {
        let jwt_manager = JwtManager::new("test-secret");
        
        // Test invalid token
        let result = jwt_manager.verify_api_key_token("invalid-token");
        assert!(result.is_err());
        
        // Test token with wrong signature
        let wrong_secret_manager = JwtManager::new("wrong-secret");
        let claims = ApiKeyClaims::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            vec!["read".to_string()],
            "rg_test".to_string(),
            None,
        );
        let token = wrong_secret_manager.create_api_key_token(&claims).unwrap();
        
        let verify_result = jwt_manager.verify_api_key_token(&token);
        assert!(verify_result.is_err());
    }
}
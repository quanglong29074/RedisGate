// Simple demonstration of JWT API key functionality
// Run with: cargo run --bin demo_jwt

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyClaims {
    pub api_key_id: String,
    pub user_id: String,
    pub organization_id: String,
    pub scopes: Vec<String>,
    pub key_prefix: String,
    pub exp: i64,
    pub iat: i64,
}

impl ApiKeyClaims {
    pub fn new(
        api_key_id: String,
        user_id: String,
        organization_id: String,
        scopes: Vec<String>,
        key_prefix: String,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::days(365); // 1 year expiry

        Self {
            api_key_id,
            user_id,
            organization_id,
            scopes,
            key_prefix,
            exp: exp.timestamp(),
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

    pub fn create_api_key_token(&self, claims: &ApiKeyClaims) -> Result<String, String> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| format!("Token creation failed: {}", e))
    }

    pub fn verify_api_key_token(&self, token: &str) -> Result<TokenData<ApiKeyClaims>, String> {
        decode::<ApiKeyClaims>(token, &self.decoding_key, &Validation::default())
            .map_err(|e| format!("Token verification failed: {}", e))
    }
}

fn main() {
    println!("🚀 JWT API Key Demo for RedisGate");
    println!("==================================");

    // Create JWT manager
    let jwt_secret = "demo-secret-key";
    let jwt_manager = JwtManager::new(jwt_secret);

    // Create API key claims
    let api_key_id = Uuid::new_v4().to_string();
    let user_id = Uuid::new_v4().to_string();
    let organization_id = Uuid::new_v4().to_string();
    let scopes = vec!["read".to_string(), "write".to_string()];
    let key_prefix = format!("rg_{}", &api_key_id[..8]);

    let claims = ApiKeyClaims::new(
        api_key_id.clone(),
        user_id.clone(),
        organization_id.clone(),
        scopes.clone(),
        key_prefix.clone(),
    );

    println!("\n📋 API Key Claims:");
    println!("  API Key ID: {}", claims.api_key_id);
    println!("  User ID: {}", claims.user_id);
    println!("  Organization ID: {}", claims.organization_id);
    println!("  Scopes: {:?}", claims.scopes);
    println!("  Key Prefix: {}", claims.key_prefix);

    // Generate JWT token
    match jwt_manager.create_api_key_token(&claims) {
        Ok(token) => {
            println!("\n🔑 Generated JWT Token:");
            println!("  {}", token);
            println!("  Length: {} characters", token.len());
            println!("  Contains dots: {}", token.contains('.'));

            // Verify the token
            match jwt_manager.verify_api_key_token(&token) {
                Ok(verified) => {
                    println!("\n✅ Token Verification Successful!");
                    println!("  API Key ID: {}", verified.claims.api_key_id);
                    println!("  Organization ID: {}", verified.claims.organization_id);
                    println!("  Scopes: {:?}", verified.claims.scopes);
                    println!("  Expires: {}", chrono::DateTime::from_timestamp(verified.claims.exp, 0).unwrap());
                    
                    // Test invalid token
                    let invalid_result = jwt_manager.verify_api_key_token("invalid.token.here");
                    match invalid_result {
                        Err(_) => println!("\n❌ Invalid token correctly rejected"),
                        Ok(_) => println!("\n⚠️  Warning: Invalid token was accepted!"),
                    }

                    println!("\n🎉 JWT API Key implementation working correctly!");
                    println!("   ✓ No database lookup required for verification");
                    println!("   ✓ Self-contained tokens with organization context");
                    println!("   ✓ Fast verification for Redis API requests");
                }
                Err(e) => {
                    println!("\n❌ Token verification failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("\n❌ Token generation failed: {}", e);
        }
    }
}
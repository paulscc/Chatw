use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // NEW VERSION: Remove database_url dependency
    pub server_addr: String,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_service_role_key: String,
    pub redis_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            // NEW VERSION: Remove database_url dependency
            server_addr: env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            jwt_expiration: env::var("JWT_EXPIRATION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(86400), // 24 hours
            supabase_url: env::var("SUPABASE_URL")
                .unwrap_or_else(|_| "https://your-project.supabase.co".to_string()),
            supabase_anon_key: env::var("SUPABASE_ANON_KEY")
                .unwrap_or_else(|_| "your-anon-key".to_string()),
            supabase_service_role_key: env::var("SUPABASE_SERVICE_ROLE_KEY")
                .unwrap_or_else(|_| "your-service-role-key".to_string()),
            redis_url: env::var("REDIS_URL")
                .expect("REDIS_URL must be set (Upstash Redis URL)"),
        })
    }
}

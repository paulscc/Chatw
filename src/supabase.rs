use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::AppError;
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseUser {
    pub id: String,
    pub email: String,
    pub email_confirmed_at: Option<String>,
    pub phone: Option<String>,
    pub phone_confirmed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub user_metadata: HashMap<String, serde_json::Value>,
    pub app_metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub data: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub user: SupabaseUser,
}

#[derive(Debug, Clone)]
pub struct SupabaseClient {
    client: Client,
    url: String,
    anon_key: String,
    service_role_key: String,
}

impl SupabaseClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            url: config.supabase_url.clone(),
            anon_key: config.supabase_anon_key.clone(),
            service_role_key: config.supabase_service_role_key.clone(),
        }
    }

    fn get_headers(&self, use_service_role: bool) -> HashMap<String, String> {
        let key = if use_service_role {
            &self.service_role_key
        } else {
            &self.anon_key
        };

        let mut headers = HashMap::new();
        headers.insert("apikey".to_string(), key.clone());
        headers.insert("Authorization".to_string(), format!("Bearer {}", key));
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers
    }

    // Auth endpoints
    pub async fn sign_up(&self, email: &str, password: str) -> Result<AuthResponse, AppError> {
        let headers = self.get_headers(false);
        let body = SignUpRequest {
            email: email.to_string(),
            password,
            data: None,
        };

        let response = self.client
            .post(&format!("{}/auth/v1/signup", self.url))
            .headers((&headers).try_into().unwrap())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response
                .json()
                .await
                .map_err(|e| AppError::ExternalService(e.to_string()))?;
            Ok(auth_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Authentication(error_text))
        }
    }

    pub async fn sign_in(&self, email: &str, password: str) -> Result<AuthResponse, AppError> {
        let headers = self.get_headers(false);
        let body = SignInRequest {
            email: email.to_string(),
            password,
        };

        let response = self.client
            .post(&format!("{}/auth/v1/token?grant_type=password", self.url))
            .headers((&headers).try_into().unwrap())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            let auth_response: AuthResponse = response
                .json()
                .await
                .map_err(|e| AppError::ExternalService(e.to_string()))?;
            Ok(auth_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Authentication(error_text))
        }
    }

    pub async fn sign_out(&self, access_token: &str) -> Result<(), AppError> {
        let mut headers = self.get_headers(false);
        headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));

        let response = self.client
            .post(&format!("{}/auth/v1/logout", self.url))
            .headers((&headers).try_into().unwrap())
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Authentication(error_text))
        }
    }

    pub async fn get_user(&self, access_token: &str) -> Result<SupabaseUser, AppError> {
        let mut headers = self.get_headers(false);
        headers.insert("Authorization".to_string(), format!("Bearer {}", access_token));

        let response = self.client
            .get(&format!("{}/auth/v1/user", self.url))
            .headers((&headers).try_into().unwrap())
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            let user: SupabaseUser = response
                .json()
                .await
                .map_err(|e| AppError::ExternalService(e.to_string()))?;
            Ok(user)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Authentication(error_text))
        }
    }

    // Database endpoints (using REST API)
    pub async fn insert<T: Serialize>(&self, table: &str, data: &T) -> Result<serde_json::Value, AppError> {
        let headers = self.get_headers(true);

        let response = self.client
            .post(&format!("{}/rest/v1/{}", self.url, table))
            .headers((&headers).try_into().unwrap())
            .json(data)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| AppError::ExternalService(e.to_string()))?;
            Ok(result)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Database(error_text))
        }
    }

    pub async fn select(&self, table: &str, query: &str) -> Result<Vec<serde_json::Value>, AppError> {
        let headers = self.get_headers(true);

        let response = self.client
            .get(&format!("{}/rest/v1/{}?{}", self.url, table, query))
            .headers((&headers).try_into().unwrap())
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            let result: Vec<serde_json::Value> = response
                .json()
                .await
                .map_err(|e| AppError::ExternalService(e.to_string()))?;
            Ok(result)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Database(error_text))
        }
    }

    pub async fn update<T: Serialize>(&self, table: &str, id: &str, data: &T) -> Result<serde_json::Value, AppError> {
        let headers = self.get_headers(true);

        let response = self.client
            .patch(&format!("{}/rest/v1/{}?id=eq.{}", self.url, table, id))
            .headers((&headers).try_into().unwrap())
            .json(data)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            let result: serde_json::Value = response
                .json()
                .await
                .map_err(|e| AppError::ExternalService(e.to_string()))?;
            Ok(result)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Database(error_text))
        }
    }

    pub async fn delete(&self, table: &str, id: &str) -> Result<(), AppError> {
        let headers = self.get_headers(true);

        let response = self.client
            .delete(&format!("{}/rest/v1/{}?id=eq.{}", self.url, table, id))
            .headers((&headers).try_into().unwrap())
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(AppError::Database(error_text))
        }
    }
}

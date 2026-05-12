use crate::supabase::{SupabaseClient, SupabaseUser};
use crate::db::Database;
use crate::error::AppError;
use crate::models::{Profile, UpdateProfile, PresenceStatus};
use uuid::Uuid;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SupabaseAuthService {
    supabase: SupabaseClient,
    db: Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignUpRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUserResponse {
    pub user: Profile,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestSessionResponse {
    pub user: Profile,
    pub guest_token: String,
}

impl SupabaseAuthService {
    pub fn new(supabase: SupabaseClient, db: Database) -> Self {
        Self { supabase, db }
    }

    // Authentication methods
    pub async fn sign_up(&self, request: SignUpRequest) -> Result<AuthUserResponse, AppError> {
        // Sign up in Supabase Auth
        let mut user_metadata = HashMap::new();
        if let Some(display_name) = &request.display_name {
            user_metadata.insert("display_name".to_string(), json!(display_name));
        }

        let auth_response = self.supabase.sign_up(&request.email, &request.password).await?;

        // Create profile in our database
        let profile = self.create_or_update_profile_from_supabase_user(&auth_response.user, request.display_name.as_ref()).await?;

        Ok(AuthUserResponse {
            user: profile,
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_in: auth_response.expires_in,
        })
    }

    pub async fn sign_in(&self, request: SignInRequest) -> Result<AuthUserResponse, AppError> {
        // Sign in with Supabase Auth
        let auth_response = self.supabase.sign_in(&request.email, &request.password).await?;

        // Get or create profile in our database
        let profile = self.create_or_update_profile_from_supabase_user(&auth_response.user, None).await?;

        Ok(AuthUserResponse {
            user: profile,
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            expires_in: auth_response.expires_in,
        })
    }

    pub async fn sign_out(&self, access_token: &str) -> Result<(), AppError> {
        self.supabase.sign_out(access_token).await
    }

    pub async fn get_current_user(&self, access_token: &str) -> Result<Profile, AppError> {
        let supabase_user = self.supabase.get_user(access_token).await?;
        
        // Get or create profile in our database
        let profile = self.create_or_update_profile_from_supabase_user(&supabase_user, None).await?;
        
        Ok(profile)
    }

    pub async fn create_guest_session(&self, display_name: String) -> Result<GuestSessionResponse, AppError> {
        let id = Uuid::new_v4();
        let guest_token = Uuid::new_v4().to_string();
        let now = Utc::now();

        let profile = sqlx::query_as::<_, Profile>(
            r#"
            INSERT INTO profiles (id, email, guest_token, display_name, avatar_url, status_message, presence, is_guest, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(None::<String>)
        .bind(&guest_token)
        .bind(&display_name)
        .bind(None::<String>)
        .bind(None::<String>)
        .bind(PresenceStatus::Online)
        .bind(true)
        .bind(json!({"session_type": "guest"}))
        .bind(now)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(GuestSessionResponse {
            user: profile,
            guest_token,
        })
    }

    // Profile management methods (integrated with Supabase)
    async fn create_or_update_profile_from_supabase_user(&self, supabase_user: &SupabaseUser, display_name: Option<&String>) -> Result<Profile, AppError> {
        let user_uuid = Uuid::parse_str(&supabase_user.id)
            .map_err(|_| AppError::Validation("Invalid user ID format".to_string()))?;

        // Try to get existing profile
        match sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = $1")
            .bind(user_uuid)
            .fetch_optional(&self.db.pool)
            .await?
        {
            Some(mut profile) => {
                // Update existing profile with latest Supabase data
                profile.email = Some(supabase_user.email.clone());
                profile.updated_at = Utc::now();
                
                // Update display name if provided or if not set
                if let Some(display_name) = display_name {
                    profile.display_name = display_name.clone();
                } else if profile.display_name.is_empty() || profile.display_name == "Unknown User" {
                    // Extract display name from email or user metadata
                    profile.display_name = supabase_user.user_metadata
                        .get("display_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or_else(|| {
                            supabase_user.email.split('@').next().unwrap_or("Unknown User")
                        })
                        .to_string();
                }

                // Update in database
                let updated_profile = sqlx::query_as::<_, Profile>(
                    "UPDATE profiles SET email = $1, display_name = $2, updated_at = $3 WHERE id = $4 RETURNING *"
                )
                .bind(&profile.email)
                .bind(&profile.display_name)
                .bind(Utc::now())
                .bind(profile.id)
                .fetch_one(&self.db.pool)
                .await?;

                Ok(updated_profile)
            }
            None => {
                // Create new profile
                let now = Utc::now();
                let display_name_final = display_name
                    .cloned()
                    .or_else(|| {
                        supabase_user.user_metadata
                            .get("display_name")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| {
                        supabase_user.email.split('@').next().unwrap_or("Unknown User").to_string()
                    });

                let profile = sqlx::query_as::<_, Profile>(
                    r#"
                    INSERT INTO profiles (id, email, guest_token, display_name, avatar_url, status_message, presence, is_guest, metadata, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                    RETURNING *
                    "#
                )
                .bind(user_uuid)
                .bind(&supabase_user.email)
                .bind(None::<String>)
                .bind(&display_name_final)
                .bind(None::<String>)
                .bind(None::<String>)
                .bind(PresenceStatus::Online)
                .bind(false)
                .bind(json!({
                    "supabase_user_id": supabase_user.id,
                    "email_confirmed_at": supabase_user.email_confirmed_at,
                    "phone": supabase_user.phone,
                    "phone_confirmed_at": supabase_user.phone_confirmed_at,
                    "user_metadata": supabase_user.user_metadata,
                    "app_metadata": supabase_user.app_metadata
                }))
                .bind(now)
                .bind(now)
                .fetch_one(&self.db.pool)
                .await?;

                Ok(profile)
            }
        }
    }

    // Legacy profile methods (keeping compatibility)
    pub async fn get_profile_by_id(&self, id: Uuid) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.db.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Profile {} not found", id)))?;

        Ok(profile)
    }

    pub async fn get_profile_by_email(&self, email: &str) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.db.pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Profile with email {} not found", email)))?;

        Ok(profile)
    }

    pub async fn get_profile_by_guest_token(&self, token: &str) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE guest_token = $1")
            .bind(token)
            .fetch_optional(&self.db.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Profile with guest token not found".to_string()))?;

        Ok(profile)
    }

    pub async fn update_profile(&self, id: Uuid, update: UpdateProfile) -> Result<Profile, AppError> {
        let mut query = String::from("UPDATE profiles SET updated_at = NOW()");
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(display_name) = &update.display_name {
            param_count += 1;
            query.push_str(&format!(", display_name = ${}", param_count));
            params.push(display_name.clone());
        }

        if let Some(avatar_url) = &update.avatar_url {
            param_count += 1;
            query.push_str(&format!(", avatar_url = ${}", param_count));
            params.push(avatar_url.clone());
        }

        if let Some(status_message) = &update.status_message {
            param_count += 1;
            query.push_str(&format!(", status_message = ${}", param_count));
            params.push(status_message.clone());
        }

        if let Some(presence) = update.presence {
            param_count += 1;
            query.push_str(&format!(", presence = ${}", param_count));
            params.push(presence.to_string());
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut query_builder = sqlx::query_as::<_, Profile>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(id);

        let profile = query_builder
            .fetch_one(&self.db.pool)
            .await?;

        Ok(profile)
    }

    pub async fn update_presence(&self, id: Uuid, presence: PresenceStatus) -> Result<Profile, AppError> {
        let now = Utc::now();
        let profile = sqlx::query_as::<_, Profile>(
            r#"
            UPDATE profiles 
            SET presence = $1, last_seen_at = $2, updated_at = $2
            WHERE id = $3
            RETURNING *
            "#
        )
        .bind(presence)
        .bind(now)
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(profile)
    }

    pub async fn ban_profile(&self, id: Uuid) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>("UPDATE profiles SET is_banned = true, updated_at = NOW() WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.db.pool)
            .await?;

        Ok(profile)
    }

    pub async fn unban_profile(&self, id: Uuid) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>("UPDATE profiles SET is_banned = false, updated_at = NOW() WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_one(&self.db.pool)
            .await?;

        Ok(profile)
    }

    pub async fn delete_profile(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM profiles WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn list_profiles(&self, limit: i64, offset: i64) -> Result<Vec<Profile>, AppError> {
        let profiles = sqlx::query_as::<_, Profile>("SELECT * FROM profiles ORDER BY created_at DESC LIMIT $1 OFFSET $2")
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db.pool)
            .await?;

        Ok(profiles)
    }
}

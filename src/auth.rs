use crate::db::Database;
use crate::error::AppError;
use crate::models::{Profile, CreateProfile, UpdateProfile, PresenceStatus};
use uuid::Uuid;
use chrono::Utc;

pub struct AuthService {
    db: Database,
}

impl AuthService {
    pub fn new(db: Database) -> Self {
        AuthService { db }
    }

    pub async fn create_profile(&self, profile: CreateProfile) -> Result<Profile, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let profile = sqlx::query_as::<_, Profile>(
            r#"
            INSERT INTO profiles (id, email, guest_token, display_name, avatar_url, status_message, presence, is_guest, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&profile.email)
        .bind(&profile.guest_token)
        .bind(&profile.display_name)
        .bind(&profile.avatar_url)
        .bind(&profile.status_message)
        .bind(PresenceStatus::Offline)
        .bind(profile.is_guest)
        .bind(serde_json::json!({}))
        .bind(now)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(profile)
    }

    pub async fn get_profile_by_id(&self, id: Uuid) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Profile {} not found", id)))?;

        Ok(profile)
    }

    pub async fn get_profile_by_email(&self, email: &str) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Profile with email {} not found", email)))?;

        Ok(profile)
    }

    pub async fn get_profile_by_guest_token(&self, token: &str) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles WHERE guest_token = $1"
        )
        .bind(token)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Profile with guest token not found")))?;

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
        let profile = sqlx::query_as::<_, Profile>(
            "UPDATE profiles SET is_banned = true, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(profile)
    }

    pub async fn unban_profile(&self, id: Uuid) -> Result<Profile, AppError> {
        let profile = sqlx::query_as::<_, Profile>(
            "UPDATE profiles SET is_banned = false, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
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
        let profiles = sqlx::query_as::<_, Profile>(
            "SELECT * FROM profiles ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(profiles)
    }
}

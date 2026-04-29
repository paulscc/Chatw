use crate::db::Database;
use crate::error::AppError;
use crate::models::{Channel, CreateChannel, UpdateChannel, ChannelType};
use uuid::Uuid;
use chrono::Utc;

pub struct ChannelService {
    db: Database,
}

impl ChannelService {
    pub fn new(db: Database) -> Self {
        ChannelService { db }
    }

    pub async fn create_channel(&self, channel: CreateChannel) -> Result<Channel, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let is_private = channel.is_private.unwrap_or(false);

        let channel = sqlx::query_as::<_, Channel>(
            r#"
            INSERT INTO channels (id, workspace_id, name, description, type, is_private, is_archived, last_message_at, message_count, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, false, NULL, 0, $7, $8)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(channel.workspace_id)
        .bind(&channel.name)
        .bind(&channel.description)
        .bind(channel.r#type)
        .bind(is_private)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(channel)
    }

    pub async fn get_channel_by_id(&self, id: Uuid) -> Result<Channel, AppError> {
        let channel = sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Channel {} not found", id)))?;

        Ok(channel)
    }

    pub async fn update_channel(&self, id: Uuid, update: UpdateChannel) -> Result<Channel, AppError> {
        let mut query = String::from("UPDATE channels SET updated_at = NOW()");
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(name) = &update.name {
            param_count += 1;
            query.push_str(&format!(", name = ${}", param_count));
            params.push(name.clone());
        }

        if let Some(description) = &update.description {
            param_count += 1;
            query.push_str(&format!(", description = ${}", param_count));
            params.push(description.clone());
        }

        if let Some(is_private) = update.is_private {
            param_count += 1;
            query.push_str(&format!(", is_private = ${}", param_count));
            params.push(is_private.to_string());
        }

        if let Some(is_archived) = update.is_archived {
            param_count += 1;
            query.push_str(&format!(", is_archived = ${}", param_count));
            params.push(is_archived.to_string());
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut query_builder = sqlx::query_as::<_, Channel>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(id);

        let channel = query_builder
            .fetch_one(&self.db.pool)
            .await?;

        Ok(channel)
    }

    pub async fn archive_channel(&self, id: Uuid) -> Result<Channel, AppError> {
        let channel = sqlx::query_as::<_, Channel>(
            "UPDATE channels SET is_archived = true, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(channel)
    }

    pub async fn unarchive_channel(&self, id: Uuid) -> Result<Channel, AppError> {
        let channel = sqlx::query_as::<_, Channel>(
            "UPDATE channels SET is_archived = false, updated_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(channel)
    }

    pub async fn delete_channel(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM channels WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn list_workspace_channels(&self, workspace_id: Uuid) -> Result<Vec<Channel>, AppError> {
        let channels = sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE workspace_id = $1 ORDER BY created_at ASC"
        )
        .bind(workspace_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(channels)
    }

    pub async fn list_public_channels(&self, workspace_id: Uuid) -> Result<Vec<Channel>, AppError> {
        let channels = sqlx::query_as::<_, Channel>(
            "SELECT * FROM channels WHERE workspace_id = $1 AND type = $2 AND is_private = false AND is_archived = false ORDER BY created_at ASC"
        )
        .bind(workspace_id)
        .bind(ChannelType::Public)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(channels)
    }
}

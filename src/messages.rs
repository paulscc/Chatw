use crate::db::Database;
use crate::error::AppError;
use crate::models::{
    Message, CreateMessage, UpdateMessage, MessageType,
    MessageVersion, MessageAttachment, CreateMessageAttachment,
    MessageReaction, CreateMessageReaction
};
use uuid::Uuid;
use chrono::Utc;

pub struct MessageService {
    db: Database,
}

impl MessageService {
    pub fn new(db: Database) -> Self {
        MessageService { db }
    }

    // ==================== MESSAGES ====================

    pub async fn create_message(&self, message: CreateMessage) -> Result<Message, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let metadata = message.metadata.unwrap_or_else(|| serde_json::json!({}));

        let message = sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, channel_id, conversation_id, sender_id, parent_id, content, type, is_edited, is_deleted, is_pinned, pinned_at, deleted_at, edited_at, reply_count, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, false, false, false, NULL, NULL, NULL, 0, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(message.channel_id)
        .bind(message.conversation_id)
        .bind(message.sender_id)
        .bind(message.parent_id)
        .bind(&message.content)
        .bind(message.r#type)
        .bind(metadata)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(message)
    }

    pub async fn get_message_by_id(&self, id: Uuid) -> Result<Message, AppError> {
        let message = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Message {} not found", id)))?;

        Ok(message)
    }

    pub async fn update_message(&self, id: Uuid, update: UpdateMessage) -> Result<Message, AppError> {
        let mut query = String::from("UPDATE messages SET updated_at = NOW()");
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(content) = &update.content {
            param_count += 1;
            query.push_str(&format!(", content = ${}", param_count));
            params.push(content.clone());
        }

        if let Some(is_deleted) = update.is_deleted {
            param_count += 1;
            query.push_str(&format!(", is_deleted = ${}", param_count));
            params.push(is_deleted.to_string());
        }

        if let Some(is_pinned) = update.is_pinned {
            param_count += 1;
            query.push_str(&format!(", is_pinned = ${}", param_count));
            params.push(is_pinned.to_string());
            if is_pinned {
                param_count += 1;
                query.push_str(&format!(", pinned_at = ${}", param_count));
                params.push(Utc::now().to_string());
            }
        }

        if let Some(metadata) = update.metadata {
            param_count += 1;
            query.push_str(&format!(", metadata = ${}", param_count));
            params.push(metadata.to_string());
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut query_builder = sqlx::query_as::<_, Message>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(id);

        let message = query_builder
            .fetch_one(&self.db.pool)
            .await?;

        Ok(message)
    }

    pub async fn delete_message(&self, id: Uuid) -> Result<Message, AppError> {
        // Soft delete
        let message = self.update_message(id, UpdateMessage {
            content: None,
            is_deleted: Some(true),
            is_pinned: None,
            metadata: None,
        }).await?;

        Ok(message)
    }

    pub async fn pin_message(&self, id: Uuid) -> Result<Message, AppError> {
        let message = self.update_message(id, UpdateMessage {
            content: None,
            is_deleted: None,
            is_pinned: Some(true),
            metadata: None,
        }).await?;

        Ok(message)
    }

    pub async fn unpin_message(&self, id: Uuid) -> Result<Message, AppError> {
        let message = self.update_message(id, UpdateMessage {
            content: None,
            is_deleted: None,
            is_pinned: Some(false),
            metadata: None,
        }).await?;

        Ok(message)
    }

    pub async fn list_channel_messages(&self, channel_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Message>, AppError> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE channel_id = $1 AND is_deleted = false ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(channel_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(messages)
    }

    pub async fn list_conversation_messages(&self, conversation_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Message>, AppError> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE conversation_id = $1 AND is_deleted = false ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(conversation_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(messages)
    }

    pub async fn list_thread_messages(&self, parent_id: Uuid, limit: i64, offset: i64) -> Result<Vec<Message>, AppError> {
        let messages = sqlx::query_as::<_, Message>(
            "SELECT * FROM messages WHERE parent_id = $1 AND is_deleted = false ORDER BY created_at ASC LIMIT $2 OFFSET $3"
        )
        .bind(parent_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(messages)
    }

    pub async fn search_messages(&self, workspace_id: Uuid, query: &str, limit: i64) -> Result<Vec<Message>, AppError> {
        let messages = sqlx::query_as::<_, Message>(
            r#"
            SELECT m.* FROM messages m
            INNER JOIN channels c ON m.channel_id = c.id
            WHERE c.workspace_id = $1
            AND m.fts_weighted @@ plainto_tsquery('spanish', $2)
            AND m.is_deleted = false
            ORDER BY m.created_at DESC
            LIMIT $3
            "#
        )
        .bind(workspace_id)
        .bind(query)
        .bind(limit)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(messages)
    }

    // ==================== MESSAGE VERSIONS ====================

    pub async fn get_message_versions(&self, message_id: Uuid) -> Result<Vec<MessageVersion>, AppError> {
        let versions = sqlx::query_as::<_, MessageVersion>(
            "SELECT * FROM message_versions WHERE message_id = $1 ORDER BY edited_at DESC"
        )
        .bind(message_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(versions)
    }

    // ==================== MESSAGE ATTACHMENTS ====================

    pub async fn add_attachment(&self, attachment: CreateMessageAttachment) -> Result<MessageAttachment, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let attachment = sqlx::query_as::<_, MessageAttachment>(
            r#"
            INSERT INTO message_attachments (id, message_id, file_url, file_name, file_type, file_size, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(attachment.message_id)
        .bind(&attachment.file_url)
        .bind(&attachment.file_name)
        .bind(&attachment.file_type)
        .bind(attachment.file_size)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(attachment)
    }

    pub async fn get_message_attachments(&self, message_id: Uuid) -> Result<Vec<MessageAttachment>, AppError> {
        let attachments = sqlx::query_as::<_, MessageAttachment>(
            "SELECT * FROM message_attachments WHERE message_id = $1 ORDER BY created_at ASC"
        )
        .bind(message_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(attachments)
    }

    pub async fn delete_attachment(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM message_attachments WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    // ==================== MESSAGE REACTIONS ====================

    pub async fn add_reaction(&self, reaction: CreateMessageReaction) -> Result<MessageReaction, AppError> {
        let id = Uuid::new_v4();

        let reaction = sqlx::query_as::<_, MessageReaction>(
            r#"
            INSERT INTO message_reactions (id, message_id, profile_id, emoji)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (message_id, profile_id, emoji) DO NOTHING
            RETURNING *
            "#
        )
        .bind(id)
        .bind(reaction.message_id)
        .bind(reaction.profile_id)
        .bind(&reaction.emoji)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(reaction)
    }

    pub async fn remove_reaction(&self, message_id: Uuid, profile_id: Uuid, emoji: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM message_reactions WHERE message_id = $1 AND profile_id = $2 AND emoji = $3")
            .bind(message_id)
            .bind(profile_id)
            .bind(emoji)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn get_message_reactions(&self, message_id: Uuid) -> Result<Vec<MessageReaction>, AppError> {
        let reactions = sqlx::query_as::<_, MessageReaction>(
            "SELECT * FROM message_reactions WHERE message_id = $1"
        )
        .bind(message_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(reactions)
    }
}

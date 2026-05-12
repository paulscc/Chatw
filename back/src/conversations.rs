use crate::db::Database;
use crate::error::AppError;
use crate::models::{
    Conversation, CreateConversation,
    ConversationParticipant, CreateConversationParticipant,
    ConversationType
};
use uuid::Uuid;
use chrono::Utc;

pub struct ConversationService {
    db: Database,
}

impl ConversationService {
    pub fn new(db: Database) -> Self {
        ConversationService { db }
    }

    // ==================== CONVERSATIONS ====================

    pub async fn create_conversation(&self, conversation: CreateConversation) -> Result<Conversation, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let conversation = sqlx::query_as::<_, Conversation>(
            r#"
            INSERT INTO conversations (id, workspace_id, type, last_message_at, created_at, updated_at)
            VALUES ($1, $2, $3, NULL, $4, $5)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(conversation.workspace_id)
        .bind(conversation.r#type)
        .bind(now)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(conversation)
    }

    pub async fn get_conversation_by_id(&self, id: Uuid) -> Result<Conversation, AppError> {
        let conversation = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Conversation {} not found", id)))?;

        Ok(conversation)
    }

    pub async fn update_conversation(&self, id: Uuid, last_message_at: Option<chrono::DateTime<Utc>>) -> Result<Conversation, AppError> {
        let conversation = sqlx::query_as::<_, Conversation>(
            r#"
            UPDATE conversations 
            SET last_message_at = COALESCE($1, last_message_at), updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#
        )
        .bind(last_message_at)
        .bind(id)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(conversation)
    }

    pub async fn delete_conversation(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn list_workspace_conversations(&self, workspace_id: Uuid) -> Result<Vec<Conversation>, AppError> {
        let conversations = sqlx::query_as::<_, Conversation>(
            "SELECT * FROM conversations WHERE workspace_id = $1 ORDER BY last_message_at DESC NULLS LAST"
        )
        .bind(workspace_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(conversations)
    }

    pub async fn list_user_conversations(&self, profile_id: Uuid) -> Result<Vec<Conversation>, AppError> {
        let conversations = sqlx::query_as::<_, Conversation>(
            r#"
            SELECT c.* FROM conversations c
            INNER JOIN conversation_participants cp ON c.id = cp.conversation_id
            WHERE cp.profile_id = $1 AND cp.is_active = true
            ORDER BY c.last_message_at DESC NULLS LAST
            "#
        )
        .bind(profile_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(conversations)
    }

    // ==================== CONVERSATION PARTICIPANTS ====================

    pub async fn add_participant(&self, participant: CreateConversationParticipant) -> Result<ConversationParticipant, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let participant = sqlx::query_as::<_, ConversationParticipant>(
            r#"
            INSERT INTO conversation_participants (id, conversation_id, profile_id, is_active, joined_at)
            VALUES ($1, $2, $3, true, $4)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(participant.conversation_id)
        .bind(participant.profile_id)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(participant)
    }

    pub async fn get_participant(&self, conversation_id: Uuid, profile_id: Uuid) -> Result<ConversationParticipant, AppError> {
        let participant = sqlx::query_as::<_, ConversationParticipant>(
            "SELECT * FROM conversation_participants WHERE conversation_id = $1 AND profile_id = $2"
        )
        .bind(conversation_id)
        .bind(profile_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Participant not found".to_string()))?;

        Ok(participant)
    }

    pub async fn remove_participant(&self, conversation_id: Uuid, profile_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM conversation_participants WHERE conversation_id = $1 AND profile_id = $2")
            .bind(conversation_id)
            .bind(profile_id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn list_conversation_participants(&self, conversation_id: Uuid) -> Result<Vec<ConversationParticipant>, AppError> {
        let participants = sqlx::query_as::<_, ConversationParticipant>(
            "SELECT * FROM conversation_participants WHERE conversation_id = $1 AND is_active = true ORDER BY joined_at ASC"
        )
        .bind(conversation_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(participants)
    }

    // ==================== DIRECT MESSAGE HELPERS ====================

    pub async fn get_or_create_direct_conversation(&self, profile1: Uuid, profile2: Uuid) -> Result<Conversation, AppError> {
        // Try to find existing direct conversation between these two users
        let existing = sqlx::query_as::<_, Conversation>(
            r#"
            SELECT c.* FROM conversations c
            WHERE c.type = $1
            AND id IN (
                SELECT conversation_id FROM conversation_participants WHERE profile_id = $2
                INTERSECT
                SELECT conversation_id FROM conversation_participants WHERE profile_id = $3
            )
            LIMIT 1
            "#
        )
        .bind(ConversationType::Direct)
        .bind(profile1)
        .bind(profile2)
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(conv) = existing {
            return Ok(conv);
        }

        // Create new direct conversation
        let conv = self.create_conversation(CreateConversation {
            workspace_id: None,
            r#type: ConversationType::Direct,
        }).await?;

        // Add both participants
        self.add_participant(CreateConversationParticipant {
            conversation_id: conv.id,
            profile_id: profile1,
        }).await?;

        self.add_participant(CreateConversationParticipant {
            conversation_id: conv.id,
            profile_id: profile2,
        }).await?;

        Ok(conv)
    }
}

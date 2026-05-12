use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// ENUMS
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "presence_status", rename_all = "lowercase")]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

impl std::fmt::Display for PresenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PresenceStatus::Online => write!(f, "online"),
            PresenceStatus::Away => write!(f, "away"),
            PresenceStatus::Busy => write!(f, "busy"),
            PresenceStatus::Offline => write!(f, "offline"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "workspace_role", rename_all = "lowercase")]
pub enum WorkspaceRole {
    Owner,
    Admin,
    Member,
    Guest,
}

impl std::fmt::Display for WorkspaceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkspaceRole::Owner => write!(f, "owner"),
            WorkspaceRole::Admin => write!(f, "admin"),
            WorkspaceRole::Member => write!(f, "member"),
            WorkspaceRole::Guest => write!(f, "guest"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "channel_type", rename_all = "lowercase")]
pub enum ChannelType {
    Public,
    Private,
    Announcement,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "channel_member_role", rename_all = "lowercase")]
pub enum ChannelMemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "conversation_type", rename_all = "lowercase")]
pub enum ConversationType {
    Direct,
    Group,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    File,
    System,
    Code,
    LinkPreview,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "invitation_status", rename_all = "lowercase")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_action", rename_all = "snake_case")]
pub enum AuditAction {
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserBanned,
    WorkspaceCreated,
    WorkspaceUpdated,
    WorkspaceDeleted,
    ChannelCreated,
    ChannelUpdated,
    ChannelArchived,
    ChannelDeleted,
    MessageCreated,
    MessageEdited,
    MessageDeleted,
    MemberInvited,
    MemberJoined,
    MemberRemoved,
    MemberRoleChanged,
    FileUploaded,
    FileDeleted,
}

// ============================================================================
// PROFILES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Profile {
    pub id: Uuid,
    pub email: Option<String>,
    pub guest_token: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status_message: Option<String>,
    pub presence: PresenceStatus,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub is_guest: bool,
    pub is_banned: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProfile {
    pub email: Option<String>,
    pub guest_token: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status_message: Option<String>,
    pub is_guest: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status_message: Option<String>,
    pub presence: Option<PresenceStatus>,
}

// ============================================================================
// WORKSPACES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub plan_tier: String,
    pub is_active: bool,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspace {
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub plan_tier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspace {
    pub name: Option<String>,
    pub logo_url: Option<String>,
    pub plan_tier: Option<String>,
    pub is_active: Option<bool>,
    pub settings: Option<serde_json::Value>,
}

// ============================================================================
// WORKSPACE MEMBERS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkspaceMember {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub profile_id: Uuid,
    pub role: WorkspaceRole,
    pub is_active: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceMember {
    pub workspace_id: Uuid,
    pub profile_id: Uuid,
    pub role: WorkspaceRole,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkspaceMember {
    pub role: Option<WorkspaceRole>,
    pub is_active: Option<bool>,
}

// ============================================================================
// WORKSPACE INVITATIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkspaceInvitation {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub email: String,
    pub token: String,
    pub invited_by: Uuid,
    pub role: WorkspaceRole,
    pub status: InvitationStatus,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWorkspaceInvitation {
    pub workspace_id: Uuid,
    pub email: String,
    pub invited_by: Uuid,
    pub role: WorkspaceRole,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInvitation {
    pub token: String,
}

// ============================================================================
// CHANNELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Channel {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub r#type: ChannelType,
    pub is_private: bool,
    pub is_archived: bool,
    pub last_message_at: Option<DateTime<Utc>>,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateChannel {
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub r#type: ChannelType,
    pub is_private: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannel {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub is_archived: Option<bool>,
}

// ============================================================================
// CONVERSATIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub r#type: ConversationType,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversation {
    pub workspace_id: Option<Uuid>,
    pub r#type: ConversationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConversationParticipant {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub profile_id: Uuid,
    pub is_active: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationParticipant {
    pub conversation_id: Uuid,
    pub profile_id: Uuid,
}

// ============================================================================
// MESSAGES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: Option<String>,
    pub r#type: MessageType,
    pub is_edited: bool,
    pub is_deleted: bool,
    pub is_pinned: bool,
    pub pinned_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub edited_at: Option<DateTime<Utc>>,
    pub reply_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessage {
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: Option<String>,
    pub r#type: MessageType,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessage {
    pub content: Option<String>,
    pub is_deleted: Option<bool>,
    pub is_pinned: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// MESSAGE VERSIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageVersion {
    pub id: Uuid,
    pub message_id: Uuid,
    pub old_content: Option<String>,
    pub old_metadata: Option<serde_json::Value>,
    pub edited_at: DateTime<Utc>,
}

// ============================================================================
// MESSAGE ATTACHMENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub file_url: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageAttachment {
    pub message_id: Uuid,
    pub file_url: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
}

// ============================================================================
// MESSAGE REACTIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageReaction {
    pub id: Uuid,
    pub message_id: Uuid,
    pub profile_id: Uuid,
    pub emoji: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMessageReaction {
    pub message_id: Uuid,
    pub profile_id: Uuid,
    pub emoji: String,
}

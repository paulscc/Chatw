use actix_web::{web, HttpResponse, Scope};
use uuid::Uuid;
use serde::Deserialize;

use crate::db::Database;
use crate::error::AppError;
use crate::models::*;
use crate::auth::AuthService;
use crate::workspaces::WorkspaceService;
use crate::channels::ChannelService;
use crate::conversations::ConversationService;
use crate::messages::MessageService;

// ============================================================================
// API ROUTES CONFIGURATION
// ============================================================================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            web::scope("/api")
                .service(health_check)
                .service(profiles_routes())
                .service(workspaces_routes())
                .service(channels_routes())
                .service(conversations_routes())
                .service(messages_routes())
        );
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

#[actix_web::get("/health")]
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "proyect-chat"
    }))
}

// ============================================================================
// PROFILES ROUTES
// ============================================================================

fn profiles_routes() -> Scope {
    web::scope("/profiles")
        .route("", web::post().to(create_profile))
        .route("/{id}", web::get().to(get_profile))
        .route("/{id}", web::put().to(update_profile))
        .route("/{id}", web::delete().to(delete_profile))
        .route("/{id}/presence", web::put().to(update_presence))
        .route("", web::get().to(list_profiles))
}

#[derive(Deserialize)]
struct CreateProfileRequest {
    email: Option<String>,
    guest_token: Option<String>,
    display_name: String,
    avatar_url: Option<String>,
    status_message: Option<String>,
    is_guest: bool,
}

async fn create_profile(
    db: web::Data<Database>,
    req: web::Json<CreateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    let service = AuthService::new(db.get_ref().clone());
    let profile = service.create_profile(CreateProfile {
        email: req.email.clone(),
        guest_token: req.guest_token.clone(),
        display_name: req.display_name.clone(),
        avatar_url: req.avatar_url.clone(),
        status_message: req.status_message.clone(),
        is_guest: req.is_guest,
    }).await?;
    Ok(HttpResponse::Created().json(profile))
}

async fn get_profile(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = AuthService::new(db.get_ref().clone());
    let profile = service.get_profile_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    display_name: Option<String>,
    avatar_url: Option<String>,
    status_message: Option<String>,
    presence: Option<PresenceStatus>,
}

async fn update_profile(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateProfileRequest>,
) -> Result<HttpResponse, AppError> {
    let service = AuthService::new(db.get_ref().clone());
    let profile = service.update_profile(*path, UpdateProfile {
        display_name: req.display_name.clone(),
        avatar_url: req.avatar_url.clone(),
        status_message: req.status_message.clone(),
        presence: req.presence,
    }).await?;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(Deserialize)]
struct UpdatePresenceRequest {
    presence: PresenceStatus,
}

async fn update_presence(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<UpdatePresenceRequest>,
) -> Result<HttpResponse, AppError> {
    let service = AuthService::new(db.get_ref().clone());
    let profile = service.update_presence(*path, req.presence).await?;
    Ok(HttpResponse::Ok().json(profile))
}

async fn delete_profile(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = AuthService::new(db.get_ref().clone());
    service.delete_profile(*path).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_profiles(
    db: web::Data<Database>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let service = AuthService::new(db.get_ref().clone());
    let profiles = service.list_profiles(query.limit.unwrap_or(50), query.offset.unwrap_or(0)).await?;
    Ok(HttpResponse::Ok().json(profiles))
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

// ============================================================================
// WORKSPACES ROUTES
// ============================================================================

fn workspaces_routes() -> Scope {
    web::scope("/workspaces")
        .route("", web::post().to(create_workspace))
        .route("/{id}", web::get().to(get_workspace))
        .route("/{id}", web::put().to(update_workspace))
        .route("/{id}", web::delete().to(delete_workspace))
        .route("", web::get().to(list_workspaces))
        .route("/{id}/members", web::post().to(add_member))
        .route("/{id}/members", web::get().to(list_members))
        .route("/{id}/members/{profile_id}", web::put().to(update_member))
        .route("/{id}/members/{profile_id}", web::delete().to(remove_member))
        .route("/{id}/invitations", web::post().to(create_invitation))
        .route("/{id}/invitations", web::get().to(list_invitations))
        .route("/invitations/{token}/accept", web::post().to(accept_invitation))
        .route("/invitations/{token}/revoke", web::post().to(revoke_invitation))
}

#[derive(Deserialize)]
struct CreateWorkspaceRequest {
    name: String,
    slug: String,
    logo_url: Option<String>,
    plan_tier: Option<String>,
}

async fn create_workspace(
    db: web::Data<Database>,
    req: web::Json<CreateWorkspaceRequest>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let workspace = service.create_workspace(CreateWorkspace {
        name: req.name.clone(),
        slug: req.slug.clone(),
        logo_url: req.logo_url.clone(),
        plan_tier: req.plan_tier.clone(),
    }).await?;
    Ok(HttpResponse::Created().json(workspace))
}

async fn get_workspace(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let workspace = service.get_workspace_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(workspace))
}

#[derive(Deserialize)]
struct UpdateWorkspaceRequest {
    name: Option<String>,
    logo_url: Option<String>,
    plan_tier: Option<String>,
    is_active: Option<bool>,
    settings: Option<serde_json::Value>,
}

async fn update_workspace(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateWorkspaceRequest>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let workspace = service.update_workspace(*path, UpdateWorkspace {
        name: req.name.clone(),
        logo_url: req.logo_url.clone(),
        plan_tier: req.plan_tier.clone(),
        is_active: req.is_active,
        settings: req.settings.clone(),
    }).await?;
    Ok(HttpResponse::Ok().json(workspace))
}

async fn delete_workspace(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    service.delete_workspace(*path).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_workspaces(
    db: web::Data<Database>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let workspaces = service.list_workspaces(query.limit.unwrap_or(50), query.offset.unwrap_or(0)).await?;
    Ok(HttpResponse::Ok().json(workspaces))
}

#[derive(Deserialize)]
struct AddMemberRequest {
    profile_id: Uuid,
    role: WorkspaceRole,
}

async fn add_member(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<AddMemberRequest>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let member = service.add_member(CreateWorkspaceMember {
        workspace_id: *path,
        profile_id: req.profile_id,
        role: req.role,
    }).await?;
    Ok(HttpResponse::Created().json(member))
}

async fn list_members(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let members = service.list_workspace_members(*path).await?;
    Ok(HttpResponse::Ok().json(members))
}

#[derive(Deserialize)]
struct UpdateMemberRequest {
    role: Option<WorkspaceRole>,
    is_active: Option<bool>,
}

async fn update_member(
    db: web::Data<Database>,
    path: web::Path<(Uuid, Uuid)>,
    req: web::Json<UpdateMemberRequest>,
) -> Result<HttpResponse, AppError> {
    let (workspace_id, profile_id) = *path;
    let service = WorkspaceService::new(db.get_ref().clone());
    let member = service.update_member(workspace_id, profile_id, UpdateWorkspaceMember {
        role: req.role,
        is_active: req.is_active,
    }).await?;
    Ok(HttpResponse::Ok().json(member))
}

async fn remove_member(
    db: web::Data<Database>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (workspace_id, profile_id) = *path;
    let service = WorkspaceService::new(db.get_ref().clone());
    service.remove_member(workspace_id, profile_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
struct CreateInvitationRequest {
    email: String,
    invited_by: Uuid,
    role: WorkspaceRole,
    expires_at: chrono::DateTime<chrono::Utc>,
}

async fn create_invitation(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<CreateInvitationRequest>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let invitation = service.create_invitation(CreateWorkspaceInvitation {
        workspace_id: *path,
        email: req.email.clone(),
        invited_by: req.invited_by,
        role: req.role,
        expires_at: req.expires_at,
    }).await?;
    Ok(HttpResponse::Created().json(invitation))
}

async fn list_invitations(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let invitations = service.list_invitations(*path).await?;
    Ok(HttpResponse::Ok().json(invitations))
}

#[derive(Deserialize)]
struct AcceptInvitationRequest {
    profile_id: Uuid,
}

async fn accept_invitation(
    db: web::Data<Database>,
    path: web::Path<String>,
    req: web::Json<AcceptInvitationRequest>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let invitation = service.accept_invitation(&path, req.profile_id).await?;
    Ok(HttpResponse::Ok().json(invitation))
}

async fn revoke_invitation(
    db: web::Data<Database>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let service = WorkspaceService::new(db.get_ref().clone());
    let invitation = service.revoke_invitation(&path).await?;
    Ok(HttpResponse::Ok().json(invitation))
}

// ============================================================================
// CHANNELS ROUTES
// ============================================================================

fn channels_routes() -> Scope {
    web::scope("/channels")
        .route("", web::post().to(create_channel))
        .route("/{id}", web::get().to(get_channel))
        .route("/{id}", web::put().to(update_channel))
        .route("/{id}", web::delete().to(delete_channel))
        .route("/{id}/archive", web::post().to(archive_channel))
        .route("/{id}/unarchive", web::post().to(unarchive_channel))
        .route("/workspace/{workspace_id}", web::get().to(list_workspace_channels))
        .route("/workspace/{workspace_id}/public", web::get().to(list_public_channels))
}

#[derive(Deserialize)]
struct CreateChannelRequest {
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    r#type: ChannelType,
    is_private: Option<bool>,
}

async fn create_channel(
    db: web::Data<Database>,
    req: web::Json<CreateChannelRequest>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channel = service.create_channel(CreateChannel {
        workspace_id: req.workspace_id,
        name: req.name.clone(),
        description: req.description.clone(),
        r#type: req.r#type,
        is_private: req.is_private,
    }).await?;
    Ok(HttpResponse::Created().json(channel))
}

async fn get_channel(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channel = service.get_channel_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(channel))
}

#[derive(Deserialize)]
struct UpdateChannelRequest {
    name: Option<String>,
    description: Option<String>,
    is_private: Option<bool>,
    is_archived: Option<bool>,
}

async fn update_channel(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateChannelRequest>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channel = service.update_channel(*path, UpdateChannel {
        name: req.name.clone(),
        description: req.description.clone(),
        is_private: req.is_private,
        is_archived: req.is_archived,
    }).await?;
    Ok(HttpResponse::Ok().json(channel))
}

async fn archive_channel(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channel = service.archive_channel(*path).await?;
    Ok(HttpResponse::Ok().json(channel))
}

async fn unarchive_channel(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channel = service.unarchive_channel(*path).await?;
    Ok(HttpResponse::Ok().json(channel))
}

async fn delete_channel(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    service.delete_channel(*path).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_workspace_channels(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channels = service.list_workspace_channels(*path).await?;
    Ok(HttpResponse::Ok().json(channels))
}

async fn list_public_channels(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ChannelService::new(db.get_ref().clone());
    let channels = service.list_public_channels(*path).await?;
    Ok(HttpResponse::Ok().json(channels))
}

// ============================================================================
// CONVERSATIONS ROUTES
// ============================================================================

fn conversations_routes() -> Scope {
    web::scope("/conversations")
        .route("", web::post().to(create_conversation))
        .route("/{id}", web::get().to(get_conversation))
        .route("/{id}", web::delete().to(delete_conversation))
        .route("/workspace/{workspace_id}", web::get().to(list_workspace_conversations))
        .route("/user/{profile_id}", web::get().to(list_user_conversations))
        .route("/{id}/participants", web::post().to(add_participant))
        .route("/{id}/participants", web::get().to(list_participants))
        .route("/{id}/participants/{profile_id}", web::delete().to(remove_participant))
        .route("/direct/{profile1}/{profile2}", web::get().to(get_or_create_direct))
}

#[derive(Deserialize)]
struct CreateConversationRequest {
    workspace_id: Option<Uuid>,
    r#type: ConversationType,
}

async fn create_conversation(
    db: web::Data<Database>,
    req: web::Json<CreateConversationRequest>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    let conv = service.create_conversation(CreateConversation {
        workspace_id: req.workspace_id,
        r#type: req.r#type,
    }).await?;
    Ok(HttpResponse::Created().json(conv))
}

async fn get_conversation(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    let conv = service.get_conversation_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(conv))
}

async fn delete_conversation(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    service.delete_conversation(*path).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_workspace_conversations(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    let convs = service.list_workspace_conversations(*path).await?;
    Ok(HttpResponse::Ok().json(convs))
}

async fn list_user_conversations(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    let convs = service.list_user_conversations(*path).await?;
    Ok(HttpResponse::Ok().json(convs))
}

#[derive(Deserialize)]
struct AddParticipantRequest {
    profile_id: Uuid,
}

async fn add_participant(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<AddParticipantRequest>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    let participant = service.add_participant(CreateConversationParticipant {
        conversation_id: *path,
        profile_id: req.profile_id,
    }).await?;
    Ok(HttpResponse::Created().json(participant))
}

async fn list_participants(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = ConversationService::new(db.get_ref().clone());
    let participants = service.list_conversation_participants(*path).await?;
    Ok(HttpResponse::Ok().json(participants))
}

async fn remove_participant(
    db: web::Data<Database>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (conversation_id, profile_id) = *path;
    let service = ConversationService::new(db.get_ref().clone());
    service.remove_participant(conversation_id, profile_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn get_or_create_direct(
    db: web::Data<Database>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, AppError> {
    let (profile1, profile2) = *path;
    let service = ConversationService::new(db.get_ref().clone());
    let conv = service.get_or_create_direct_conversation(profile1, profile2).await?;
    Ok(HttpResponse::Ok().json(conv))
}

// ============================================================================
// MESSAGES ROUTES
// ============================================================================

fn messages_routes() -> Scope {
    web::scope("/messages")
        .route("", web::post().to(create_message))
        .route("/{id}", web::get().to(get_message))
        .route("/{id}", web::put().to(update_message))
        .route("/{id}", web::delete().to(delete_message))
        .route("/{id}/pin", web::post().to(pin_message))
        .route("/{id}/unpin", web::post().to(unpin_message))
        .route("/{id}/versions", web::get().to(get_message_versions))
        .route("/{id}/attachments", web::post().to(add_attachment))
        .route("/{id}/attachments", web::get().to(get_attachments))
        .route("/attachments/{attachment_id}", web::delete().to(delete_attachment))
        .route("/{id}/reactions", web::post().to(add_reaction))
        .route("/{id}/reactions", web::get().to(get_reactions))
        .route("/{id}/reactions/{profile_id}/{emoji}", web::delete().to(remove_reaction))
        .route("/channel/{channel_id}", web::get().to(list_channel_messages))
        .route("/conversation/{conversation_id}", web::get().to(list_conversation_messages))
        .route("/thread/{parent_id}", web::get().to(list_thread_messages))
        .route("/search/{workspace_id}", web::get().to(search_messages))
}

#[derive(Deserialize)]
struct CreateMessageRequest {
    channel_id: Option<Uuid>,
    conversation_id: Option<Uuid>,
    sender_id: Uuid,
    parent_id: Option<Uuid>,
    content: Option<String>,
    r#type: MessageType,
    metadata: Option<serde_json::Value>,
}

async fn create_message(
    db: web::Data<Database>,
    req: web::Json<CreateMessageRequest>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let message = service.create_message(CreateMessage {
        channel_id: req.channel_id,
        conversation_id: req.conversation_id,
        sender_id: req.sender_id,
        parent_id: req.parent_id,
        content: req.content.clone(),
        r#type: req.r#type,
        metadata: req.metadata.clone(),
    }).await?;
    Ok(HttpResponse::Created().json(message))
}

async fn get_message(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let message = service.get_message_by_id(*path).await?;
    Ok(HttpResponse::Ok().json(message))
}

#[derive(Deserialize)]
struct UpdateMessageRequest {
    content: Option<String>,
    is_deleted: Option<bool>,
    is_pinned: Option<bool>,
    metadata: Option<serde_json::Value>,
}

async fn update_message(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateMessageRequest>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let message = service.update_message(*path, UpdateMessage {
        content: req.content.clone(),
        is_deleted: req.is_deleted,
        is_pinned: req.is_pinned,
        metadata: req.metadata.clone(),
    }).await?;
    Ok(HttpResponse::Ok().json(message))
}

async fn delete_message(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let message = service.delete_message(*path).await?;
    Ok(HttpResponse::Ok().json(message))
}

async fn pin_message(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let message = service.pin_message(*path).await?;
    Ok(HttpResponse::Ok().json(message))
}

async fn unpin_message(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let message = service.unpin_message(*path).await?;
    Ok(HttpResponse::Ok().json(message))
}

async fn get_message_versions(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let versions = service.get_message_versions(*path).await?;
    Ok(HttpResponse::Ok().json(versions))
}

#[derive(Deserialize)]
struct AddAttachmentRequest {
    file_url: String,
    file_name: String,
    file_type: String,
    file_size: i64,
}

async fn add_attachment(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<AddAttachmentRequest>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let attachment = service.add_attachment(CreateMessageAttachment {
        message_id: *path,
        file_url: req.file_url.clone(),
        file_name: req.file_name.clone(),
        file_type: req.file_type.clone(),
        file_size: req.file_size,
    }).await?;
    Ok(HttpResponse::Created().json(attachment))
}

async fn get_attachments(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let attachments = service.get_message_attachments(*path).await?;
    Ok(HttpResponse::Ok().json(attachments))
}

async fn delete_attachment(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    service.delete_attachment(*path).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
struct AddReactionRequest {
    profile_id: Uuid,
    emoji: String,
}

async fn add_reaction(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    req: web::Json<AddReactionRequest>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let reaction = service.add_reaction(CreateMessageReaction {
        message_id: *path,
        profile_id: req.profile_id,
        emoji: req.emoji.clone(),
    }).await?;
    Ok(HttpResponse::Created().json(reaction))
}

async fn get_reactions(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let reactions = service.get_message_reactions(*path).await?;
    Ok(HttpResponse::Ok().json(reactions))
}

async fn remove_reaction(
    db: web::Data<Database>,
    path: web::Path<(Uuid, Uuid, String)>,
) -> Result<HttpResponse, AppError> {
    let (message_id, profile_id, emoji) = path.into_inner();
    let service = MessageService::new(db.get_ref().clone());
    service.remove_reaction(message_id, profile_id, &emoji).await?;
    Ok(HttpResponse::NoContent().finish())
}

async fn list_channel_messages(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let messages = service.list_channel_messages(*path, query.limit.unwrap_or(50), query.offset.unwrap_or(0)).await?;
    Ok(HttpResponse::Ok().json(messages))
}

async fn list_conversation_messages(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let messages = service.list_conversation_messages(*path, query.limit.unwrap_or(50), query.offset.unwrap_or(0)).await?;
    Ok(HttpResponse::Ok().json(messages))
}

async fn list_thread_messages(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let messages = service.list_thread_messages(*path, query.limit.unwrap_or(50), query.offset.unwrap_or(0)).await?;
    Ok(HttpResponse::Ok().json(messages))
}

async fn search_messages(
    db: web::Data<Database>,
    path: web::Path<Uuid>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse, AppError> {
    let service = MessageService::new(db.get_ref().clone());
    let messages = service.search_messages(*path, &query.q, query.limit.unwrap_or(20)).await?;
    Ok(HttpResponse::Ok().json(messages))
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<i64>,
}

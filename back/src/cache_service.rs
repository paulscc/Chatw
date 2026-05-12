use crate::redis_client::RedisClient;
use crate::models::{Profile, Channel, Message, Workspace};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProfile {
    pub id: String,
    pub email: Option<String>,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status_message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedChannel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub workspace_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMessage {
    pub id: String,
    pub content: String,
    pub channel_id: String,
    pub user_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWorkspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct CacheService {
    redis_client: RedisClient,
}

impl CacheService {
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }

    fn profile_key(profile_id: &str) -> String {
        format!("profile:{}", profile_id)
    }

    fn channel_key(channel_id: &str) -> String {
        format!("channel:{}", channel_id)
    }

    fn message_key(message_id: &str) -> String {
        format!("message:{}", message_id)
    }

    fn workspace_key(workspace_id: &str) -> String {
        format!("workspace:{}", workspace_id)
    }

    fn profile_channels_key(profile_id: &str) -> String {
        format!("profile:{}:channels", profile_id)
    }

    fn channel_messages_key(channel_id: &str) -> String {
        format!("channel:{}:messages", channel_id)
    }

    fn workspace_channels_key(workspace_id: &str) -> String {
        format!("workspace:{}:channels", workspace_id)
    }

    fn profile_workspaces_key(profile_id: &str) -> String {
        format!("profile:{}:workspaces", profile_id)
    }

    pub async fn cache_profile(&self, profile: &Profile, ttl_hours: u64) -> redis::RedisResult<()> {
        let cached_profile = CachedProfile {
            id: profile.id.to_string(),
            email: profile.email.clone(),
            display_name: profile.display_name.clone(),
            avatar_url: profile.avatar_url.clone(),
            status_message: profile.status_message.clone(),
            created_at: profile.created_at.to_rfc3339(),
        };

        let ttl_seconds = ttl_hours * 3600;
        self.redis_client
            .set(&Self::profile_key(&profile.id.to_string()), &cached_profile, Some(ttl_seconds))
            .await
    }

    pub async fn get_cached_profile(&self, profile_id: &str) -> redis::RedisResult<Option<CachedProfile>> {
        self.redis_client.get(&Self::profile_key(profile_id)).await
    }

    pub async fn cache_channel(&self, channel: &Channel, ttl_hours: u64) -> redis::RedisResult<()> {
        let cached_channel = CachedChannel {
            id: channel.id.to_string(),
            name: channel.name.clone(),
            description: channel.description.clone(),
            workspace_id: channel.workspace_id.to_string(),
            created_at: channel.created_at.to_rfc3339(),
        };

        let ttl_seconds = ttl_hours * 3600;
        self.redis_client
            .set(&Self::channel_key(&channel.id.to_string()), &cached_channel, Some(ttl_seconds))
            .await
    }

    pub async fn get_cached_channel(&self, channel_id: &str) -> redis::RedisResult<Option<CachedChannel>> {
        self.redis_client.get(&Self::channel_key(channel_id)).await
    }

    pub async fn cache_message(&self, message: &Message, ttl_hours: u64) -> redis::RedisResult<()> {
        let cached_message = CachedMessage {
            id: message.id.to_string(),
            content: message.content.clone().unwrap_or_default(),
            channel_id: message.channel_id.map(|id| id.to_string()).unwrap_or_default(),
            user_id: message.sender_id.to_string(),
            created_at: message.created_at.to_rfc3339(),
        };

        let ttl_seconds = ttl_hours * 3600;
        self.redis_client
            .set(&Self::message_key(&message.id.to_string()), &cached_message, Some(ttl_seconds))
            .await
    }

    pub async fn get_cached_message(&self, message_id: &str) -> redis::RedisResult<Option<CachedMessage>> {
        self.redis_client.get(&Self::message_key(message_id)).await
    }

    pub async fn cache_workspace(&self, workspace: &Workspace, ttl_hours: u64) -> redis::RedisResult<()> {
        let cached_workspace = CachedWorkspace {
            id: workspace.id.to_string(),
            name: workspace.name.clone(),
            description: None, // Workspace doesn't have description field
            created_at: workspace.created_at.to_rfc3339(),
        };

        let ttl_seconds = ttl_hours * 3600;
        self.redis_client
            .set(&Self::workspace_key(&workspace.id.to_string()), &cached_workspace, Some(ttl_seconds))
            .await
    }

    pub async fn get_cached_workspace(&self, workspace_id: &str) -> redis::RedisResult<Option<CachedWorkspace>> {
        self.redis_client.get(&Self::workspace_key(workspace_id)).await
    }

    pub async fn add_profile_to_channel(&self, profile_id: &str, channel_id: &str) -> redis::RedisResult<bool> {
        self.redis_client
            .add_to_set(&Self::profile_channels_key(profile_id), &channel_id)
            .await
    }

    pub async fn remove_profile_from_channel(&self, profile_id: &str, channel_id: &str) -> redis::RedisResult<bool> {
        self.redis_client
            .remove_from_set(&Self::profile_channels_key(profile_id), &channel_id)
            .await
    }

    pub async fn is_profile_in_channel(&self, profile_id: &str, channel_id: &str) -> redis::RedisResult<bool> {
        self.redis_client
            .is_in_set(&Self::profile_channels_key(profile_id), &channel_id)
            .await
    }

    pub async fn add_message_to_channel(&self, channel_id: &str, message_id: &str) -> redis::RedisResult<bool> {
        self.redis_client
            .add_to_set(&Self::channel_messages_key(channel_id), &message_id)
            .await
    }

    pub async fn add_channel_to_workspace(&self, workspace_id: &str, channel_id: &str) -> redis::RedisResult<bool> {
        self.redis_client
            .add_to_set(&Self::workspace_channels_key(workspace_id), &channel_id)
            .await
    }

    pub async fn add_profile_to_workspace(&self, profile_id: &str, workspace_id: &str) -> redis::RedisResult<bool> {
        self.redis_client
            .add_to_set(&Self::profile_workspaces_key(profile_id), &workspace_id)
            .await
    }

    pub async fn invalidate_profile_cache(&self, profile_id: &str) -> redis::RedisResult<()> {
        let profile_key = Self::profile_key(profile_id);
        let profile_channels_key = Self::profile_channels_key(profile_id);
        let profile_workspaces_key = Self::profile_workspaces_key(profile_id);

        self.redis_client.delete(&profile_key).await?;
        self.redis_client.delete(&profile_channels_key).await?;
        self.redis_client.delete(&profile_workspaces_key).await?;

        Ok(())
    }

    pub async fn invalidate_channel_cache(&self, channel_id: &str) -> redis::RedisResult<()> {
        let channel_key = Self::channel_key(channel_id);
        let channel_messages_key = Self::channel_messages_key(channel_id);

        self.redis_client.delete(&channel_key).await?;
        self.redis_client.delete(&channel_messages_key).await?;

        Ok(())
    }

    pub async fn invalidate_workspace_cache(&self, workspace_id: &str) -> redis::RedisResult<()> {
        let workspace_key = Self::workspace_key(workspace_id);
        let workspace_channels_key = Self::workspace_channels_key(workspace_id);

        self.redis_client.delete(&workspace_key).await?;
        self.redis_client.delete(&workspace_channels_key).await?;

        Ok(())
    }

    pub async fn set_session(&self, session_id: &str, user_id: &str, ttl_hours: u64) -> redis::RedisResult<()> {
        let ttl_seconds = ttl_hours * 3600;
        let session_key = format!("session:{}", session_id);
        self.redis_client
            .set(&session_key, &user_id, Some(ttl_seconds))
            .await
    }

    pub async fn get_session_user(&self, session_id: &str) -> redis::RedisResult<Option<String>> {
        let session_key = format!("session:{}", session_id);
        self.redis_client.get(&session_key).await
    }

    pub async fn invalidate_session(&self, session_id: &str) -> redis::RedisResult<bool> {
        let session_key = format!("session:{}", session_id);
        self.redis_client.delete(&session_key).await
    }

    pub async fn increment_api_rate_limit(&self, profile_id: &str, window_seconds: u64) -> redis::RedisResult<i64> {
        let rate_limit_key = format!("rate_limit:{}", profile_id);
        let count = self.redis_client.increment(&rate_limit_key).await?;
        
        if count == 1 {
            self.redis_client.expire(&rate_limit_key, window_seconds).await?;
        }
        
        Ok(count)
    }

    pub async fn set_profile_online_status(&self, profile_id: &str, online: bool) -> redis::RedisResult<()> {
        let status_key = format!("online_status:{}", profile_id);
        let ttl_seconds = 300; // 5 minutes
        
        if online {
            self.redis_client.set(&status_key, &true, Some(ttl_seconds)).await?;
        } else {
            self.redis_client.delete(&status_key).await?;
        }
        
        Ok(())
    }

    pub async fn is_profile_online(&self, profile_id: &str) -> redis::RedisResult<bool> {
        let status_key = format!("online_status:{}", profile_id);
        self.redis_client.exists(&status_key).await
    }
}

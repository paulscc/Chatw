use crate::db::Database;
use crate::error::AppError;
use crate::models::{
    Workspace, CreateWorkspace, UpdateWorkspace,
    WorkspaceMember, CreateWorkspaceMember, UpdateWorkspaceMember,
    WorkspaceInvitation, CreateWorkspaceInvitation, InvitationStatus
};
use uuid::Uuid;
use chrono::Utc;

pub struct WorkspaceService {
    db: Database,
}

impl WorkspaceService {
    pub fn new(db: Database) -> Self {
        WorkspaceService { db }
    }

    // ==================== WORKSPACES ====================

    pub async fn create_workspace(&self, workspace: CreateWorkspace) -> Result<Workspace, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let plan_tier = workspace.plan_tier.unwrap_or_else(|| "free".to_string());

        let workspace = sqlx::query_as::<_, Workspace>(
            r#"
            INSERT INTO workspaces (id, name, slug, logo_url, plan_tier, is_active, settings, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, true, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(&workspace.name)
        .bind(&workspace.slug)
        .bind(&workspace.logo_url)
        .bind(&plan_tier)
        .bind(serde_json::json!({}))
        .bind(now)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(workspace)
    }

    pub async fn get_workspace_by_id(&self, id: Uuid) -> Result<Workspace, AppError> {
        let workspace = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", id)))?;

        Ok(workspace)
    }

    pub async fn get_workspace_by_slug(&self, slug: &str) -> Result<Workspace, AppError> {
        let workspace = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE slug = $1"
        )
        .bind(slug)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Workspace with slug {} not found", slug)))?;

        Ok(workspace)
    }

    pub async fn update_workspace(&self, id: Uuid, update: UpdateWorkspace) -> Result<Workspace, AppError> {
        let mut query = String::from("UPDATE workspaces SET updated_at = NOW()");
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(name) = &update.name {
            param_count += 1;
            query.push_str(&format!(", name = ${}", param_count));
            params.push(name.clone());
        }

        if let Some(logo_url) = &update.logo_url {
            param_count += 1;
            query.push_str(&format!(", logo_url = ${}", param_count));
            params.push(logo_url.clone());
        }

        if let Some(plan_tier) = &update.plan_tier {
            param_count += 1;
            query.push_str(&format!(", plan_tier = ${}", param_count));
            params.push(plan_tier.clone());
        }

        if let Some(is_active) = update.is_active {
            param_count += 1;
            query.push_str(&format!(", is_active = ${}", param_count));
            params.push(is_active.to_string());
        }

        if let Some(settings) = update.settings {
            param_count += 1;
            query.push_str(&format!(", settings = ${}", param_count));
            params.push(settings.to_string());
        }

        param_count += 1;
        query.push_str(&format!(" WHERE id = ${} RETURNING *", param_count));

        let mut query_builder = sqlx::query_as::<_, Workspace>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(id);

        let workspace = query_builder
            .fetch_one(&self.db.pool)
            .await?;

        Ok(workspace)
    }

    pub async fn delete_workspace(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM workspaces WHERE id = $1")
            .bind(id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn list_workspaces(&self, limit: i64, offset: i64) -> Result<Vec<Workspace>, AppError> {
        let workspaces = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(workspaces)
    }

    // ==================== WORKSPACE MEMBERS ====================

    pub async fn add_member(&self, member: CreateWorkspaceMember) -> Result<WorkspaceMember, AppError> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let member = sqlx::query_as::<_, WorkspaceMember>(
            r#"
            INSERT INTO workspace_members (id, workspace_id, profile_id, role, is_active, joined_at)
            VALUES ($1, $2, $3, $4, true, $5)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(member.workspace_id)
        .bind(member.profile_id)
        .bind(member.role)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(member)
    }

    pub async fn get_member(&self, workspace_id: Uuid, profile_id: Uuid) -> Result<WorkspaceMember, AppError> {
        let member = sqlx::query_as::<_, WorkspaceMember>(
            "SELECT * FROM workspace_members WHERE workspace_id = $1 AND profile_id = $2"
        )
        .bind(workspace_id)
        .bind(profile_id)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Member not found".to_string()))?;

        Ok(member)
    }

    pub async fn update_member(&self, workspace_id: Uuid, profile_id: Uuid, update: UpdateWorkspaceMember) -> Result<WorkspaceMember, AppError> {
        let mut query = String::from("UPDATE workspace_members SET");
        let mut params = Vec::new();
        let mut param_count = 0;
        let mut has_update = false;

        if let Some(role) = update.role {
            param_count += 1;
            query.push_str(&format!(" role = ${}", param_count));
            params.push(role.to_string());
            has_update = true;
        }

        if let Some(is_active) = update.is_active {
            if has_update {
                query.push_str(",");
            }
            param_count += 1;
            query.push_str(&format!(" is_active = ${}", param_count));
            params.push(is_active.to_string());
            has_update = true;
        }

        if !has_update {
            return self.get_member(workspace_id, profile_id).await;
        }

        param_count += 1;
        query.push_str(&format!(" WHERE workspace_id = ${} AND profile_id = ${} RETURNING *", param_count, param_count + 1));

        let mut query_builder = sqlx::query_as::<_, WorkspaceMember>(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }
        query_builder = query_builder.bind(workspace_id);
        query_builder = query_builder.bind(profile_id);

        let member = query_builder
            .fetch_one(&self.db.pool)
            .await?;

        Ok(member)
    }

    pub async fn remove_member(&self, workspace_id: Uuid, profile_id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM workspace_members WHERE workspace_id = $1 AND profile_id = $2")
            .bind(workspace_id)
            .bind(profile_id)
            .execute(&self.db.pool)
            .await?;

        Ok(())
    }

    pub async fn list_workspace_members(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceMember>, AppError> {
        let members = sqlx::query_as::<_, WorkspaceMember>(
            "SELECT * FROM workspace_members WHERE workspace_id = $1 ORDER BY joined_at ASC"
        )
        .bind(workspace_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(members)
    }

    // ==================== WORKSPACE INVITATIONS ====================

    pub async fn create_invitation(&self, invitation: CreateWorkspaceInvitation) -> Result<WorkspaceInvitation, AppError> {
        let id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();

        let invitation = sqlx::query_as::<_, WorkspaceInvitation>(
            r#"
            INSERT INTO workspace_invitations (id, workspace_id, email, token, invited_by, role, status, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(invitation.workspace_id)
        .bind(&invitation.email)
        .bind(&token)
        .bind(invitation.invited_by)
        .bind(invitation.role)
        .bind(InvitationStatus::Pending)
        .bind(invitation.expires_at)
        .bind(now)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(invitation)
    }

    pub async fn get_invitation_by_token(&self, token: &str) -> Result<WorkspaceInvitation, AppError> {
        let invitation = sqlx::query_as::<_, WorkspaceInvitation>(
            "SELECT * FROM workspace_invitations WHERE token = $1"
        )
        .bind(token)
        .fetch_optional(&self.db.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Invitation not found".to_string()))?;

        Ok(invitation)
    }

    pub async fn accept_invitation(&self, token: &str, profile_id: Uuid) -> Result<WorkspaceInvitation, AppError> {
        let invitation = self.get_invitation_by_token(token).await?;

        if invitation.status != InvitationStatus::Pending {
            return Err(AppError::Conflict("Invitation is not pending".to_string()));
        }

        if invitation.expires_at < Utc::now() {
            return Err(AppError::Conflict("Invitation has expired".to_string()));
        }

        // Add the member
        self.add_member(CreateWorkspaceMember {
            workspace_id: invitation.workspace_id,
            profile_id,
            role: invitation.role,
        }).await?;

        // Update invitation status
        let updated = sqlx::query_as::<_, WorkspaceInvitation>(
            "UPDATE workspace_invitations SET status = $1 WHERE token = $2 RETURNING *"
        )
        .bind(InvitationStatus::Accepted)
        .bind(token)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(updated)
    }

    pub async fn revoke_invitation(&self, token: &str) -> Result<WorkspaceInvitation, AppError> {
        let invitation = sqlx::query_as::<_, WorkspaceInvitation>(
            "UPDATE workspace_invitations SET status = $1 WHERE token = $2 RETURNING *"
        )
        .bind(InvitationStatus::Revoked)
        .bind(token)
        .fetch_one(&self.db.pool)
        .await?;

        Ok(invitation)
    }

    pub async fn list_invitations(&self, workspace_id: Uuid) -> Result<Vec<WorkspaceInvitation>, AppError> {
        let invitations = sqlx::query_as::<_, WorkspaceInvitation>(
            "SELECT * FROM workspace_invitations WHERE workspace_id = $1 ORDER BY created_at DESC"
        )
        .bind(workspace_id)
        .fetch_all(&self.db.pool)
        .await?;

        Ok(invitations)
    }
}

-- ============================================================
-- ENTERPRISE CHAT SYSTEM — PostgreSQL Schema (Consolidated)
-- Auth: Supabase Auth (auth.users)
-- Optimization: Full-Text Search, Message Versioning, Multi-tenant
-- ============================================================

-- ─────────────────────────────────────────
-- EXTENSIONES
-- ─────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";   
CREATE EXTENSION IF NOT EXISTS "btree_gin"; 

-- ─────────────────────────────────────────
-- ENUMS
-- ─────────────────────────────────────────
CREATE TYPE presence_status      AS ENUM ('online', 'away', 'busy', 'offline');
CREATE TYPE workspace_role       AS ENUM ('owner', 'admin', 'member', 'guest');
CREATE TYPE channel_type         AS ENUM ('public', 'private', 'announcement');
CREATE TYPE channel_member_role  AS ENUM ('owner', 'admin', 'member');
CREATE TYPE conversation_type    AS ENUM ('direct', 'group');
CREATE TYPE message_type         AS ENUM ('text', 'image', 'file', 'system', 'code', 'link_preview');
CREATE TYPE invitation_status    AS ENUM ('pending', 'accepted', 'expired', 'revoked');
CREATE TYPE audit_action         AS ENUM (
  'user.created', 'user.updated', 'user.deleted', 'user.banned',
  'workspace.created', 'workspace.updated', 'workspace.deleted',
  'channel.created', 'channel.updated', 'channel.archived', 'channel.deleted',
  'message.created', 'message.edited', 'message.deleted',
  'member.invited', 'member.joined', 'member.removed', 'member.role_changed',
  'file.uploaded', 'file.deleted'
);

-- ─────────────────────────────────────────
-- TABLA: profiles
-- ─────────────────────────────────────────
CREATE TABLE profiles (
  id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  email           TEXT UNIQUE,
  guest_token     TEXT UNIQUE,
  display_name    TEXT NOT NULL CHECK (char_length(display_name) BETWEEN 1 AND 80),
  avatar_url      TEXT,
  status_message  TEXT CHECK (char_length(status_message) <= 200),
  presence        presence_status NOT NULL DEFAULT 'offline',
  last_seen_at    TIMESTAMPTZ,
  is_guest        BOOLEAN NOT NULL DEFAULT false,
  is_banned       BOOLEAN NOT NULL DEFAULT false,
  metadata        JSONB NOT NULL DEFAULT '{}',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  CONSTRAINT chk_identity CHECK (
    (email IS NOT NULL AND guest_token IS NULL AND is_guest = false)
    OR
    (email IS NULL AND guest_token IS NOT NULL AND is_guest = true)
  )
);

-- ─────────────────────────────────────────
-- TABLA: workspaces
-- ─────────────────────────────────────────
CREATE TABLE workspaces (
  id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name         TEXT NOT NULL CHECK (char_length(name) BETWEEN 2 AND 120),
  slug         TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9-]{2,60}$'),
  logo_url     TEXT,
  plan_tier    TEXT NOT NULL DEFAULT 'free' CHECK (plan_tier IN ('free', 'pro', 'business', 'enterprise')),
  is_active    BOOLEAN NOT NULL DEFAULT true,
  settings     JSONB NOT NULL DEFAULT '{}',
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────
-- TABLA: workspace_members
-- ─────────────────────────────────────────
CREATE TABLE workspace_members (
  id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workspace_id    UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  profile_id      UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  role            workspace_role NOT NULL DEFAULT 'member',
  is_active       BOOLEAN NOT NULL DEFAULT true,
  joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (workspace_id, profile_id)
);

-- ─────────────────────────────────────────
-- TABLA: workspace_invitations
-- ─────────────────────────────────────────
CREATE TABLE workspace_invitations (
  id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workspace_id  UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  email         TEXT NOT NULL,
  token         TEXT UNIQUE NOT NULL,
  invited_by    UUID NOT NULL REFERENCES profiles(id),
  role          workspace_role NOT NULL DEFAULT 'member',
  status        invitation_status NOT NULL DEFAULT 'pending',
  expires_at    TIMESTAMPTZ NOT NULL,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────
-- TABLA: channels
-- ─────────────────────────────────────────
CREATE TABLE channels (
  id             UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workspace_id   UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  name           TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
  description    TEXT CHECK (char_length(description) <= 500),
  type           channel_type NOT NULL DEFAULT 'public',
  is_private     BOOLEAN NOT NULL DEFAULT false,
  is_archived    BOOLEAN NOT NULL DEFAULT false,
  last_message_at TIMESTAMPTZ,
  message_count  BIGINT NOT NULL DEFAULT 0,
  created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (workspace_id, name)
);

-- ─────────────────────────────────────────
-- TABLA: conversations (DMs / Groups)
-- ─────────────────────────────────────────
CREATE TABLE conversations (
  id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  workspace_id    UUID REFERENCES workspaces(id) ON DELETE CASCADE,
  type            conversation_type NOT NULL DEFAULT 'direct',
  last_message_at TIMESTAMPTZ,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE conversation_participants (
  id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  conversation_id     UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  profile_id          UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  is_active           BOOLEAN NOT NULL DEFAULT true,
  joined_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (conversation_id, profile_id)
);

-- ─────────────────────────────────────────
-- TABLA: messages
-- ─────────────────────────────────────────
CREATE TABLE messages (
  id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  channel_id       UUID REFERENCES channels(id) ON DELETE CASCADE,
  conversation_id  UUID REFERENCES conversations(id) ON DELETE CASCADE,
  sender_id        UUID NOT NULL REFERENCES profiles(id) ON DELETE RESTRICT,
  parent_id        UUID REFERENCES messages(id) ON DELETE CASCADE,
  content          TEXT,
  type             message_type NOT NULL DEFAULT 'text',
  is_edited        BOOLEAN NOT NULL DEFAULT false,
  is_deleted       BOOLEAN NOT NULL DEFAULT false,
  is_pinned        BOOLEAN NOT NULL DEFAULT false,
  pinned_at        TIMESTAMPTZ,
  deleted_at       TIMESTAMPTZ,
  edited_at        TIMESTAMPTZ,
  reply_count      INT NOT NULL DEFAULT 0,
  metadata         JSONB NOT NULL DEFAULT '{}',
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  
  -- Columna generada para búsqueda rápida
  fts_weighted     tsvector GENERATED ALWAYS AS (to_tsvector('spanish', coalesce(content, ''))) STORED,
  
  CONSTRAINT chk_target CHECK (
    (channel_id IS NOT NULL AND conversation_id IS NULL) OR (channel_id IS NULL AND conversation_id IS NOT NULL)
  )
);

-- ─────────────────────────────────────────
-- TABLA: message_versions (Auditoría de cambios)
-- ─────────────────────────────────────────
CREATE TABLE message_versions (
  id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  message_id   UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  old_content  TEXT,
  old_metadata JSONB,
  edited_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────
-- OTRAS TABLAS SOPORTE (Reacciones, Archivos)
-- ─────────────────────────────────────────
CREATE TABLE message_attachments (
  id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  message_id   UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  file_url     TEXT NOT NULL,
  file_name    TEXT NOT NULL,
  file_type    TEXT NOT NULL,
  file_size    BIGINT NOT NULL,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE message_reactions (
  id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  message_id   UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  profile_id   UUID NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  emoji        TEXT NOT NULL,
  UNIQUE (message_id, profile_id, emoji)
);

-- ─────────────────────────────────────────
-- ÍNDICES OPTIMIZADOS
-- ─────────────────────────────────────────
CREATE INDEX idx_msg_channel_time ON messages(channel_id, created_at DESC) WHERE channel_id IS NOT NULL;
CREATE INDEX idx_msg_conv_time    ON messages(conversation_id, created_at DESC) WHERE conversation_id IS NOT NULL;
CREATE INDEX idx_msg_fts_idx      ON messages USING GIN (fts_weighted);
CREATE INDEX idx_profiles_trgm    ON profiles USING GIN (display_name gin_trgm_ops);

-- ─────────────────────────────────────────
-- TRIGGERS Y FUNCIONES
-- ─────────────────────────────────────────

-- 1. Control de versiones y soft-delete
CREATE OR REPLACE FUNCTION fn_handle_message_update()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
  -- Si es un borrado lógico
  IF NEW.is_deleted = true AND OLD.is_deleted = false THEN
    NEW.content = NULL;
    NEW.deleted_at = NOW();
    RETURN NEW;
  END IF;

  -- Si es una edición de contenido
  IF (OLD.content IS DISTINCT FROM NEW.content) THEN
    INSERT INTO message_versions (message_id, old_content, old_metadata)
    VALUES (OLD.id, OLD.content, OLD.metadata);
    NEW.is_edited = true;
    NEW.edited_at = NOW();
  END IF;
  
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$;

CREATE TRIGGER trg_message_update
  BEFORE UPDATE ON messages
  FOR EACH ROW EXECUTE FUNCTION fn_handle_message_update();

-- 2. Actualizar contadores y timestamps de actividad
CREATE OR REPLACE FUNCTION fn_update_activity_stats()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
  IF NEW.channel_id IS NOT NULL THEN
    UPDATE channels SET last_message_at = NEW.created_at, message_count = message_count + 1 WHERE id = NEW.channel_id;
  ELSIF NEW.conversation_id IS NOT NULL THEN
    UPDATE conversations SET last_message_at = NEW.created_at WHERE id = NEW.conversation_id;
  END IF;
  
  -- Incrementar contador de respuestas si es un hilo
  IF NEW.parent_id IS NOT NULL THEN
    UPDATE messages SET reply_count = reply_count + 1 WHERE id = NEW.parent_id;
  END IF;
  
  RETURN NEW;
END;
$$;

CREATE TRIGGER trg_after_message_insert
  AFTER INSERT ON messages
  FOR EACH ROW EXECUTE FUNCTION fn_update_activity_stats();

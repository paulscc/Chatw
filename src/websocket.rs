use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::db::Database;
use crate::models::PresenceStatus;

// ============================================================================
// WEBSOCKET MESSAGE TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "subscribe")]
    Subscribe { target: SubscribeTarget },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { target: SubscribeTarget },
    #[serde(rename = "message")]
    ChatMessage { data: MessageData },
    #[serde(rename = "presence")]
    Presence { status: PresenceStatus },
    #[serde(rename = "typing")]
    Typing { is_typing: bool },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "target_type")]
pub enum SubscribeTarget {
    #[serde(rename = "channel")]
    Channel { id: Uuid },
    #[serde(rename = "conversation")]
    Conversation { id: Uuid },
    #[serde(rename = "workspace")]
    Workspace { id: Uuid },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub channel_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub content: String,
    pub message_type: String,
    pub parent_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub r#type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// WEBSOCKET SESSION ACTOR
// ============================================================================

pub struct WsSession {
    pub addr: Addr<ChatServer>,
    pub profile_id: Uuid,
    pub subscriptions: HashSet<SubscribeTarget>,
}

impl WsSession {
    pub fn new(addr: Addr<ChatServer>, profile_id: Uuid) -> Self {
        WsSession {
            addr,
            profile_id,
            subscriptions: HashSet::new(),
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr.send(Connect {
            addr: ctx.address(),
            profile_id: self.profile_id,
        }).into_actor(self)
            .map(|res, act, ctx| {
                if let Err(_) = res {
                    ctx.stop();
                }
            })
            .spawn(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        for target in &self.subscriptions {
            self.addr.send(Unsubscribe {
                addr: ctx.address(),
                profile_id: self.profile_id,
                target: target.clone(),
            }).into_actor(self)
                .map(|res, _, ctx| {
                    if let Err(_) = res {
                        tracing::error!("Failed to unsubscribe");
                    }
                })
                .spawn(ctx);
        }

        self.addr.send(Disconnect {
            addr: ctx.address(),
            profile_id: self.profile_id,
        }).into_actor(self)
            .map(|res, _, ctx| {
                if let Err(_) = res {
                    tracing::error!("Failed to disconnect");
                }
            })
            .spawn(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    self.handle_ws_message(ws_msg, ctx);
                } else {
                    tracing::error!("Invalid WebSocket message format");
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WsSession {
    fn handle_ws_message(&mut self, msg: WsMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            WsMessage::Ping => {
                ctx.text(serde_json::to_string(&WsMessage::Pong).unwrap());
            }
            WsMessage::Subscribe { target } => {
                self.subscriptions.insert(target.clone());
                self.addr.send(Subscribe {
                    addr: ctx.address(),
                    profile_id: self.profile_id,
                    target,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to subscribe");
                        }
                    })
                    .spawn(ctx);
            }
            WsMessage::Unsubscribe { target } => {
                self.subscriptions.remove(&target);
                self.addr.send(Unsubscribe {
                    addr: ctx.address(),
                    profile_id: self.profile_id,
                    target,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to unsubscribe");
                        }
                    })
                    .spawn(ctx);
            }
            WsMessage::ChatMessage { data } => {
                self.addr.send(NewMessage {
                    profile_id: self.profile_id,
                    data,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to send message");
                        }
                    })
                    .spawn(ctx);
            }
            WsMessage::Presence { status } => {
                self.addr.send(UpdatePresence {
                    profile_id: self.profile_id,
                    status,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to update presence");
                        }
                    })
                    .spawn(ctx);
            }
            WsMessage::Typing { is_typing } => {
                self.addr.send(Typing {
                    profile_id: self.profile_id,
                    is_typing,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to send typing status");
                        }
                    })
                    .spawn(ctx);
            }
            _ => {}
        }
    }
}

impl Handler<BroadcastMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

// ============================================================================
// CHAT SERVER ACTOR
// ============================================================================

pub struct ChatServer {
    sessions: HashMap<Uuid, HashSet<Addr<WsSession>>>,
    channel_subscribers: HashMap<Uuid, HashSet<Addr<WsSession>>>,
    conversation_subscribers: HashMap<Uuid, HashSet<Addr<WsSession>>>,
    workspace_subscribers: HashMap<Uuid, HashSet<Addr<WsSession>>>,
    db: Database,
}

impl ChatServer {
    pub fn new(db: Database) -> Self {
        ChatServer {
            sessions: HashMap::new(),
            channel_subscribers: HashMap::new(),
            conversation_subscribers: HashMap::new(),
            workspace_subscribers: HashMap::new(),
            db,
        }
    }

    fn broadcast_to_channel(&self, channel_id: Uuid, msg: BroadcastMessage) {
        if let Some(subscribers) = self.channel_subscribers.get(&channel_id) {
            for addr in subscribers {
                addr.do_send(msg.clone());
            }
        }
    }

    fn broadcast_to_conversation(&self, conversation_id: Uuid, msg: BroadcastMessage) {
        if let Some(subscribers) = self.conversation_subscribers.get(&conversation_id) {
            for addr in subscribers {
                addr.do_send(msg.clone());
            }
        }
    }

    fn broadcast_to_workspace(&self, workspace_id: Uuid, msg: BroadcastMessage) {
        if let Some(subscribers) = self.workspace_subscribers.get(&workspace_id) {
            for addr in subscribers {
                addr.do_send(msg.clone());
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

// ============================================================================
// CHAT SERVER MESSAGES
// ============================================================================

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Addr<WsSession>,
    pub profile_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Addr<WsSession>,
    pub profile_id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub addr: Addr<WsSession>,
    pub profile_id: Uuid,
    pub target: SubscribeTarget,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Unsubscribe {
    pub addr: Addr<WsSession>,
    pub profile_id: Uuid,
    pub target: SubscribeTarget,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewMessage {
    pub profile_id: Uuid,
    pub data: MessageData,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdatePresence {
    pub profile_id: Uuid,
    pub status: PresenceStatus,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Typing {
    pub profile_id: Uuid,
    pub is_typing: bool,
}

// ============================================================================
// CHAT SERVER HANDLERS
// ============================================================================

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) {
        self.sessions.entry(msg.profile_id)
            .or_insert_with(HashSet::new)
            .insert(msg.addr.clone());

        tracing::info!("Profile {} connected", msg.profile_id);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) {
        if let Some(sessions) = self.sessions.get_mut(&msg.profile_id) {
            sessions.remove(&msg.addr);
            if sessions.is_empty() {
                self.sessions.remove(&msg.profile_id);
            }
        }

        for subscribers in self.channel_subscribers.values_mut() {
            subscribers.remove(&msg.addr);
        }
        for subscribers in self.conversation_subscribers.values_mut() {
            subscribers.remove(&msg.addr);
        }
        for subscribers in self.workspace_subscribers.values_mut() {
            subscribers.remove(&msg.addr);
        }

        tracing::info!("Profile {} disconnected", msg.profile_id);
    }
}

impl Handler<Subscribe> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) {
        match msg.target {
            SubscribeTarget::Channel { id } => {
                self.channel_subscribers.entry(id)
                    .or_insert_with(HashSet::new)
                    .insert(msg.addr);
            }
            SubscribeTarget::Conversation { id } => {
                self.conversation_subscribers.entry(id)
                    .or_insert_with(HashSet::new)
                    .insert(msg.addr);
            }
            SubscribeTarget::Workspace { id } => {
                self.workspace_subscribers.entry(id)
                    .or_insert_with(HashSet::new)
                    .insert(msg.addr);
            }
        }
    }
}

impl Handler<Unsubscribe> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _ctx: &mut Self::Context) {
        match msg.target {
            SubscribeTarget::Channel { id } => {
                if let Some(subscribers) = self.channel_subscribers.get_mut(&id) {
                    subscribers.remove(&msg.addr);
                    if subscribers.is_empty() {
                        self.channel_subscribers.remove(&id);
                    }
                }
            }
            SubscribeTarget::Conversation { id } => {
                if let Some(subscribers) = self.conversation_subscribers.get_mut(&id) {
                    subscribers.remove(&msg.addr);
                    if subscribers.is_empty() {
                        self.conversation_subscribers.remove(&id);
                    }
                }
            }
            SubscribeTarget::Workspace { id } => {
                if let Some(subscribers) = self.workspace_subscribers.get_mut(&id) {
                    subscribers.remove(&msg.addr);
                    if subscribers.is_empty() {
                        self.workspace_subscribers.remove(&id);
                    }
                }
            }
        }
    }
}

impl Handler<NewMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: NewMessage, ctx: &mut Self::Context) {
        let db = self.db.clone();
        let profile_id = msg.profile_id;
        let data = msg.data.clone();

        async move {
            use crate::messages::MessageService;
            use crate::models::MessageType;

            let service = MessageService::new(db);
            
            let message_type = match data.message_type.as_str() {
                "text" => MessageType::Text,
                "image" => MessageType::Image,
                "file" => MessageType::File,
                "code" => MessageType::Code,
                _ => MessageType::Text,
            };

            let message = service.create_message(crate::models::CreateMessage {
                channel_id: data.channel_id,
                conversation_id: data.conversation_id,
                sender_id: profile_id,
                parent_id: data.parent_id,
                content: Some(data.content),
                r#type: message_type,
                metadata: data.metadata,
            }).await;

            message
        }
        .into_actor(self)
        .map(move |res, act, _ctx| {
            if let Ok(message) = res {
                let broadcast = BroadcastMessage {
                    r#type: "message".to_string(),
                    data: serde_json::to_value(&message).unwrap(),
                    timestamp: chrono::Utc::now(),
                };

                if let Some(channel_id) = message.channel_id {
                    act.broadcast_to_channel(channel_id, broadcast);
                } else if let Some(conversation_id) = message.conversation_id {
                    act.broadcast_to_conversation(conversation_id, broadcast);
                }
            }
        })
        .spawn(ctx);
    }
}

impl Handler<UpdatePresence> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: UpdatePresence, ctx: &mut Self::Context) {
        let db = self.db.clone();
        let profile_id = msg.profile_id;
        let status = msg.status;

        async move {
            use crate::auth::AuthService;
            let service = AuthService::new(db);
            service.update_presence(profile_id, status).await
        }
        .into_actor(self)
        .map(|res, act, _ctx| {
            if let Ok(profile) = res {
                let broadcast = BroadcastMessage {
                    r#type: "presence".to_string(),
                    data: serde_json::to_value(&profile).unwrap(),
                    timestamp: chrono::Utc::now(),
                };

                for subscribers in act.workspace_subscribers.values() {
                    for addr in subscribers {
                        addr.do_send(broadcast.clone());
                    }
                }
            }
        })
        .spawn(ctx);
    }
}

impl Handler<Typing> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Typing, _ctx: &mut Self::Context) {
        let broadcast = BroadcastMessage {
            r#type: "typing".to_string(),
            data: serde_json::json!({
                "profile_id": msg.profile_id,
                "is_typing": msg.is_typing,
            }),
            timestamp: chrono::Utc::now(),
        };

        for subscribers in self.channel_subscribers.values() {
            for addr in subscribers {
                addr.do_send(broadcast.clone());
            }
        }
        for subscribers in self.conversation_subscribers.values() {
            for addr in subscribers {
                addr.do_send(broadcast.clone());
            }
        }
    }
}

// ============================================================================
// WEBSOCKET ROUTE
// ============================================================================

pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    db: web::Data<Database>,
    chat_server: web::Data<Addr<ChatServer>>,
    profile_id: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let profile_id = profile_id.into_inner();
    let ws_session = WsSession::new(chat_server.get_ref().clone(), profile_id);
    
    ws::start(ws_session, &req, stream)
}

pub async fn start_chat_server(db: Database) -> Addr<ChatServer> {
    ChatServer::new(db).start()
}

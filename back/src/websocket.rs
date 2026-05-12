use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::db::Database;
use crate::models::PresenceStatus;
use crate::redis_client::RedisClient;
use crate::cache_service::CacheService;

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

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
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
    pub room_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub message_type: String,
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
                tracing::info!("Mensaje recibido: {}", text);
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    tracing::info!("Mensaje parseado correctamente: {:?}", ws_msg);
                    self.handle_ws_message(ws_msg, ctx);
                } else {
                    tracing::error!("Invalid WebSocket message format. Mensaje: {}", text);
                    // Intentar parsear como JSON genérico para ver la estructura
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
                        tracing::error!("Estructura JSON: {}", serde_json::to_string_pretty(&json_value).unwrap_or_default());
                    }
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
                
                // Enviar suscripción al ChatServer para que gestione los suscriptores
                self.addr.send(Subscribe {
                    addr: ctx.address(),
                    profile_id: self.profile_id,
                    target,
                }).into_actor(self)
                    .map(|res, _, _ctx| {
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
    db: Option<Database>,
    redis_client: RedisClient,
    cache_service: CacheService,
    addr: Addr<ChatServer>,
    redis_subscriptions: HashMap<String, Addr<ChatServer>>, // room_code -> ChatServer addr
}

impl ChatServer {
    pub fn new(db: Database, redis_client: RedisClient, cache_service: CacheService, addr: Addr<ChatServer>) -> Self {
        ChatServer {
            sessions: HashMap::new(),
            channel_subscribers: HashMap::new(),
            conversation_subscribers: HashMap::new(),
            workspace_subscribers: HashMap::new(),
            db: Some(db),
            redis_client,
            cache_service,
            addr,
            redis_subscriptions: HashMap::new(),
        }
    }
    
    pub fn new_no_db(redis_client: RedisClient, cache_service: CacheService, addr: Addr<ChatServer>) -> Self {
        ChatServer {
            sessions: HashMap::new(),
            channel_subscribers: HashMap::new(),
            conversation_subscribers: HashMap::new(),
            workspace_subscribers: HashMap::new(),
            db: None,
            redis_client,
            cache_service,
            addr,
            redis_subscriptions: HashMap::new(),
        }
    }

    fn start_redis_subscription(&self, room_code: &str, ctx: &mut Context<Self>) {
        let redis_client = self.redis_client.clone();
        let addr = ctx.address();
        let room_code = room_code.to_string();
        
        actix::spawn(async move {
            // Suscribirse directamente al canal con el nombre de la sala
            let redis_channel = room_code.clone(); // Usar directamente el nombre de sala
            match redis_client.subscribe(&redis_channel).await {
                Ok(mut pubsub) => {
                    tracing::info!("Suscrito a Redis pub/sub channel: {}", redis_channel);
                    
                    // Escuchar mensajes de Redis
                    while let Some(msg) = pubsub.next().await {
                        match msg {
                            Ok(redis_msg) => {
                                tracing::info!("Mensaje recibido de Redis para sala {}: {}", room_code, redis_msg);
                                
                                // Parsear y reenviar a clientes WebSocket
                                if let Ok(parsed_msg) = serde_json::from_str::<serde_json::Value>(&redis_msg) {
                                    // Enviar mensaje al ChatServer para que lo distribuya
                                    addr.do_send(RedisMessage { data: parsed_msg });
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error recibiendo mensaje de Redis para sala {}: {}", room_code, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Error suscribiéndose a Redis pub/sub para sala {}: {}", room_code, e);
                }
            }
        });
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

#[derive(Message)]
#[rtype(result = "()")]
pub struct RedisMessage {
    pub data: serde_json::Value,
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

        // Remove from Redis cache when user disconnects
        let profile_id_str = msg.profile_id.to_string();
        let cache_service = self.cache_service.clone();
        actix::spawn(async move {
            if let Err(e) = cache_service.invalidate_profile_cache(&profile_id_str).await {
                tracing::error!("Failed to invalidate profile cache: {}", e);
            }
        });

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
                    .insert(msg.addr.clone());
                
                tracing::info!("Usuario {} suscrito al canal {}", msg.profile_id, id);
            }
            SubscribeTarget::Conversation { id } => {
                self.conversation_subscribers.entry(id)
                    .or_insert_with(HashSet::new)
                    .insert(msg.addr.clone());
            }
            SubscribeTarget::Workspace { id } => {
                self.workspace_subscribers.entry(id)
                    .or_insert_with(HashSet::new)
                    .insert(msg.addr.clone());
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

    fn handle(&mut self, msg: NewMessage, _ctx: &mut Self::Context) {
        // Sistema Redis-based de mensajería por sala
        let profile_id = msg.profile_id;
        let data = msg.data.clone();

        tracing::info!("Procesando mensaje de usuario {}: {}", profile_id, data.content);

        // Determinar el canal Redis usando directamente el nombre de sala
        let room_code = data.room_code.as_ref().unwrap_or(&"global".to_string()).clone();
        let redis_channel = room_code.clone(); // Usar directamente el nombre de sala como canal

        // Crear mensaje para Redis pub/sub
        let redis_message = serde_json::json!({
            "type": "message",
            "data": {
                "id": Uuid::new_v4(),
                "content": data.content,
                "sender_id": profile_id,
                "timestamp": chrono::Utc::now(),
                "room_code": room_code
            }
        });

        tracing::info!("Publicando mensaje en canal Redis: {}", redis_channel);

        // Publicar en Redis pub/sub específico de la sala
        let redis_client = self.redis_client.clone();
        actix::spawn(async move {
            // Publicar en el canal específico de la sala
            if let Err(e) = redis_client.publish(&redis_channel, &redis_message.to_string()).await {
                tracing::error!("Error publicando mensaje en Redis canal {}: {}", redis_channel, e);
            } else {
                tracing::info!("Mensaje publicado exitosamente en canal: {}", redis_channel);
            }
        });
    }
}

impl Handler<UpdatePresence> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: UpdatePresence, _ctx: &mut Self::Context) {
        // Versión simplificada: Solo broadcast de presencia
        let profile_id = msg.profile_id;
        let status = msg.status;

        let broadcast = BroadcastMessage {
            message_type: "presence".to_string(),
            data: serde_json::json!({
                "profile_id": profile_id,
                "status": status,
                "timestamp": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
        };

        // Broadcast a todas las sesiones conectadas
        for session in self.sessions.values() {
            for addr in session {
                addr.do_send(broadcast.clone());
            }
        }
    }
}

impl Handler<Typing> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Typing, _ctx: &mut Self::Context) {
        let broadcast = BroadcastMessage {
            message_type: "typing".to_string(),
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

impl Handler<RedisMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RedisMessage, _ctx: &mut Self::Context) {
        tracing::info!("Recibido mensaje de Redis, distribuyendo a {} sesiones", self.sessions.len());
        
        // Crear mensaje para broadcast
        let broadcast = BroadcastMessage {
            message_type: "message".to_string(),
            data: msg.data,
            timestamp: chrono::Utc::now(),
        };

        // Enviar a todas las sesiones conectadas
        for session in self.sessions.values() {
            for addr in session {
                addr.do_send(broadcast.clone());
            }
        }

        tracing::info!("Mensaje de Redis distribuido a todas las sesiones");
    }
}

// ============================================================================
// WEBSOCKET ROUTE
// ============================================================================

pub async fn index(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let room_code = path.into_inner();
    
    tracing::info!("Iniciando conexión WebSocket para sala: {}", room_code);
    
    // Versión simplificada: Solo WebSocket básico sin dependencias complejas
    let redis_client = match RedisClient::new("redis://localhost:6379").await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Error conectando a Redis: {}", e);
            return Err(actix_web::error::ErrorInternalServerError("Error Redis"));
        }
    };
    
    let cache_service = CacheService::new(redis_client.clone());
    
    // Crear ChatServer sin Redis suscripción por ahora (versión simplificada)
    let chat_server = ChatServer::new_no_db(redis_client, cache_service, Default::default());
    let chat_server_addr = chat_server.start();
    
    // Iniciar suscripción al canal específico de la sala
    // Nota: Esto debería hacerse en el started() del actor, pero por ahora lo dejamos así
    tracing::info!("Debería suscribirse al canal Redis: {}", room_code);
    
    let ws_session = WsSession::new(chat_server_addr, Uuid::new_v4());
    
    ws::start(ws_session, &req, stream)
}

pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    _db: web::Data<Database>,
    chat_server: web::Data<Addr<ChatServer>>,
    profile_id: web::Path<Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let profile_id = profile_id.into_inner();
    let ws_session = WsSession::new(chat_server.get_ref().clone(), profile_id);
    
    ws::start(ws_session, &req, stream)
}

pub async fn start_chat_server(db: Database, redis_client: RedisClient, cache_service: CacheService) -> Addr<ChatServer> {
    ChatServer::new(db, redis_client, cache_service).start()
}

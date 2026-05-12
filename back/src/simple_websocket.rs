use actix::prelude::*;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

// ============================================================================
// SIMPLE WEBSOCKET FOR ROOM-BASED CHAT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SimpleWsMessage {
    #[serde(rename = "join")]
    JoinRoom { data: RoomJoinData },
    #[serde(rename = "message")]
    ChatMessage { data: MessageData },
    #[serde(rename = "leave")]
    LeaveRoom,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomJoinData {
    pub room_code: String,
    pub user: RoomUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomUser {
    pub id: String,
    pub display_name: String,
    pub avatar: String,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageData {
    pub content: String,
    pub room_code: String,
    pub sender_id: String,
    pub sender_name: String,
    pub sender_avatar: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub r#type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// SIMPLE CHAT SESSION
// ============================================================================

pub struct SimpleWsSession {
    pub addr: Addr<SimpleChatServer>,
    pub session_id: String,
    pub room_code: Option<String>,
    pub user: Option<RoomUser>,
}

impl SimpleWsSession {
    pub fn new(addr: Addr<SimpleChatServer>, session_id: String) -> Self {
        SimpleWsSession {
            addr,
            session_id,
            room_code: None,
            user: None,
        }
    }
}

impl Actor for SimpleWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Session started
        tracing::info!("WebSocket session started: {}", self.session_id);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Leave room if connected
        if let Some(room_code) = &self.room_code {
            self.addr.send(LeaveRoom {
                session_id: self.session_id.clone(),
                room_code: room_code.clone(),
            }).into_actor(self)
                .map(|res, _, ctx| {
                    if let Err(_) = res {
                        tracing::error!("Failed to leave room on disconnect");
                    }
                })
                .spawn(ctx);
        }

        tracing::info!("WebSocket session ended: {}", self.session_id);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for SimpleWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // Keep alive
            }
            Ok(ws::Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<SimpleWsMessage>(&text) {
                    self.handle_ws_message(ws_msg, ctx);
                } else {
                    tracing::error!("Invalid WebSocket message format: {}", text);
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

impl SimpleWsSession {
    fn handle_ws_message(&mut self, msg: SimpleWsMessage, ctx: &mut ws::WebsocketContext<Self>) {
        match msg {
            SimpleWsMessage::JoinRoom { data } => {
                self.room_code = Some(data.room_code.clone());
                self.user = Some(data.user.clone());
                
                self.addr.send(JoinRoom {
                    session_id: self.session_id.clone(),
                    room_code: data.room_code,
                    user: data.user,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to join room");
                        }
                    })
                    .spawn(ctx);
            }
            SimpleWsMessage::ChatMessage { data } => {
                self.addr.send(RoomMessage {
                    session_id: self.session_id.clone(),
                    room_code: data.room_code.clone(),
                    data,
                }).into_actor(self)
                    .map(|res, _, ctx| {
                        if let Err(_) = res {
                            tracing::error!("Failed to send message");
                        }
                    })
                    .spawn(ctx);
            }
            SimpleWsMessage::LeaveRoom => {
                if let Some(room_code) = &self.room_code {
                    self.addr.send(LeaveRoom {
                        session_id: self.session_id.clone(),
                        room_code: room_code.clone(),
                    }).into_actor(self)
                        .map(|res, _, ctx| {
                            if let Err(_) = res {
                                tracing::error!("Failed to leave room");
                            }
                        })
                        .spawn(ctx);
                }
            }
            SimpleWsMessage::Ping => {
                ctx.text(serde_json::to_string(&SimpleWsMessage::Pong).unwrap());
            }
            _ => {}
        }
    }
}

impl Handler<BroadcastMessage> for SimpleWsSession {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

// ============================================================================
// SIMPLE CHAT SERVER
// ============================================================================

pub struct SimpleChatServer {
    sessions: HashMap<String, Addr<SimpleWsSession>>, // session_id -> session
    room_users: HashMap<String, HashMap<String, RoomUser>>, // room_code -> users
}

impl SimpleChatServer {
    pub fn new() -> Self {
        SimpleChatServer {
            sessions: HashMap::new(),
            room_users: HashMap::new(),
        }
    }

    fn broadcast_to_room(&self, room_code: &str, msg: BroadcastMessage) {
        if let Some(users) = self.room_users.get(room_code) {
            for (user_id, _) in users {
                if let Some(session) = self.sessions.get(user_id) {
                    session.do_send(msg.clone());
                }
            }
        }
    }

    fn add_user_to_room(&mut self, room_code: String, user: RoomUser, session_id: String) -> Vec<RoomUser> {
        let users = self.room_users.entry(room_code.clone()).or_insert_with(HashMap::new);
        users.insert(user.id.clone(), user.clone());
        
        // Get all users in room
        let all_users: Vec<RoomUser> = users.values().cloned().collect();
        
        // Broadcast user joined
        let join_msg = BroadcastMessage {
            r#type: "user_joined".to_string(),
            data: serde_json::json!({
                "displayName": user.display_name,
                "users": all_users
            }),
            timestamp: chrono::Utc::now(),
        };
        self.broadcast_to_room(&room_code, join_msg);
        
        all_users
    }

    fn remove_user_from_room(&mut self, room_code: &str, session_id: &str) -> Option<Vec<RoomUser>> {
        if let Some(users) = self.room_users.get_mut(room_code) {
            // Find and remove user
            let user_to_remove = users.iter()
                .find(|(id, _)| *id == session_id)
                .map(|(_, user)| user.clone());
            
            if let Some(user) = user_to_remove {
                users.remove(session_id);
                
                // Get remaining users
                let all_users: Vec<RoomUser> = users.values().cloned().collect();
                
                // Broadcast user left
                let leave_msg = BroadcastMessage {
                    r#type: "user_left".to_string(),
                    data: serde_json::json!({
                        "displayName": user.display_name,
                        "users": all_users
                    }),
                    timestamp: chrono::Utc::now(),
                };
                self.broadcast_to_room(room_code, leave_msg);
                
                // Clean up empty rooms
                if users.is_empty() {
                    self.room_users.remove(room_code);
                }
                
                return Some(all_users);
            }
        }
        None
    }
}

impl Actor for SimpleChatServer {
    type Context = Context<Self>;
}

// ============================================================================
// SIMPLE CHAT SERVER MESSAGES
// ============================================================================

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Addr<SimpleWsSession>,
    pub session_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub addr: Addr<SimpleWsSession>,
    pub session_id: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoom {
    pub session_id: String,
    pub room_code: String,
    pub user: RoomUser,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoom {
    pub session_id: String,
    pub room_code: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage {
    pub session_id: String,
    pub room_code: String,
    pub data: MessageData,
}

// ============================================================================
// SIMPLE CHAT SERVER HANDLERS
// ============================================================================

impl Handler<Connect> for SimpleChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _ctx: &mut Self::Context) {
        self.sessions.insert(msg.session_id, msg.addr);
        tracing::info!("Session connected: {}", msg.session_id);
    }
}

impl Handler<Disconnect> for SimpleChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Self::Context) {
        self.sessions.remove(&msg.session_id);
        
        // Remove from all rooms
        let rooms_to_check: Vec<String> = self.room_users.keys().cloned().collect();
        for room_code in rooms_to_check {
            self.remove_user_from_room(&room_code, &msg.session_id);
        }
        
        tracing::info!("Session disconnected: {}", msg.session_id);
    }
}

impl Handler<JoinRoom> for SimpleChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) {
        let users = self.add_user_to_room(msg.room_code.clone(), msg.user, msg.session_id.clone());
        
        // Send user list to the joining user
        if let Some(session) = self.sessions.get(&msg.session_id) {
            let user_list_msg = BroadcastMessage {
                r#type: "user_list".to_string(),
                data: serde_json::json!(users),
                timestamp: chrono::Utc::now(),
            };
            session.do_send(user_list_msg);
        }
        
        tracing::info!("User {} joined room {}", msg.session_id, msg.room_code);
    }
}

impl Handler<LeaveRoom> for SimpleChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
        self.remove_user_from_room(&msg.room_code, &msg.session_id);
        tracing::info!("User {} left room {}", msg.session_id, msg.room_code);
    }
}

impl Handler<RoomMessage> for SimpleChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, _ctx: &mut Self::Context) {
        let broadcast = BroadcastMessage {
            r#type: "message".to_string(),
            data: serde_json::json!({
                "content": msg.data.content,
                "senderId": msg.data.sender_id,
                "senderName": msg.data.sender_name,
                "senderAvatar": msg.data.sender_avatar,
                "timestamp": msg.data.timestamp,
                "roomCode": msg.data.room_code
            }),
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_to_room(&msg.room_code, broadcast);
        tracing::info!("Message sent to room {}: {}", msg.room_code, msg.data.content);
    }
}

// ============================================================================
// SIMPLE WEBSOCKET ROUTE
// ============================================================================

pub async fn simple_websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<Addr<SimpleChatServer>>,
    room_code: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let session_id = format!("{}_{}", room_code, uuid::Uuid::new_v4());
    let ws_session = SimpleWsSession::new(chat_server.get_ref().clone(), session_id.clone());
    
    // Connect to server
    chat_server.send(Connect {
        addr: ws_session.addr.clone(),
        session_id,
    }).await.unwrap();
    
    ws::start(ws_session, &req, stream)
}

pub async fn start_simple_chat_server() -> Addr<SimpleChatServer> {
    SimpleChatServer::new().start()
}

use actix::{Actor, StreamHandler, Addr, Message};
use actix_web::{web, App, HttpServer, HttpRequest, Responder};
use actix_web_actors::ws;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use actix::ActorContext;
use actix::AsyncContext;
use gamesync_server::server::GameServer;
use gamesync_server::server_params::ServerParams;


// ChatRoom actor that manages connected sessions and broadcasts messages
struct ChatRoom {
    sessions: Arc<Mutex<HashSet<Addr<WsSession>>>>,
}

impl ChatRoom {
    fn new() -> Self {
        ChatRoom {
            sessions: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

impl Actor for ChatRoom {
    type Context = actix::Context<Self>;
}

// Message that will be sent between clients
#[derive(Message, Serialize, Deserialize, Clone)]
#[rtype(result = "()")]  // This ensures that ChatMessage has a result type ()
struct ChatMessage(String);

// WebSocket session that handles incoming/outgoing messages
struct WsSession {
    room: Addr<ChatRoom>, // This is an address to the ChatRoom Actor
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        // Register session in the room
        self.room.do_send(JoinRoom { addr: addr.clone() });
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> actix::Running {
        // Remove session from the room when stopping
        let addr = ctx.address();
        self.room.do_send(LeaveRoom { addr });
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Send a ChatMessage to the room for broadcasting
                self.room.do_send(ChatMessage(text.to_string()));
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl actix::Handler<ChatMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
        // Send the message to this client
        ctx.text(msg.0);
    }
}

// Implement Handler for ChatMessage on ChatRoom
impl actix::Handler<ChatMessage> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, _: &mut Self::Context) {
        // Broadcast the message to all connected sessions
        let sessions = self.sessions.lock().unwrap();
        for session in sessions.iter() {
            session.do_send(msg.clone()); // Send the message to each session
        }
    }
}

// Messages for joining and leaving a room
#[derive(Message)]
#[rtype(result = "()")]
struct JoinRoom {
    addr: Addr<WsSession>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct LeaveRoom {
    addr: Addr<WsSession>,
}

// ChatRoom actor handling join and leave messages
impl actix::Handler<JoinRoom> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Self::Context) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(msg.addr);
    }
}

impl actix::Handler<LeaveRoom> for ChatRoom {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _: &mut Self::Context) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(&msg.addr);
    }
}

// WebSocket handler for incoming connections
async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    room: web::Data<Addr<ChatRoom>>,
) -> impl Responder {
    let session = WsSession {
        room: room.get_ref().clone(),
    };
    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let chat_room = ChatRoom::new().start();
    let mut server = GameServer::new("8080", ServerParams { player_count: 5 }).unwrap();
    server.process_messages();
    Ok(())

    // HttpServer::new(move || {
    //     App::new()
    //         .app_data(web::Data::new(chat_room.clone()))
    //         // .route("/ws/", web::get().to(ws_handler))
    // })
    // .bind("127.0.0.1:8080")?
    // .run()
    // .await
}

// use actix::{Actor, StreamHandler, Addr, Message};
// use actix_web::{web, App, HttpServer, HttpRequest, Responder};
// use actix_web_actors::ws;
// use std::collections::HashSet;
// use std::sync::{Arc, Mutex};
// use serde::{Deserialize, Serialize};
// use actix::ActorContext;
// use actix::AsyncContext;
use gamesync_server::server::GameServer;
use gamesync_server::server_params::ServerParams;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let chat_room = ChatRoom::new().start();
    let mut server = GameServer::new("8080", ServerParams { player_count: 2 }).unwrap();
    server.process_messages();
    Ok(())
}

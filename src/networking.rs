use crate::error::GameSyncError::ParseError;
use crate::error::{print_error, GameSyncError};
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeEvent, NodeHandler, NodeTask};
use serde::{Deserialize, Serialize};

// Sync these with server-side enum
#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    Connected(), // existing event, not needed on server-side
    UserMessage(String),
    SelfPlayer(String),
    NewPlayer(String),
}

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    Broadcast(String),
    SendTo(String, String),
    // more game events to send
}

pub struct Websocket {
    handler: NodeHandler<ServerEvent>,
    server: Endpoint,
    node_task: Option<NodeTask>,
}

impl Websocket {
    pub fn new(url: &str, send_event: impl Fn(ServerEvent) + Send + 'static) -> Result<Self, GameSyncError> {
        let (handler, listener) = node::split();
        let (server, _) = handler.network().connect(Transport::Ws, url)?;

        let mut websocket = Self { handler: handler.clone(), server, node_task: None };
        let mut connection = ServerConnection::new(handler);

        let node_task = listener.for_each_async(move |event| {
            connection.handle_messages(event, |event| send_event(event));
        });

        websocket.node_task = Some(node_task);
        Ok(websocket)
    }

    pub fn send_event(&mut self, event: ClientEvent) -> Result<SendStatus, GameSyncError> {
        let payload = serde_json::to_string(&event)?;
        Ok(self.handler.network().send(self.server, payload.as_ref()))
    }
}

pub struct ServerConnection {
    handler: NodeHandler<ServerEvent>,
}

impl ServerConnection {
    pub fn new(handler: NodeHandler<ServerEvent>) -> Self {
        Self {
            handler
        }
    }

    pub fn handle_messages(&mut self, event: NodeEvent<ServerEvent>, send_event: impl Fn(ServerEvent)) {
        match event {
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Connected(_, _) | NetEvent::Accepted(_, _) => {
                    send_event(ServerEvent::Connected());
                }
                NetEvent::Message(_, message) => {
                    let payload = serde_json::from_slice(&message);

                    match payload {
                        Ok(payload) => { self.handler.signals().send(payload); }
                        Err(e) => { print_error(ParseError(e)) }
                    }
                }
                NetEvent::Disconnected(_) => {}
            },
            NodeEvent::Signal(msg) => {
                // TODO: implement callbacks here
                match msg {
                    ServerEvent::UserMessage(message) => {
                        send_event(ServerEvent::UserMessage(message));
                    }
                    ServerEvent::SelfPlayer(data) => {
                        println!("My id: {:?}", data);
                        send_event(ServerEvent::SelfPlayer(data));
                    }
                    ServerEvent::NewPlayer(data) => {
                        println!("New player {:?} joined!", data);
                        send_event(ServerEvent::NewPlayer(data));
                    }
                    _ => {}
                }
            }
        }
    }
}

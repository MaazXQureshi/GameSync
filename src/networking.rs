use crate::store::DataStore;
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{GameSyncError, print_error};

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    UserMessage(String),
    SelfPlayer(String),
    NewPlayer(String),
}

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    Broadcast(String),
    SendTo(String, String), // To, Msg
    // more game events to send b/w clients and server
}

pub struct Websocket {
    handler: NodeHandler<ClientEvent>,
    listener: Option<NodeListener<ClientEvent>>,
    data_store: DataStore,
}

impl Websocket {
    pub fn new(port: &str) -> Result<Self, GameSyncError> {
        let (handler, listener) = node::split();
        handler.network().listen(Transport::Ws, String::from("0.0.0.0") + ":" + port)?;
        let data_store = DataStore::new();
        println!("Server set up");

        Ok(Websocket { handler, listener: Some(listener), data_store })
    }
    pub fn process_messages(&mut self) {
        self.listener.take().unwrap().for_each(move |event|
            match event {
                NodeEvent::Network(net_event) => match net_event {
                    NetEvent::Accepted(endpoint, _) | NetEvent::Connected(endpoint, _) => {
                        match self.handle_new_connections(endpoint) {
                            Ok(_) => {},
                            Err(e) => print_error(e),
                        }
                    }
                    NetEvent::Message(_, message) => {
                        let payload = serde_json::from_slice(&message).unwrap();
                        self.handler.signals().send(payload);
                    }
                    NetEvent::Disconnected(endpoint) => {
                        println!("User {} disconnected", self.data_store.get_user(endpoint).unwrap());
                        self.data_store.remove_user_endpoint(endpoint);
                    }
                }
                NodeEvent::Signal(msg) => {
                    match self.handle_messages(msg) {
                        Ok(_) => {},
                        Err(e) => print_error(e),
                    }
                }
            }
        );
    }

    fn handle_messages(&mut self, event: ClientEvent) -> Result<(), GameSyncError> {
        match event {
            ClientEvent::Broadcast(message) => {
                println!("Broadcasting message: {}", message);
                let event = ServerEvent::UserMessage(message);
                self.send_to_all_clients(event)?;
            }
            ClientEvent::SendTo(uuid, message) => {
                println!("To: {} Message: {}", uuid, message);
                let event = ServerEvent::UserMessage(message);
                self.send_to_client(&uuid, event)?;
            }
        }

        Ok(())
    }

    pub fn handle_new_connections(&mut self, endpoint: Endpoint) -> Result<(), GameSyncError> {
        // send user info to new connected client
        let id = self.data_store.add_user_endpoint(endpoint).to_string();
        println!("Connection from {}", id);
        let event = ServerEvent::SelfPlayer(id.clone());
        self.send_to_client(&id, event)?;

        // send new player id to all existing clients
        let event = ServerEvent::NewPlayer(id.to_string());
        self.send_to_all_clients(event)?;

        Ok(())
    }

    pub fn send_to_client(&mut self, to: &String, event: ServerEvent) -> Result<(), GameSyncError> {
        let user_id = Uuid::parse_str(&to)?;
        let endpoint = self.data_store.get_user_endpoint(&user_id);
        let payload = serde_json::to_string(&event)?;

        match endpoint {
            Some(endpoint) => {
                let status = self.handler.network().send(endpoint, payload.as_ref());
                if status != SendStatus::Sent {
                    return Err(GameSyncError::SendError)
                }
            },
            None => return Err(GameSyncError::SendError)
        }

        Ok(())
    }

    pub fn send_to_all_clients(&mut self, event: ServerEvent) -> Result<SendStatus, GameSyncError> {
        let endpoints = self.data_store.get_all_user_endpoints();
        let payload = serde_json::to_string(&event)?;

        for endpoint in endpoints {
            let status = self.handler.network().send(endpoint, payload.as_ref());
            if status != SendStatus::Sent {
                return Err(GameSyncError::SendError)
            }
        }

        Ok(SendStatus::Sent)
    }
}

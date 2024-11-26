use crate::error::GameSyncError::ParseError;
use crate::error::{print_error, GameSyncError};
use crate::lobby::{LobbyParams, Player, Region};
use crate::server_events::ServerEvent;
use crate::store::{LobbyID, PlayerID};
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeEvent, NodeHandler, NodeTask};
use serde::{Deserialize, Serialize};
// Sync these with server-side enum

#[derive(Serialize, Deserialize)]
// make these methods async to wait for server event of client-initiated requests
pub enum ClientEvent {
    Broadcast(String),
    SendTo(String, String),
    CreateLobby(PlayerID, LobbyParams), // async wait for LobbyCreated
    JoinLobby(PlayerID, LobbyID), // event wait for LobbyJoined
    DeleteLobby(PlayerID, LobbyID), // event wait for LobbyDeleted
    LeaveLobby(PlayerID, LobbyID), // async wait for LobbyLeft
    InviteLobby(PlayerID, LobbyID, PlayerID),  // Sender ID, Lobby ID, Invitee ID
    GetPublicLobbies(PlayerID, Region),
    EditPlayer(Player),
    MessageLobby(PlayerID, LobbyID, String), // Sender ID, Lobby ID, Message
    QueueLobby(PlayerID, LobbyID),
    CheckMatch(PlayerID, LobbyID, Option<usize>), // Sender ID, Lobby ID, Threshold
    StopQueue(PlayerID, LobbyID),
    LeaveGameAsLobby(PlayerID, LobbyID),
    GetLobbyInfo(PlayerID, LobbyID),
}

#[derive(Clone)]
pub struct Websocket {
    handler: NodeHandler<ServerEvent>,
    server: Endpoint,
    // node_task: Option<NodeTask>,
}

impl Websocket {
    pub fn new(url: &str, send_event: impl Fn(ServerEvent) + Send + 'static) -> Result<(Self, NodeTask), GameSyncError> {
        let (handler, listener) = node::split();
        let (server, _) = handler.network().connect(Transport::Ws, url)?;

        let websocket = Self { handler: handler.clone(), server };
        let mut connection = ServerConnection::new(handler);

        let node_task = listener.for_each_async(move |event| {
            connection.handle_messages(event, |event| send_event(event));
        });

        // websocket.node_task = Some(node_task);
        Ok((websocket, node_task))
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
                    ServerEvent::UserMessage(from, message) => {
                        send_event(ServerEvent::UserMessage(from, message));
                    }
                    ServerEvent::SelfPlayer(data) => {
                        send_event(ServerEvent::SelfPlayer(data));
                    }
                    ServerEvent::NewPlayer(data) => {
                        send_event(ServerEvent::NewPlayer(data));
                    }
                    ServerEvent::LobbyCreated(id) => {
                        send_event(ServerEvent::LobbyCreated(id));
                    }
                    ServerEvent::LobbyJoined(id) => {
                        send_event(ServerEvent::LobbyJoined(id));
                    }
                    ServerEvent::LobbyDeleted(id) => {
                        send_event(ServerEvent::LobbyDeleted(id));
                    }
                    ServerEvent::LobbyLeft(player_id, lobby_id) => {
                        send_event(ServerEvent::LobbyLeft(player_id, lobby_id));
                    }
                    ServerEvent::LobbyInvited(id) => {
                        send_event(ServerEvent::LobbyInvited(id));
                    }
                    ServerEvent::PublicLobbies(lobbies) => {
                        send_event(ServerEvent::PublicLobbies(lobbies));
                    }
                    ServerEvent::PlayerEdited(id) => {
                        send_event(ServerEvent::PlayerEdited(id));
                    }
                    ServerEvent::LobbyMessage(msg) => {
                        send_event(ServerEvent::LobbyMessage(msg));
                    }
                    ServerEvent::LobbyQueued(id) => {
                        send_event(ServerEvent::LobbyQueued(id));
                    }
                    ServerEvent::MatchFound(lobby) => {
                        send_event(ServerEvent::MatchFound(lobby));
                    }
                    ServerEvent::MatchNotFound => {
                        send_event(ServerEvent::MatchNotFound);
                    }
                    ServerEvent::QueueStopped(id) => {
                        send_event(ServerEvent::QueueStopped(id));
                    }
                    ServerEvent::LeftGame(lobbies) => {
                        send_event(ServerEvent::LeftGame(lobbies));
                    }
                    ServerEvent::LobbyInfo(id) => {
                        send_event(ServerEvent::LobbyInfo(id));
                    }
                    _ => {}
                }
            }
        }
    }
}


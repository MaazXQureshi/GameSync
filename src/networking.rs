use crate::store::DataStore;
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{GameSyncError, print_error};
use crate::lobby::{*};

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    UserMessage(String),
    SelfPlayer(String),
    NewPlayer(String),
    LobbyCreated(Uuid), // Returns Lobby ID
    LobbyJoined(Uuid),
    LobbyDeleted(Uuid),
    LobbyLeft(Uuid),
    LobbyInvited(Uuid), // Lobby ID
    PublicLobbies(Vec<Lobby>),
    PlayerEdited(Uuid)
}

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    Broadcast(String),
    SendTo(String, String), // To, Msg
    // more game events to send b/w clients and server
    // TODO: Make type aliases for Uuid -> LobbyID and PlayerID to avoid confusion 
    CreateLobby(Uuid, LobbyParams), // Uuid = Sender ID
    JoinLobby(Uuid, Uuid), // Sender ID, Lobby ID
    DeleteLobby(Uuid, Uuid), // Sender ID, Lobby ID
    LeaveLobby(Uuid, Uuid), // Sender ID, Lobby ID
    InviteLobby(Uuid, Uuid, Uuid),  // Sender ID, Lobby ID, Invitee ID
    GetPublicLobbies(Uuid, Region), // Sender ID
    EditPlayer(Player)
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
                        self.data_store.delete_player(endpoint);
                        // TODO: Add check for if player belongs to any lobbies, and to remove them from it
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
            ClientEvent::CreateLobby(uuid, lobby_params) => { // TODO: Change uuid to playerID to avoid confusion
                println!("CreateLobby => Player ID: {:?} LobbyParams: {:?}", uuid, lobby_params);
                self.create_lobby(uuid, &lobby_params)?;
            },
            ClientEvent::JoinLobby(uuid, lobby_id) => {
                println!("JoinLobby => Player ID: {:?} LobbyID: {:?}", uuid, lobby_id);
                self.join_lobby(uuid, lobby_id)?;
            },
            ClientEvent::DeleteLobby(uuid, lobby_id) => {
                println!("DeleteLobby => Player ID: {:?} LobbyID: {:?}", uuid, lobby_id);
                self.delete_lobby(uuid, lobby_id)?;
            },
            ClientEvent::LeaveLobby(uuid, lobby_id) => {
                println!("LeaveLobby => Player ID: {:?} LobbyID: {:?}", uuid, lobby_id);
                self.leave_lobby(uuid, lobby_id)?;
            },
            ClientEvent::InviteLobby(uuid, lobby_id, invitee_id) => {
                println!("InviteLobby => Player ID: {:?} LobbyID: {:?} InviteeID: {:?}", uuid, lobby_id, invitee_id);
                self.invite_lobby(uuid, lobby_id, invitee_id)?;
            },
            ClientEvent::GetPublicLobbies(uuid, region) => {
                println!("GetPublicLobbies => Player ID: {:?} Region: {:?}", uuid, region);
                self.get_public_lobbies(uuid, region)?;
            },
            ClientEvent::EditPlayer(player) => {
                println!("EditPlayer => Player: {:?}", player);
                self.edit_player(player)?;
            },
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

        // Default player
        let player_id = Uuid::parse_str(&id)?;
        let rating = 0;
        let player = Player { player_id, rating };
        self.data_store.add_player(player_id, player);
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

    pub fn create_lobby(&mut self, uuid: Uuid, lobby_params: &LobbyParams) -> Result<(), GameSyncError> {
        let lobby_id = Uuid::new_v4();
        match self.data_store.get_player(uuid) {
            Some(player) => {
                let lobby = Lobby {
                    params: lobby_params.clone(),
                    leader: player,
                    status: LobbyStatus::Idle,
                    player_list: vec![player]
                };    
                self.data_store.create_lobby(lobby_params.region, lobby_id, lobby);
                self.data_store.create_region_lobby(lobby_id, lobby_params.region);
                // self.data_store.print_global_lobby_map(); // Uncomment for debugging
                let event = ServerEvent::LobbyCreated(lobby_id);
                self.send_to_client(&uuid.to_string(), event)?;
            },
            None => {
                return Err(GameSyncError::PlayerFindError)
            }
        }
        Ok(())
    }

    pub fn join_lobby(&mut self, uuid: Uuid, lobby_id: Uuid) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) { // Sorry, these 3 matches look ugly. I'll clean them up later if I can find a better way
            Some(region) => {
                let mut lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                match lobby.player_list.iter().find(|&p| p.player_id == uuid) { // If player is part of this lobby
                    Some(_) => {
                        // TODO: Add error checking for when lobby full. Where to initialize this max lobby value -> currently thinking of GameServer construction
                        match self.data_store.get_player(uuid) {
                            Some(player) => {
                                lobby.player_list.push(player);
                                self.data_store.edit_lobby(region, lobby_id, lobby)?;
                            },
                            None => {
                                return Err(GameSyncError::PlayerFindError)
                            }
                        }
                    },
                    None => return Err(GameSyncError::LobbyFindError)
                }
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        let event = ServerEvent::LobbyJoined(lobby_id);
        self.send_to_client(&uuid.to_string(), event)?;
        Ok(())
    }

    pub fn delete_lobby(&mut self, uuid: Uuid, lobby_id: Uuid) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                let lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                self.check_lobby_owner(uuid, lobby.leader.player_id)?;
                self.data_store.delete_lobby(region, lobby_id)?;
                self.data_store.delete_region_lobby(lobby_id)?;
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        let event = ServerEvent::LobbyDeleted(lobby_id);
        self.send_to_client(&uuid.to_string(), event)?;
        Ok(())
    }

    pub fn leave_lobby(&mut self, uuid: Uuid, lobby_id: Uuid) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                let mut lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
            
                // If last player or leader leaves, delete lobby and send message to all players in lobby
                if lobby.player_list.is_empty() || lobby.leader.player_id == uuid {
                    self.data_store.delete_lobby(region, lobby_id)?;
                    self.data_store.delete_region_lobby(lobby_id)?;
                    for player in lobby.player_list.iter() {
                        self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyLeft(lobby_id))?;
                        self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyDeleted(lobby_id))?;
                    }
                } else {
                    lobby.player_list.retain(|&player| player.player_id != uuid); // Remove player
                    self.data_store.edit_lobby(region, lobby_id, lobby)?;
                    self.send_to_client(&uuid.to_string(), ServerEvent::LobbyLeft(lobby_id))?;
                }
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn check_lobby_owner(&self, player_id: Uuid, lobby_owner: Uuid) -> Result<(), GameSyncError> {
        if lobby_owner != player_id {
            return Err(GameSyncError::LobbyOwnerError)
        } else {
            Ok(())
        }
    }

    pub fn invite_lobby(&mut self, uuid: Uuid, lobby_id: Uuid, invitee_id: Uuid) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                let lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                match lobby.player_list.iter().find(|&p| p.player_id == uuid) { // If player is part of this lobby
                    Some(_) => {
                        self.send_to_client(&invitee_id.to_string(), ServerEvent::LobbyInvited(lobby_id))?;
                    },
                    None => return Err(GameSyncError::LobbyFindError)
                }
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn get_public_lobbies(&mut self, uuid: Uuid, region: Region) -> Result<(), GameSyncError> {
        let public_lobbies = self.data_store.get_region_lobbies(region);
        let event = ServerEvent::PublicLobbies(public_lobbies);
        self.send_to_client(&uuid.to_string(), event)?;
        Ok(())
    }

    pub fn edit_player(&mut self, player: Player) -> Result<(), GameSyncError> {
        self.data_store.edit_player(player);
        let event = ServerEvent::PlayerEdited(player.player_id);
        self.send_to_client(&player.player_id.to_string(), event)?;
        Ok(())
    }
}

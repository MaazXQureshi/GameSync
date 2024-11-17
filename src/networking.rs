use crate::server_params::ServerParams;
use crate::store::DataStore;
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{GameSyncError, print_error};
use crate::lobby::{*};

type PlayerID = Uuid;
type LobbyID = Uuid;

#[derive(Serialize, Deserialize)]
pub enum ServerEvent {
    UserMessage(String),
    SelfPlayer(String),
    NewPlayer(String),
    LobbyCreated(LobbyID), // Lobby ID
    LobbyJoined(LobbyID), // Lobby ID
    LobbyDeleted(LobbyID), // Lobby ID
    LobbyLeft(LobbyID), // Lobby ID
    LobbyInvited(LobbyID), // Lobby ID
    PublicLobbies(Vec<Lobby>),
    PlayerEdited(PlayerID), // Player ID
    LobbyMessage(String), // Msg
    LobbyQueued(LobbyID),
    MatchFound(Lobby), // Opponent lobby
    MatchNotFound,
}

#[derive(Serialize, Deserialize)]
pub enum ClientEvent {
    Broadcast(String),
    SendTo(String, String), // To, Msg
    // more game events to send b/w clients and server
    CreateLobby(PlayerID, LobbyParams),
    JoinLobby(PlayerID, LobbyID),
    DeleteLobby(PlayerID, LobbyID),
    LeaveLobby(PlayerID, LobbyID),
    InviteLobby(PlayerID, LobbyID, PlayerID),  // Sender ID, Lobby ID, Invitee ID
    GetPublicLobbies(PlayerID, Region),
    EditPlayer(Player),
    MessageLobby(PlayerID, LobbyID, String), // Sender ID, Lobby ID, Message
    QueueLobby(PlayerID, LobbyID),
    CheckMatch(PlayerID, LobbyID, usize),
}

pub struct Websocket {
    handler: NodeHandler<ClientEvent>,
    listener: Option<NodeListener<ClientEvent>>,
    data_store: DataStore,
}

impl Websocket {
    pub fn new(port: &str, server_params: ServerParams) -> Result<Self, GameSyncError> {
        let (handler, listener) = node::split();
        handler.network().listen(Transport::Ws, String::from("0.0.0.0") + ":" + port)?;
        let data_store = DataStore::new(server_params);
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

                        // Delete/leave any lobbies the player belongs to
                        match self.clean_up_player(endpoint) {
                            Ok(_) => {},
                            Err(e) => print_error(e),
                        }
                        self.data_store.remove_user_endpoint(endpoint); // Do this at the end
                        println!("Cleaned up datastructures");
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
            ClientEvent::CreateLobby(player_id, lobby_params) => {
                println!("CreateLobby => Player ID: {:?} LobbyParams: {:?}", player_id, lobby_params);
                self.create_lobby(player_id, &lobby_params)?;
            },
            ClientEvent::JoinLobby(player_id, lobby_id) => {
                println!("JoinLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.join_lobby(player_id, lobby_id)?;
            },
            ClientEvent::DeleteLobby(player_id, lobby_id) => {
                println!("DeleteLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.delete_lobby(player_id, lobby_id)?;
            },
            ClientEvent::LeaveLobby(player_id, lobby_id) => {
                println!("LeaveLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.leave_lobby(player_id, lobby_id)?;
            },
            ClientEvent::InviteLobby(player_id, lobby_id, invitee_id) => {
                println!("InviteLobby => Player ID: {:?} LobbyID: {:?} InviteeID: {:?}", player_id, lobby_id, invitee_id);
                self.invite_lobby(player_id, lobby_id, invitee_id)?;
            },
            ClientEvent::GetPublicLobbies(player_id, region) => {
                println!("GetPublicLobbies => Player ID: {:?} Region: {:?}", player_id, region);
                self.get_public_lobbies(player_id, region)?;
            },
            ClientEvent::EditPlayer(player) => {
                println!("EditPlayer => Player: {:?}", player);
                self.edit_player(player)?;
            },
            ClientEvent::MessageLobby(player_id, lobby_id, message) => {
                println!("MessageLobby => Player ID: {:?} LobbyID: {:?} Message: {}", player_id, lobby_id, message);
                self.message_lobby(player_id, lobby_id, message)?;
            },
            ClientEvent::QueueLobby(player_id, lobby_id) => {
                println!("QueueLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                // TODO: Implement QueueLobby. Check if lobby full, change lobby state to Queueing and add to match queue
            },
            ClientEvent::CheckMatch(player_id, lobby_id, threshold) => {
                println!("CheckMatch => Player ID: {:?} LobbyID: {:?} Threshold {}", player_id, lobby_id, threshold);
                // TODO: Implement CheckMatch. Check queue to see if match is found using threshold for average rating calculation
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

    pub fn send_to_client(&self, to: &String, event: ServerEvent) -> Result<(), GameSyncError> {
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

    pub fn create_lobby(&mut self, player_id: PlayerID, lobby_params: &LobbyParams) -> Result<(), GameSyncError> {
        match self.data_store.get_player_info(player_id) {
            Some(player_info) => {
                match player_info.1 {
                    Some(_) => { // Checks if player is already in a lobby
                        return Err(GameSyncError::LobbyCreateError);
                    },
                    None => {
                        let lobby_id = Uuid::new_v4();
                        let lobby = Lobby {
                            lobby_id: lobby_id.clone(),
                            params: lobby_params.clone(),
                            leader: player_info.0,
                            status: LobbyStatus::Idle,
                            player_list: vec![player_info.0]
                        };    
                        self.data_store.create_lobby(lobby_params.region, lobby_id, lobby.clone());
                        self.data_store.create_region_lobby(lobby_id, lobby_params.region);
                        self.data_store.edit_player(player_id, None, Some(lobby.clone()));
                        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
                        let event = ServerEvent::LobbyCreated(lobby_id);
                        self.send_to_client(&player_id.to_string(), event)?;
                    }
                }
            },
            None => {
                return Err(GameSyncError::PlayerFindError)
            }
        }
        Ok(())
    }

    pub fn join_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        match self.data_store.get_player_info(player_id) {
            Some(player_info) => {
                match player_info.1 {
                    Some(_) => { // Checks if player is already in a lobby
                        return Err(GameSyncError::LobbyJoinError);
                    },
                    None => { // Player can join lobby if it exists and there is space
                        match self.data_store.get_region_lobby(&lobby_id) {
                            Some(region) => { // Lobby exists
                                let mut lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                                if lobby.player_list.len() == self.data_store.lobby_size() { // Check if lobby if full
                                    return Err(GameSyncError::LobbyFullError)
                                }
                                lobby.player_list.push(player_info.0);
                                self.data_store.edit_lobby(region, lobby_id, lobby.clone())?;
                                self.data_store.edit_player(player_id, None, Some(lobby.clone()));
                                for player in lobby.player_list.iter() { // Send join notification to all players in lobby
                                    self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyJoined(lobby_id))?;
                                }
                            },
                            None => return Err(GameSyncError::LobbyFindError)
                        }
                    }
                }
            },
            None => {
                return Err(GameSyncError::PlayerFindError)
            }
        }
        Ok(())
    }

    pub fn delete_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                let lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                if player_id != lobby.leader.player_id {
                    return Err(GameSyncError::LobbyOwnerError)
                }
                self.data_store.delete_lobby(region, lobby_id)?;
                self.data_store.delete_region_lobby(lobby_id)?;
                for player in lobby.player_list.iter() { // Remove all players in lobby and send notification
                    self.data_store.remove_player_lobby(player.player_id);
                    self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyLeft(lobby_id))?;
                    self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyDeleted(lobby_id))?;
                }
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        let event = ServerEvent::LobbyDeleted(lobby_id);
        self.send_to_client(&player_id.to_string(), event)?;
        Ok(())
    }

    pub fn leave_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                let mut lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                // If leader leaves, delete lobby and send message to all players in lobby
                // Leader will be the last one to leave. If it isn't, lobby will be deleted regardless
                match lobby.player_list.iter().find(|&p| p.player_id == player_id) { // If player is part of this lobby
                    Some(_) => {
                        if player_id == lobby.leader.player_id {
                            self.data_store.delete_lobby(region, lobby_id)?;
                            self.data_store.delete_region_lobby(lobby_id)?;
                            for player in lobby.player_list.iter() { // Remove all players in lobby and send notification
                                self.data_store.remove_player_lobby(player.player_id);
                                self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyLeft(lobby_id))?;
                                self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyDeleted(lobby_id))?;
                            }
                        } else {
                            lobby.player_list.retain(|&player| player.player_id != player_id); // Remove player
                            self.data_store.edit_lobby(region, lobby_id, lobby)?;
                            self.data_store.remove_player_lobby(player_id);
                            self.send_to_client(&player_id.to_string(), ServerEvent::LobbyLeft(lobby_id))?;
                        }
                    },
                    None => return Err(GameSyncError::LobbyInviteError)
                } 
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn message_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID, message: String) -> Result<(), GameSyncError> {
        match self.data_store.get_player_info(player_id) {
            Some(player_info) => {
                match player_info.1 {
                    Some(lobby) => {
                        if lobby_id != lobby.lobby_id {
                            return Err(GameSyncError::LobbyMessageError)
                        } else {
                            for player in lobby.player_list.iter() { // Send message to all players in lobby
                                self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyMessage(message.clone()))?; // Can leave as clone for now. Optionally figure out better way
                            }
                        }
                    },
                    None => {
                        return Err(GameSyncError::LobbyPlayerError)
                    }
                }
            },
            None => {
                return Err(GameSyncError::PlayerFindError)
            }
        }
        Ok(())
    }

    pub fn invite_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID, invitee_id: PlayerID) -> Result<(), GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                let lobby = self.data_store.get_lobby(region, lobby_id).unwrap(); // Guaranteed to return
                match lobby.player_list.iter().find(|&p| p.player_id == player_id) { // If player is part of this lobby
                    Some(_) => {
                        self.send_to_client(&invitee_id.to_string(), ServerEvent::LobbyInvited(lobby_id))?;
                    },
                    None => return Err(GameSyncError::LobbyInviteError)
                }
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn get_public_lobbies(&mut self, player_id: PlayerID, region: Region) -> Result<(), GameSyncError> {
        let public_lobbies = self.data_store.get_region_lobbies(region);
        let event = ServerEvent::PublicLobbies(public_lobbies);
        self.send_to_client(&player_id.to_string(), event)?;
        Ok(())
    }

    pub fn edit_player(&mut self, player: Player) -> Result<(), GameSyncError> {
        self.data_store.edit_player(player.player_id, Some(player), None);
        let event = ServerEvent::PlayerEdited(player.player_id);
        self.send_to_client(&player.player_id.to_string(), event)?;
        Ok(())
    }

    pub fn clean_up_player(&mut self, endpoint: Endpoint) -> Result<(), GameSyncError> {
        let player_id = self.data_store.get_user(endpoint);
        if let Some(player_id) = player_id {
            match self.data_store.get_player_info(player_id) {
                Some(player_info) => {
                    match player_info.1 { 
                        Some(mut lobby) => { // If user is part of a lobby, need to delete if owner, leave if in party
                            if player_id == lobby.leader.player_id { // If user is leader of a lobby, delete and kick party
                                self.data_store.delete_lobby(lobby.params.region, lobby.lobby_id)?;
                                self.data_store.delete_region_lobby(lobby.lobby_id)?;
                                for player in lobby.player_list.iter() { // Remove all players from lobby and send messages to all connected users
                                    if player.player_id != player_id { 
                                        self.data_store.remove_player_lobby(player.player_id);
                                        self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyLeft(lobby.lobby_id))?;
                                        self.send_to_client(&player.player_id.to_string(), ServerEvent::LobbyDeleted(lobby.lobby_id))?;
                                    }
                                }
                                self.data_store.delete_player(player_id); // Delete player at the end
                            } 
                            else { // If user is part of a lobby (i.e. not a leader)
                                lobby.player_list.retain(|&player| player.player_id != player_id); // Remove player
                                self.data_store.edit_lobby(lobby.params.region, lobby.lobby_id, lobby)?;
                                self.data_store.remove_player_lobby(player_id);
                            }
                        },
                        None => { // If user not part of a lobby, can just remove from player map and return. Other data structures should ideally not have this player in them
                            self.data_store.delete_player(player_id)
                        }
                    }
                },
                None => {
                    println!("Player not found in clean-up. This should ideally never happen. If this happens, some datastructure operations have gone wrong, particularly player_map");
                }
            }
        } else {
            println!("Player not found in clean-up. This should ideally never happen. If this happens, some datastructure operations have gone wrong, particularly endpoint_user_map");
        }
        Ok(())
    }
}

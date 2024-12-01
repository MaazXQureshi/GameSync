use std::thread;
use std::time::Duration;
use crate::server_params::ServerParams;
use crate::store::DataStore;
use message_io::network::{Endpoint, NetEvent, SendStatus, Transport};
use message_io::node::{self, NodeEvent, NodeHandler, NodeListener};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{GameSyncError, print_error};
use crate::lobby::{*};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerEvent {
    UserMessage(PlayerID, String), // From, Msg
    SelfPlayer(String),
    NewPlayer(String),
    LobbyCreated(Lobby), // Lobby
    LobbyJoined(PlayerID, LobbyID), // Lobby ID
    LobbyDeleted(LobbyID), // Lobby ID
    LobbyLeft(PlayerID, LobbyID), // Lobby ID
    LobbyInvited(LobbyID), // Lobby ID
    PublicLobbies(Vec<Lobby>),
    PlayerEdited(PlayerID), // Player ID
    LobbyMessage(PlayerID, String), // From, Msg
    LobbyQueued(LobbyID),
    MatchFound(Lobby), // Opponent lobby
    MatchNotFound,
    QueueStopped(LobbyID),
    LeftGame(LobbyID),
    LobbyInfo(Lobby)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientEvent {
    Broadcast(String),
    SendTo(String, String), // To, Msg
    // more game events to send b/w clients and server
    CreateLobby(LobbyParams),
    JoinLobby(LobbyID),
    DeleteLobby(LobbyID),
    LeaveLobby(LobbyID),
    InviteLobby(LobbyID, PlayerID),  // Sender ID, Lobby ID, Invitee ID
    GetPublicLobbies(Region),
    EditPlayer(Player),
    MessageLobby(LobbyID, String), // Sender ID, Lobby ID, Message
    QueueLobby(LobbyID),
    CheckMatch(LobbyID, Option<usize>), // Sender ID, Lobby ID, Threshold
    StopQueue(LobbyID),
    LeaveGameAsLobby(LobbyID),
    GetLobbyInfo(LobbyID),
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
                    NetEvent::Message(endpoint, message) => {
                        let msg = serde_json::from_slice(&message).unwrap();

                        match self.handle_messages(endpoint, msg) {
                            Ok(_) => {},
                            Err(e) => print_error(e),
                        }
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
                },
                _ => {}
            }
        );
    }

    fn handle_messages(&mut self, endpoint: Endpoint, event: ClientEvent) -> Result<(), GameSyncError> {
        let player_id = match self.data_store.get_user(endpoint) {
            Some(user) => user,
            None => return Err(GameSyncError::UserNotFound)
        };
        match event {
            ClientEvent::Broadcast(message) => {
                println!("Broadcasting message: {}", message);
                let event = ServerEvent::UserMessage(player_id, message);
                self.send_to_all_clients(endpoint, event)?;
            }
            ClientEvent::SendTo(uuid, message) => {
                println!("To: {} Message: {}", uuid, message);
                let event = ServerEvent::UserMessage(player_id, message);
                self.send_to_client(&uuid, event)?;
            }
            ClientEvent::CreateLobby(lobby_params) => {
                println!("CreateLobby => Player ID: {:?} LobbyParams: {:?}", player_id, lobby_params);
                self.create_lobby(player_id, &lobby_params)?;
            },
            ClientEvent::JoinLobby(lobby_id) => {
                println!("JoinLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.join_lobby(player_id, lobby_id)?;
            },
            ClientEvent::DeleteLobby(lobby_id) => {
                println!("DeleteLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.delete_lobby(player_id, lobby_id)?;
            },
            ClientEvent::LeaveLobby(lobby_id) => {
                println!("LeaveLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.leave_lobby(player_id, lobby_id)?;
            },
            ClientEvent::InviteLobby(lobby_id, invitee_id) => {
                println!("InviteLobby => Player ID: {:?} LobbyID: {:?} InviteeID: {:?}", player_id, lobby_id, invitee_id);
                self.invite_lobby(player_id, lobby_id, invitee_id)?;
            },
            ClientEvent::GetPublicLobbies(region) => {
                println!("GetPublicLobbies => Player ID: {:?} Region: {:?}", player_id, region);
                self.get_public_lobbies(player_id, region)?;
            },
            ClientEvent::EditPlayer(player) => {
                println!("EditPlayer => Player: {:?}", player);
                self.edit_player(player)?;
            },
            ClientEvent::MessageLobby(lobby_id, message) => {
                println!("MessageLobby => Player ID: {:?} LobbyID: {:?} Message: {}", player_id, lobby_id, message);
                self.message_lobby(player_id, lobby_id, message)?;
            },
            ClientEvent::QueueLobby(lobby_id) => {
                println!("QueueLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.queue_lobby(player_id, lobby_id)?;
            },
            ClientEvent::CheckMatch(lobby_id, threshold) => {
                println!("CheckMatch => Player ID: {:?} LobbyID: {:?} Threshold {:?}", player_id, lobby_id, threshold);
                let threshold = threshold.unwrap_or(0);
                self.check_match(player_id, lobby_id, threshold)?;
            },
            ClientEvent::StopQueue(lobby_id) => {
                println!("StopQueue => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.stop_queue(player_id, lobby_id)?;
            },
            ClientEvent::LeaveGameAsLobby(lobby_id) => {
                println!("LeaveGameAsLobby => Player ID: {:?} LobbyID: {:?}", player_id, lobby_id);
                self.leave_game_as_lobby(player_id, lobby_id)?;
            },
            ClientEvent::GetLobbyInfo(lobby_id) => {
                println!("GetLobbyInfo => Lobby ID: {:?}", lobby_id);
                self.get_lobby_info(player_id, lobby_id)?;
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

        // send new player id to all existing clients (wait a bit for client to get set up before receiving events)
        thread::sleep(Duration::from_millis(100));
        let event = ServerEvent::NewPlayer(id.to_string());
        self.send_to_all_clients(endpoint, event)?;

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

    pub fn send_to_all_clients(&mut self, msg_sender: Endpoint, event: ServerEvent) -> Result<SendStatus, GameSyncError> {
        let endpoints = self.data_store.get_all_user_endpoints();
        let payload = serde_json::to_string(&event)?;

        for endpoint in endpoints {
            if (msg_sender == endpoint) {
                continue;
            }
            let status = self.handler.network().send(endpoint, payload.as_ref());
            if status != SendStatus::Sent {
                return Err(GameSyncError::SendError)
            }
        }

        Ok(SendStatus::Sent)
    }

    pub fn create_lobby(&mut self, player_id: PlayerID, lobby_params: &LobbyParams) -> Result<(), GameSyncError> {
        let player_info = self.find_player(player_id)?;
        match player_info.1 {
            Some(_) => { // Checks if player is already in a lobby
                return Err(GameSyncError::LobbyCreateError);
            },
            None => {
                let lobby_id = Uuid::new_v4();
                let lobby = Lobby {
                    lobby_id: lobby_id.clone(),
                    params: lobby_params.clone(),
                    leader: player_id,
                    status: LobbyStatus::Idle,
                    player_list: vec![player_id],
                    queue_threshold: 0
                };
                self.data_store.create_lobby(lobby_params.region, lobby_id, lobby.clone());
                self.data_store.create_region_lobby(lobby_id, lobby_params.region);
                self.data_store.edit_player(player_id, None, Some(lobby_id.clone()));
                // self.data_store.print_global_lobby_map(); // Uncomment for debugging
                let event = ServerEvent::LobbyCreated(lobby);
                self.send_to_client(&player_id.to_string(), event)?;
            }
        }
        Ok(())
    }

    pub fn join_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let player_info = self.find_player(player_id)?;
        match player_info.1 {
            Some(_) => { // Checks if player is already in a lobby
                return Err(GameSyncError::LobbyJoinError);
            },
            None => { // Player can join lobby if it exists and there is space
                let region = self.find_region_lobby(lobby_id)?;
                let mut lobby = self.find_lobby(region, lobby_id)?;
                if lobby.player_list.len() == self.data_store.lobby_size() { // Check if lobby if full
                    return Err(GameSyncError::LobbyFullError)
                }
                lobby.player_list.push(player_id);
                self.data_store.edit_lobby(region, lobby_id, lobby.clone())?;
                self.data_store.edit_player(player_id, None, Some(lobby_id.clone()));
                for player_id_lobby in lobby.player_list.iter() { // Send join notification to all players in lobby
                    self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyJoined(player_id, lobby_id))?;
                }
            }
        }
        Ok(())
    }

    pub fn delete_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let lobby = self.find_lobby(region, lobby_id)?;
        if player_id != lobby.leader {
            return Err(GameSyncError::LobbyOwnerError)
        }
        if lobby.status != LobbyStatus::Idle {
            return Err(GameSyncError::LobbyDeleteError)
        }
        self.data_store.delete_lobby(region, lobby_id)?;
        self.data_store.delete_region_lobby(lobby_id)?;
        for player_id_lobby in lobby.player_list.iter() { // Remove all players in lobby and send notification
            self.data_store.remove_player_lobby(player_id_lobby.clone());
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyLeft(player_id_lobby.clone(), lobby_id))?;
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyDeleted(lobby_id))?;
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn leave_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let mut lobby = self.find_lobby(region, lobby_id)?;
        // If leader leaves, delete lobby and send message to all players in lobby
        // Leader will be the last one to leave. If it isn't, lobby will be deleted regardless
        match lobby.player_list.iter().find(|&&p| p == player_id) { // If player is part of this lobby
            Some(_) => {
                // Regardless of whether player is leader or part of lobby, if lobby is in queue, it will be removed
                // If lobby is queueing, remove from queue and make idle.
                let mut lobby_queueing: bool = false;
                if lobby.status == LobbyStatus::Queueing {
                    lobby.status = LobbyStatus::Idle;
                    match lobby.params.mode {
                        GameMode::Casual => {
                            self.data_store.remove_casual_lobby(region, lobby.lobby_id);
                        },
                        GameMode::Competitive => {
                            self.data_store.remove_competitive_lobby(region, lobby.lobby_id);
                        }
                    }
                    lobby_queueing = true;
                }

                // If lobby was In-game or Idle, continue as normal. If leader was the one who left, remove all players from lobby
                if player_id == lobby.leader {
                    self.data_store.delete_lobby(region, lobby_id)?;
                    self.data_store.delete_region_lobby(lobby_id)?;
                    for player_id_lobby in lobby.player_list.iter() { // Remove all players in lobby and send notification
                        self.data_store.remove_player_lobby(player_id_lobby.clone());
                        self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyLeft(player_id_lobby.clone(), lobby_id))?;
                        self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyDeleted(lobby_id))?;
                    }
                }
                else {
                    lobby.player_list.retain(|&player| player != player_id); // Remove player
                    self.data_store.edit_lobby(region, lobby_id, lobby.clone())?; // Edit lobby
                    self.data_store.remove_player_lobby(player_id); // Remove the leaving user
                    self.send_to_client(&player_id.to_string(), ServerEvent::LobbyLeft(player_id, lobby_id))?; // Notify the leaving user
                    for player_id_lobby in lobby.player_list.iter() { // Notify all remaining players in lobby
                        if lobby_queueing {
                            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::QueueStopped(lobby_id))?;
                        }
                        self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyLeft(player_id, lobby_id))?; // Notify that this particular player has left
                    }
                }
            },
            None => return Err(GameSyncError::LobbyInviteError)
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn message_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID, message: String) -> Result<(), GameSyncError> {
        let player_info = self.find_player(player_id)?;
        match player_info.1 {
            Some(player_lobby) => {
                if lobby_id != player_lobby {
                    return Err(GameSyncError::LobbyMessageError)
                }
                else {
                    let region = self.find_region_lobby(lobby_id)?;
                    let lobby = self.find_lobby(region, lobby_id)?;
                    for player_id_lobby in lobby.player_list.iter() { // Send message to all players in lobby
                        self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyMessage(player_id, message.clone()))?; // Can leave as clone for now. Optionally figure out better way
                    }
                }
            },
            None => {
                return Err(GameSyncError::LobbyPlayerError)
            }
        }
        Ok(())
    }

    pub fn invite_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID, invitee_id: PlayerID) -> Result<(), GameSyncError> {
        let player_lobby = self.find_player(player_id)?.1;
        match player_lobby {
            Some(player_lobby) => {
                if player_lobby == lobby_id {
                    self.send_to_client(&invitee_id.to_string(), ServerEvent::LobbyInvited(lobby_id))?;
                }
                else {
                    return Err(GameSyncError::LobbyCurInviteError)
                }
            },
            None => return Err(GameSyncError::LobbyInviteError)
        }
        Ok(())
    }

    pub fn get_public_lobbies(&mut self, player_id: PlayerID, region: Region) -> Result<(), GameSyncError> {
        let public_lobbies = self.data_store.get_region_lobbies(region);
        let event = ServerEvent::PublicLobbies(public_lobbies);
        self.send_to_client(&player_id.to_string(), event)?;
        Ok(())
    }

    pub fn edit_player(&mut self, player: Player) -> Result<(), GameSyncError> {
        let player_info = self.find_player(player.player_id)?;
        match player_info.1 {
            Some(lobby_id) => { // Checks if player is already in a lobby
                let region = self.find_region_lobby(lobby_id)?;
                let lobby = self.find_lobby(region, lobby_id)?;
                if lobby.status != LobbyStatus::Idle {
                    return Err(GameSyncError::PlayerEditError);
                }
            },
            None => { // Player can be edited if not in lobby
            }
        }
        self.data_store.edit_player(player.player_id, Some(player), None);
        let event = ServerEvent::PlayerEdited(player.player_id);
        self.send_to_client(&player.player_id.to_string(), event)?;
        Ok(())
    }

    pub fn clean_up_player(&mut self, endpoint: Endpoint) -> Result<(), GameSyncError> {
        let player_id = self.data_store.get_user(endpoint);
        if let Some(player_id) = player_id {
            let player_info = self.find_player(player_id)?;
            match player_info.1 {
                Some(player_lobby) => { // If user is part of a lobby, need to delete if owner, leave if in party
                    // If the lobby was queuing then remove from queues and message players
                    let region = self.find_region_lobby(player_lobby)?;
                    let mut lobby = self.find_lobby(region, player_lobby)?;
                    let mut lobby_queueing: bool = false;
                    if lobby.status == LobbyStatus::Queueing {
                        lobby.status = LobbyStatus::Idle;
                        match lobby.params.mode {
                            GameMode::Casual => {
                                self.data_store.remove_casual_lobby(lobby.params.region, lobby.lobby_id);
                            },
                            GameMode::Competitive => {
                                self.data_store.remove_competitive_lobby(lobby.params.region, lobby.lobby_id);
                            }
                        }
                        lobby_queueing = true;
                    }
                    if player_id == lobby.leader { // If user is leader of a lobby, delete and kick party
                        self.data_store.delete_lobby(lobby.params.region, lobby.lobby_id)?;
                        self.data_store.delete_region_lobby(lobby.lobby_id)?;
                        for player_id_lobby in lobby.player_list.iter() { // Remove all players from lobby and send messages to all connected users
                            if *player_id_lobby != player_id {
                                self.data_store.remove_player_lobby(player_id_lobby.clone());
                                self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyLeft(player_id_lobby.clone(), lobby.lobby_id))?;
                                self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyDeleted(lobby.lobby_id))?;
                            }
                        }
                        self.data_store.delete_player(player_id); // Delete player at the end
                    }
                    else { // If user is part of a lobby (i.e. not a leader)
                        lobby.player_list.retain(|&player| player != player_id); // Remove player
                        self.data_store.edit_lobby(lobby.params.region, lobby.lobby_id, lobby.clone())?; // Edit lobby
                        self.data_store.remove_player_lobby(player_id); // Remove the leaving user
                        for player_id_lobby in lobby.player_list.iter() { // Notify all remaining players in lobby
                            if lobby_queueing {
                                self.send_to_client(&player_id_lobby.to_string(), ServerEvent::QueueStopped(lobby.lobby_id))?;
                            }
                            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyLeft(player_id, lobby.lobby_id))?; // Notify that this particular player has left
                        }
                        self.data_store.delete_player(player_id); // Delete player at the end
                    }
                },
                None => { // If user not part of a lobby, can just remove from player map and return. Other data structures should ideally not have this player in them
                    self.data_store.delete_player(player_id)
                }
            }
        }
        else {
            println!("Player not found in clean-up. This should ideally never happen. If this happens, some datastructure operations have gone wrong, particularly endpoint_user_map");
        }
        Ok(())
    }

    pub fn queue_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let mut lobby = self.find_lobby(region, lobby_id)?;
        if player_id != lobby.leader {
            return Err(GameSyncError::LobbyOwnerError)
        }
        if lobby.status != LobbyStatus::Idle {
            return Err(GameSyncError::LobbyQueueError)
        }
        if lobby.player_list.len() != self.data_store.lobby_size() {
            return Err(GameSyncError::LobbySizeError)
        }

        lobby.status = LobbyStatus::Queueing;

        match lobby.params.mode {
            GameMode::Casual => {
                self.data_store.add_casual_lobby(region, lobby.clone());
            },
            GameMode::Competitive => {
                self.data_store.add_competitive_lobby(region, lobby.clone());
            }
        }

        for player_id_lobby in lobby.player_list.iter() { // Edit and Message all players in lobby
            self.data_store.edit_lobby(region, lobby_id, lobby.clone())?;
            self.data_store.edit_player(player_id_lobby.clone(), None, Some(lobby_id.clone()));
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LobbyQueued(lobby_id))?;
        }
        Ok(())
    }

    pub fn check_match(&mut self, player_id: PlayerID, lobby_id: LobbyID, threshold: usize) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let lobby = self.find_lobby(region, lobby_id)?;
        if player_id != lobby.leader { // Only let leader check to avoid multiple map operations
            return Err(GameSyncError::LobbyOwnerError)
        }
        if lobby.status != LobbyStatus::Queueing {
            return Err(GameSyncError::LobbyCheckError)
        }

        // self.data_store.print_casual_lobbies();

        match lobby.params.mode {
            GameMode::Casual => {
                match self.data_store.check_casual_lobby(region, lobby_id) {
                    Some(lobbies) => {
                        self.finalize_match(region, lobbies, threshold)?;
                    },
                    None => {
                        self.send_to_client(&player_id.to_string(), ServerEvent::MatchNotFound)?;
                    }
                }
            },
            GameMode::Competitive => {
                match self.data_store.check_competitive_lobby(region, lobby_id, threshold) {
                    Some(lobbies) => {
                        self.finalize_match(region, lobbies, threshold)?;
                    },
                    None => {
                        self.send_to_client(&player_id.to_string(), ServerEvent::MatchNotFound)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn finalize_match(&mut self, region: Region, lobbies: (Lobby, Lobby), threshold: usize) -> Result<(), GameSyncError> {
        let (mut lobby1, mut lobby2) = lobbies;
        println!("Match found between lobby {} and {}", lobby1.lobby_id, lobby2.lobby_id);
        lobby1.status = LobbyStatus::Ingame;
        lobby1.queue_threshold = threshold;
        for player_id_lobby in lobby1.player_list.iter() { // Edit and Message all players in lobby
            self.data_store.edit_lobby(region, lobby1.lobby_id, lobby1.clone())?;
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::MatchFound(lobby2.clone()))?; // Opponent lobby
        }
        lobby2.status = LobbyStatus::Ingame;
        for player_id_lobby in lobby2.player_list.iter() { // Edit and Message all players in lobby
            self.data_store.edit_lobby(region, lobby2.lobby_id, lobby2.clone())?;
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::MatchFound(lobby1.clone()))?; // Opponent lobby
        }
        Ok(())
    }

    pub fn stop_queue(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let mut lobby = self.find_lobby(region, lobby_id)?;
        if player_id != lobby.leader {
            return Err(GameSyncError::LobbyOwnerError)
        }
        if lobby.status != LobbyStatus::Queueing {
            return Err(GameSyncError::LobbyStopError)
        }

        lobby.status = LobbyStatus::Idle;

        match lobby.params.mode {
            GameMode::Casual => {
                self.data_store.remove_casual_lobby(region, lobby_id);
            },
            GameMode::Competitive => {
                self.data_store.remove_competitive_lobby(region, lobby_id);
            }
        }

        for player_id_lobby in lobby.player_list.iter() { // Edit and Message all players in lobby
            self.data_store.edit_lobby(region, lobby_id, lobby.clone())?;
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::QueueStopped(lobby_id))?;
        }
        Ok(())
    }

    pub fn leave_game_as_lobby(&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let mut lobby = self.find_lobby(region, lobby_id)?;
        if player_id != lobby.leader { // Only lobby leader is allowed to leave game for the entire lobby
            return Err(GameSyncError::LobbyOwnerError)
        }
        if lobby.status != LobbyStatus::Ingame {
            return Err(GameSyncError::LeaveGameError)
        }

        lobby.status = LobbyStatus::Idle;

        for player_id_lobby in lobby.player_list.iter() { // Edit and Message all players in lobby
            self.data_store.edit_lobby(region, lobby_id, lobby.clone())?;
            self.send_to_client(&player_id_lobby.to_string(), ServerEvent::LeftGame(lobby_id))?;
        }
        // self.data_store.print_global_lobby_map(); // Uncomment for debugging
        Ok(())
    }

    pub fn get_lobby_info (&mut self, player_id: PlayerID, lobby_id: LobbyID) -> Result<(), GameSyncError> {
        let region = self.find_region_lobby(lobby_id)?;
        let lobby = self.find_lobby(region, lobby_id)?;
        let event = ServerEvent::LobbyInfo(lobby);
        self.send_to_client(&player_id.to_string(), event)?;
        Ok(())
    }

    /*  HELPER FUNCTIONS */

    fn find_region_lobby(&mut self, lobby_id: LobbyID) -> Result<Region, GameSyncError> {
        match self.data_store.get_region_lobby(&lobby_id) {
            Some(region) => {
                Ok(region)
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
    }

    fn find_lobby(&mut self, region: Region, lobby_id: LobbyID) -> Result<Lobby, GameSyncError> {
        match self.data_store.get_lobby(region, lobby_id) {
            Some(lobby) => {
                Ok(lobby)
            },
            None => return Err(GameSyncError::LobbyFindError)
        }
    }

    fn find_player(&mut self, player_id: PlayerID) -> Result<(Player, Option<LobbyID>), GameSyncError> {
        match self.data_store.get_player_info(player_id) {
            Some(player_info) => {
                Ok(player_info)
            },
            None => return Err(GameSyncError::PlayerFindError)
        }
    }

}

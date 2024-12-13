use crate::client::GameSyncClient;
use crate::error::GameSyncError;
use crate::networking::ClientEvent;
use crate::store::{LobbyID, PlayerID};
use message_io::network::SendStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub lobby_id: Uuid,
    pub params: LobbyParams,
    pub leader: PlayerID,
    pub status: LobbyStatus,
    pub player_list: Vec<PlayerID>,
    pub queue_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyParams {
    pub name: String,
    pub visibility: Visibility,
    pub region: Region,
    pub mode: GameMode,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub player_id: Uuid,
    pub rating: usize,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Private,
    Public,
}


#[derive(Eq, Hash, PartialEq, Copy, Debug, Clone, Serialize, Deserialize)]
pub enum Region {
    NA,
    EU,
    SA,
    MEA,
    AS,
    AU,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Casual,
    Competitive,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum LobbyStatus {
    Idle,
    Queueing,
    Ingame,
}

impl GameSyncClient {
    pub fn create_lobby(&mut self, params: LobbyParams) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::CreateLobby(params))?;
        Ok(result)
    }

    pub fn join_lobby(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::JoinLobby(lobby_id))?;
        Ok(result)
    }

    pub fn delete_lobby(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::DeleteLobby(lobby_id))?;
        Ok(result)
    }

    pub fn leave_lobby(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::LeaveLobby(lobby_id))?;
        Ok(result)
    }

    pub fn invite_lobby(&mut self, lobby_id: LobbyID, invitee: PlayerID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::InviteLobby(lobby_id, invitee))?;
        Ok(result)
    }

    pub fn get_public_lobbies(&mut self, region: Region) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::GetPublicLobbies(region))?;
        Ok(result)
    }

    pub fn edit_player(&mut self, player: Player) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::EditPlayer(player))?;
        Ok(result)
    }

    pub fn message_lobby(&mut self, lobby_id: LobbyID, message: String) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::MessageLobby(lobby_id, message))?;
        Ok(result)
    }

    pub fn queue_lobby(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::QueueLobby(lobby_id))?;
        Ok(result)
    }

    pub fn check_match(&mut self, lobby_id: LobbyID, threshold: Option<usize>) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::CheckMatch(lobby_id, threshold))?;
        Ok(result)
    }

    pub fn stop_queue(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::StopQueue(lobby_id))?;
        Ok(result)
    }

    pub fn leave_game_as_lobby(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::LeaveGameAsLobby(lobby_id))?;
        Ok(result)
    }

    pub fn get_lobby_info(&mut self, lobby_id: LobbyID) -> Result<SendStatus, GameSyncError>
    {
        let result = self.websocket.send_event(ClientEvent::GetLobbyInfo(lobby_id))?;
        Ok(result)
    }
}
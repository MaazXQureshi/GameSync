use crate::lobby::Lobby;
use crate::store::{LobbyID, PlayerID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
// server events from non-client initiated events - use event callbacks
// if client-initiated,
pub enum ServerEvent {
    Connected(),
    UserMessage(PlayerID, String),
    SelfPlayer(String),
    NewPlayer(String),
    LobbyCreated(Lobby), // Lobby
    LobbyJoined(PlayerID, LobbyID), // Lobby ID
    LobbyDeleted(LobbyID), // Lobby ID
    LobbyLeft(PlayerID, LobbyID), // Lobby ID
    LobbyInvited(LobbyID), // Lobby ID
    PublicLobbies(Vec<Lobby>),
    PlayerEdited(PlayerID), // Player IDConnected(),
    LobbyMessage(String), // Msg
    LobbyQueued(LobbyID),
    MatchFound(Lobby), // Opponent lobby
    MatchNotFound,
    QueueStopped(LobbyID),
    LeftGame(LobbyID),
    LobbyInfo(Lobby),
}
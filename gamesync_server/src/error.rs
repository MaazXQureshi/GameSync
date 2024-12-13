use std::fmt;
use serde_json::Error as ParseError;
use std::io::Error as SocketError;
use uuid::Error as UuidError;

#[derive(Debug)]
pub enum GameSyncError {
    ParseError(ParseError),
    SocketError(SocketError),
    UuidError(UuidError),
    SendError,
    UserNotFound,
    LobbyFindError,
    LobbyCreateError,
    LobbyJoinError,
    LobbyFullError,
    LobbyOwnerError,
    LobbyLeaveError,
    LobbyInviteError,
    LobbyCurInviteError,
    PlayerFindError,
    PlayerEditError,
    LobbyPlayerError,
    LobbyMessageError,
    LobbySizeError,
    LobbyQueueError,
    LobbyCheckError,
    LobbyDeleteError,
    LobbyStopError,
    LeaveGameError,
}

impl From<ParseError> for GameSyncError {
    fn from(err: ParseError) -> GameSyncError {
        GameSyncError::ParseError(err)
    }
}

impl From<SocketError> for GameSyncError {
    fn from(err: SocketError) -> GameSyncError {
        GameSyncError::SocketError(err)
    }
}

impl From<UuidError> for GameSyncError {
    fn from(err: UuidError) -> GameSyncError {
        GameSyncError::UuidError(err)
    }
}

impl fmt::Display for GameSyncError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameSyncError::ParseError(err) => write!(f, "Failed to parse event payload: {}.", err),
            GameSyncError::SocketError(err) => write!(f, "Socket error: {}.", err),
            GameSyncError::SendError => write!(f, "Failed to send socket event"),
            GameSyncError::UuidError(err) => write!(f, "Failed to parse uuid: {}", err),
            GameSyncError::LobbyFindError => write!(f, "Lobby not found."),
            GameSyncError::UserNotFound => write!(f, "Failed to find lobby"),
            GameSyncError::LobbyCreateError => write!(f, "Failed to create lobby. Player already part of a lobby"),
            GameSyncError::LobbyJoinError => write!(f, "Failed to join lobby. Player already part of a lobby"),
            GameSyncError::LobbyFullError => write!(f, "Failed to join lobby. Lobby full"),
            GameSyncError::LobbyLeaveError => write!(f, "Failed to leave lobby. Player not part of lobby"),
            GameSyncError::LobbyOwnerError => write!(f, "Invalid permissions. Player not lobby owner"),
            GameSyncError::LobbyInviteError => write!(f, "Failed to invite. Player not part of a lobby"),
            GameSyncError::LobbyCurInviteError => write!(f, "Failed to invite. Player not part of this lobby"),
            GameSyncError::LobbyPlayerError => write!(f, "Failed to send message. Player not in a lobby"),
            GameSyncError::LobbyMessageError => write!(f, "Failed to send message. Player not part of lobby"),
            GameSyncError::PlayerFindError => write!(f, "Player does not exist"),
            GameSyncError::PlayerEditError => write!(f, "Player cannot be edited. Must not be queueing or in-game"),
            GameSyncError::LobbySizeError => write!(f, "Failed to queue. Lobby is not full"),
            GameSyncError::LobbyQueueError => write!(f, "Lobby is already in queue or in-game"),
            GameSyncError::LobbyCheckError => write!(f, "Failed to check lobby. Lobby is not currently in queue"),
            GameSyncError::LobbyDeleteError => write!(f, "Failed to delete lobby. Lobby is not idle"),
            GameSyncError::LobbyStopError => write!(f, "Failed to stop queue. Lobby is not currently in queue"),
            GameSyncError::LeaveGameError => write!(f, "Failed to leave game. Lobby is not currently in-game"),
        }
    }
}

pub fn print_error(error: GameSyncError)  {
    println!("Error: {}", error)
}
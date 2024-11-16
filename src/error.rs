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
    LobbyFindError,
    LobbyOwnerError,
    LobbyInviteError,
    PlayerFindError,
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
            GameSyncError::LobbyFindError => write!(f, "Failed to find lobby"),
            GameSyncError::LobbyOwnerError => write!(f, "Invalid permissions: Player not lobby owner"),
            GameSyncError::LobbyInviteError => write!(f, "Error inviting: Player not part of lobby"),
            GameSyncError::PlayerFindError => write!(f, "Player does not exist"),
        }
    }
}

pub fn print_error(error: GameSyncError)  {
    println!("Error: {}", error)
}
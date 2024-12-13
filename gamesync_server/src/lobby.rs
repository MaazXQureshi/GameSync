use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

pub type PlayerID = Uuid;
pub type LobbyID = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub lobby_id: Uuid,
    pub params: LobbyParams,
    pub leader: PlayerID,
    pub status: LobbyStatus,
    pub player_list: Vec<PlayerID>,
    pub queue_threshold: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyParams {
    pub name: String,
	pub visibility: Visibility,
    pub region: Region,
    pub mode: GameMode
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub player_id: Uuid,
    pub rating: usize
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
	Private,
	Public
}

#[derive(Eq, Hash, PartialEq, Copy, Debug, Clone, EnumIter, Serialize, Deserialize)]
pub enum Region {
	NA,
	EU,
    SA,
    MEA,
    AS,
    AU
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Casual,
    Competitive
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum LobbyStatus {
    Idle,
    Queueing,
    Ingame
}
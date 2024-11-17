use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub(crate) lobby_id: Uuid,
    pub params: LobbyParams,
    pub leader: Player,
    pub status: LobbyStatus,
    pub player_list: Vec<Player>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyParams {
    name: String,
	pub visibility: Visibility,
    pub region: Region,
    mode: GameMode
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum LobbyStatus {
    Idle,
    Queueing,
    Ingame
}
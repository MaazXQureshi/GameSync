use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub(crate) lobby_id: Uuid,
    pub params: LobbyParams,
    pub leader: Player,
    pub status: LobbyStatus,
    pub player_list: Vec<Player>,
    pub queue_threshold: usize
}

impl Lobby {
    pub fn average_rating(&self) -> usize {
        if self.player_list.len() != 0 {
            self.player_list.iter().map(|player| player.rating).sum::<usize>() / self.player_list.len()
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyParams {
    name: String,
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
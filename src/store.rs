use crate::error::{print_error, GameSyncError};
use crate::networking::ServerEvent;
use uuid::Uuid;

pub struct Store {
    pub is_connected: bool,
    player_id: Uuid, // can change to a user struct later
    players: Vec<Uuid>,
    // more game states
}

impl Store {
    pub fn new() -> Store {
        Store { player_id: Uuid::nil(), players: Vec::new(), is_connected: false }
    }

    pub fn on_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Connected() => {
                self.is_connected = true;
            }
            ServerEvent::NewPlayer(player) => {
                match self.add_player(player) {
                    Ok(_) => {}
                    Err(error) => { print_error(error) }
                }
            }
            ServerEvent::SelfPlayer(player) => {
                match self.set_player_id(player) {
                    Ok(_) => {}
                    Err(error) => { print_error(error) }
                }
            }
            ServerEvent::UserMessage(message) => {
                println!("Received message: {}", message);
            }
        }
    }

    pub fn get_player_id(&self) -> Uuid {
        self.player_id
    }

    pub fn get_players(&self) -> Vec<Uuid> {
        self.players.clone()
    }

    pub fn set_player_id(&mut self, player_id: String) -> Result<(), GameSyncError> {
        self.player_id = Uuid::parse_str(&player_id)?;
        Ok(())
    }

    pub fn add_player(&mut self, player_id: String) -> Result<(), GameSyncError> {
        self.players.push(Uuid::parse_str(&player_id)?);
        Ok(())
    }
}
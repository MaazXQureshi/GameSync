use crate::client::MessageHandler;
use crate::error::{print_error, GameSyncError};
use crate::server_events::ServerEvent;
use message_io::node::NodeTask;
use uuid::Uuid;

pub type PlayerID = Uuid;
pub type LobbyID = Uuid;

pub struct Store {
    pub is_connected: bool,
    player_id: PlayerID, // can change to a user struct later
    // callbacks: Arc<Mutex<HashMap<String, Box<dyn Fn(ServerEvent) + Send>>>>,
    pub callbacks: Option<Box<dyn MessageHandler + Send + 'static>>,
    pub node_task: Option<NodeTask>
}

impl Store {
    pub fn new() -> Store {
        Store {
            is_connected: false,
            player_id: Uuid::nil(),
            callbacks: None,
            node_task: None,
        }
    }

    pub fn on_event(&mut self, event: ServerEvent) {
        match event.clone() {
            ServerEvent::Connected() => {
                self.is_connected = true;
            }
            // ServerEvent::NewPlayer(player) => {
            //     match self.add_player(player) {
            //         Ok(_) => {}
            //         Err(error) => { print_error(error) }
            //     }
            // }
            ServerEvent::SelfPlayer(player) => {
                match self.set_player_id(player) {
                    Ok(_) => {}
                    Err(error) => { print_error(error) }
                }
            }
            _ => {}
        }
        self.trigger_callback(event);
    }

    pub fn trigger_callback(&mut self, event: ServerEvent) {
        if let Some(callback) = &mut self.callbacks {
            callback.handle_message(event);
        }
    }

    pub fn get_player_id(&self) -> Uuid {
        self.player_id
    }

    pub fn set_player_id(&mut self, player_id: String) -> Result<(), GameSyncError> {
        self.player_id = Uuid::parse_str(&player_id)?;
        Ok(())
    }
}
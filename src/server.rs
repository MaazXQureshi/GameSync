use crate::error::GameSyncError;
use crate::networking::Websocket;

pub struct GameServer {
    websocket_server: Websocket,
}

impl GameServer {
    pub fn new(port: &str) -> Result<Self, GameSyncError> {
        let websocket_server = Websocket::new(port)?;
        Ok(GameServer {
            websocket_server
        })
    }

    pub fn process_messages(&mut self) {
        self.websocket_server.process_messages();
    }
}
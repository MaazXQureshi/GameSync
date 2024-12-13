use crate::error::GameSyncError;
use crate::networking::Websocket;
use crate::server_params::ServerParams;

pub struct GameServer {
    websocket_server: Websocket,
}

impl GameServer {
    pub fn new(port: &str, server_params: ServerParams) -> Result<Self, GameSyncError> {
        let websocket_server = Websocket::new(port, server_params)?;
        Ok(GameServer {
            websocket_server
        })
    }

    pub fn process_messages(&mut self) {
        self.websocket_server.process_messages();
    }
}
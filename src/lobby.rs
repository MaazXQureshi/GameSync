use crate::client::{Websocket, GameEvent, ChatMessageRequest};

pub struct GameSyncClient {
    // Underlying websocket connection and state management
    websocket: Websocket,
}

impl GameSyncClient {
    pub fn new(url: &str) -> Result<Self, ()> {
        let websocket = Websocket::connect(url).unwrap();
        Ok(GameSyncClient { websocket })
    }
    pub async fn send_chat_message(&mut self, lobby_id: String, message: String) -> Result<(), ()> {
        let request =  ChatMessageRequest { message, lobby_id };
        self.websocket.send_event(GameEvent::UserMessage(request));
        Ok(())
    }
}

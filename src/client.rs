use crate::error::GameSyncError;
use crate::networking::{ClientEvent, ServerEvent, Websocket};
use crate::store::Store;
use message_io::network::SendStatus;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub struct GameSyncClient {
    // Underlying websocket connection and state management
    websocket: Websocket,
    store: Arc<Mutex<Store>>,
}

impl GameSyncClient {
    pub fn connect(url: &str) -> Result<Self, GameSyncError> {
        let store = Arc::new(Mutex::new(Store::new()));
        let store_clone = Arc::clone(&store);

        let event_handler = move |event: ServerEvent| {
            if let Ok(mut store_locked) = store_clone.lock() {
                store_locked.on_event(event);
            }
        };
        let websocket = Websocket::new(url, event_handler)?;

        loop {
            let store = store.lock();
            match store {
                Ok(store) => {
                    if store.is_connected != false {
                        break;
                    }
                }
                Err(_) => {}
            }

            thread::sleep(Duration::from_millis(100));
        }

        let client = Self { websocket, store };
        Ok(client)
    }

    pub fn send_to_all_clients(&mut self, message: String) -> Result<SendStatus, GameSyncError> {
        Ok(self.websocket.send_event(ClientEvent::Broadcast(message))?)
    }

    pub fn send_to(&mut self, to: Uuid, message: String) -> Result<SendStatus, GameSyncError> {
        Ok(self.websocket.send_event(ClientEvent::SendTo(to.to_string(), message))?)
    }

    pub fn get_self(&self) -> Result<Uuid, GameSyncError> {
        let store = self.store.lock();
        match store {
            Ok(store) => { Ok(store.get_player_id()) }
            Err(_) => { Err(GameSyncError::LockError) }
        }
    }

    pub fn get_players(&self) -> Result<Vec<Uuid>, GameSyncError> {
        let store = self.store.lock();
        match store {
            Ok(store) => { Ok(store.get_players()) }
            Err(_) => { Err(GameSyncError::LockError) }
        }
    }
}

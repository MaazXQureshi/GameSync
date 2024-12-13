use crate::error::GameSyncError;
use crate::error::GameSyncError::LockError;
use crate::networking::{ClientEvent, Websocket};
use crate::server_events::ServerEvent;
use crate::store::Store;
use message_io::network::SendStatus;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;


#[derive(Clone)]
pub struct GameSyncClient {
    // Underlying websocket connection and state management
    pub(crate) websocket: Websocket,
    pub(crate) store: Arc<Mutex<Store>>,
}

pub trait MessageHandler {
    fn handle_message(&mut self, message: ServerEvent);
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
        let (websocket, node_task) = Websocket::new(url, event_handler)?;

        loop {
            let store = store.lock();
            match store {
                Ok(mut store) => {
                    if store.is_connected != false {
                        store.node_task = Some(node_task);
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

    pub fn register_callback<F>(&mut self, callback: F) -> Result<(), GameSyncError>
    where
        F: MessageHandler + Send + 'static,
    {
        // self.callbacks.lock().unwrap().insert(event.to_string(), Box::new(callback));
        let store = self.store.lock();
        match store {
            Ok(mut store) => {
                store.callbacks = Some(Box::new(callback));
            }
            Err(_) => { return Err(LockError) }
        }

        Ok(())
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
}

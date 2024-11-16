use message_io::network::Endpoint;
use strum::IntoEnumIterator;
use std::collections::HashMap;
use uuid::Uuid;
use crate::lobby::{Lobby, Player, Region, Visibility};
use dashmap::DashMap;
use std::sync::Arc;
use crate::error::GameSyncError;

pub struct DataStore {
    user_endpoint_map: HashMap<Uuid, Endpoint>,
    endpoint_user_map: HashMap<Endpoint, Uuid>,
    global_lobby_map: Arc<DashMap<Region, Arc<DashMap<Uuid, Lobby>>>>,
    region_lobby_map: Arc<DashMap<Uuid, Region>>,
    player_map: Arc<DashMap<Uuid, Player>>
}

impl DataStore {
    pub fn new() -> DataStore {
        let new_global_lobby_map: Arc<DashMap<Region, Arc<DashMap<Uuid, Lobby>>>> = Arc::new(DashMap::new());
        for region in Region::iter() {
            new_global_lobby_map.entry(region).or_insert_with(|| Arc::new(DashMap::new()));
        }
        let new_region_lobby_map: Arc<DashMap<Uuid, Region>> = Arc::new(DashMap::new());
        let new_player_map: Arc<DashMap<Uuid, Player>> = Arc::new(DashMap::new());

        Self {
            user_endpoint_map: HashMap::new(),
            endpoint_user_map: HashMap::new(),
            global_lobby_map: Arc::clone(&new_global_lobby_map),
            region_lobby_map: Arc::clone(&new_region_lobby_map),
            player_map: Arc::clone(&new_player_map),
        }
    }

    pub fn add_user_endpoint(&mut self, endpoint: Endpoint) -> Uuid {
        let user_id = Uuid::new_v4();
        self.user_endpoint_map.insert(user_id, endpoint);
        self.endpoint_user_map.insert(endpoint, user_id);
        user_id
    }

    pub fn get_all_user_endpoints(&self) -> Vec<Endpoint> {
        self.user_endpoint_map.values().cloned().collect()
    }

    pub fn get_user_endpoint(&self, user_id: &Uuid) -> Option<Endpoint> {
        self.user_endpoint_map.get(user_id).cloned()
    }

    pub fn get_user(&self, endpoint: Endpoint) -> Option<&Uuid> {
        self.endpoint_user_map.get(&endpoint)
    }

    pub fn remove_user_endpoint(&mut self, endpoint: Endpoint) {
        if self.endpoint_user_map.contains_key(&endpoint) { // TODO: Fix logic, since if user is removed from first map, cannot find in second map
            self.endpoint_user_map.remove(&endpoint);
        }

        let user_id = self.endpoint_user_map.get(&endpoint);

        if let Some(user_id) = user_id {
            self.user_endpoint_map.remove(user_id);
        }
    }

    pub fn create_lobby(&mut self, outer_key: Region, inner_key: Uuid, inner_value: Lobby) {
        let inner_map = self
            .global_lobby_map
            .entry(outer_key)
            .or_insert_with(|| Arc::new(DashMap::new()));
   
        inner_map.clone().insert(inner_key, inner_value);
    }

    pub fn print_global_lobby_map(&self) {
        for outer_entry in self.global_lobby_map.iter() {
            println!("Outer Key: {:?}", outer_entry.key());
            for inner_entry in outer_entry.value().iter() {
                println!("    Inner Key: {}, Inner Value: {:?}", inner_entry.key(), inner_entry.value());
            }
        }
    }

    pub fn edit_lobby(&mut self, outer_key: Region, inner_key: Uuid, new_value: Lobby) -> Result<(), GameSyncError> {
        if let Some(inner_map) = self.global_lobby_map.get(&outer_key) {
            inner_map.insert(inner_key, new_value);
            Ok(())
        } else {
            return Err(GameSyncError::LobbyFindError)
        }
    }

    pub fn delete_lobby(&mut self, outer_key: Region, inner_key: Uuid) -> Result<(), GameSyncError> {
        if let Some(inner_map) = self.global_lobby_map.get(&outer_key) {
            inner_map.remove(&inner_key);  // Remove the entry from the inner map
            Ok(())
        } else {
            return Err(GameSyncError::LobbyFindError)
        }
    }

    pub fn get_lobby(&self, outer_key: Region, inner_key: Uuid) -> Option<Lobby> {
        if let Some(inner_map) = self.global_lobby_map.get(&outer_key) {
            match inner_map.get(&inner_key) {
                Some(lobby) => Some(lobby.value().clone()),
                _ => {
                    println!("Inner key '{:?}' not found.", inner_key);
                    None
                }
            }
        } else {
            println!("Outer key '{:?}' not found.", outer_key);
            None
        }
    }

    pub fn get_region_lobbies(&self, region: Region) -> Vec<Lobby> {
        let region_lobbies = self.global_lobby_map.get(&region).unwrap(); // Guaranteed to return if initialized correctly
        region_lobbies.iter()
        .filter_map(|entry| {
            if entry.value().params.visibility == Visibility::Public {
                Some(entry.value().clone())
            } else {
                None
            }
        })
        .collect()
    }   

    pub fn create_region_lobby(&mut self, lobby_id: Uuid, region: Region) {
        self.region_lobby_map.insert(lobby_id, region);
    }

    pub fn get_region_lobby(&mut self, lobby_id: &Uuid) -> Option<Region> {
        match self.region_lobby_map.get(&lobby_id) {
            Some(entry) => Some(entry.value().clone()),
            _ => None    
        }
    }

    pub fn delete_region_lobby(&mut self, lobby_id: Uuid) -> Result<(), GameSyncError> {
        match self.region_lobby_map.remove(&lobby_id) {
            Some(_) => Ok(()),
            None => return Err(GameSyncError::LobbyFindError)
        }
    }

    pub fn add_player(&mut self, player_id: Uuid, player: Player) {
        self.player_map.insert(player_id, player);
    }

    pub fn edit_player(&mut self, player: Player) {
        self.player_map.entry(player.player_id)
        .and_modify(|player_struct| {
            player_struct.rating = player.rating;
        });
    }

    pub fn get_player(&mut self, player_id: Uuid) -> Option<Player> {
        match self.player_map.get(&player_id) {
            Some(entry) => Some(entry.value().clone()),
            _ => None
        }
    }

    pub fn delete_player(&mut self, endpoint: Endpoint) {
        let player_id = self.endpoint_user_map.get(&endpoint);
        if let Some(player_id) = player_id {
            self.player_map.remove(&player_id);
        }
    }

}
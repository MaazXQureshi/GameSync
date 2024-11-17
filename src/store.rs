use message_io::network::Endpoint;
use strum::IntoEnumIterator;
use uuid::Uuid;
use crate::lobby::{Lobby, Player, Region, Visibility};
use crate::server_params::ServerParams;
use dashmap::DashMap;
use std::sync::Arc;
use crate::error::GameSyncError;

pub struct DataStore {
    user_endpoint_map: Arc<DashMap<Uuid, Endpoint>>,
    endpoint_user_map: Arc<DashMap<Endpoint, Uuid>>,
    global_lobby_map: Arc<DashMap<Region, Arc<DashMap<Uuid, Lobby>>>>,
    region_lobby_map: Arc<DashMap<Uuid, Region>>,
    player_map: Arc<DashMap<Uuid, (Player, Option<Lobby>)>>,
    server_params: ServerParams,
}

impl DataStore {
    pub fn new(server_params: ServerParams) -> DataStore {
        let new_user_endpoint_map: Arc<DashMap<Uuid, Endpoint>> = Arc::new(DashMap::new());
        let new_endpoint_user_map: Arc<DashMap<Endpoint, Uuid>> = Arc::new(DashMap::new());
        let new_global_lobby_map: Arc<DashMap<Region, Arc<DashMap<Uuid, Lobby>>>> = Arc::new(DashMap::new());
        for region in Region::iter() {
            new_global_lobby_map.entry(region).or_insert_with(|| Arc::new(DashMap::new()));
        }
        let new_region_lobby_map: Arc<DashMap<Uuid, Region>> = Arc::new(DashMap::new());
        let new_player_map: Arc<DashMap<Uuid, (Player, Option<Lobby>)>> = Arc::new(DashMap::new());

        Self {
            user_endpoint_map: Arc::clone(&new_user_endpoint_map),
            endpoint_user_map: Arc::clone(&new_endpoint_user_map),
            global_lobby_map: Arc::clone(&new_global_lobby_map),
            region_lobby_map: Arc::clone(&new_region_lobby_map),
            player_map: Arc::clone(&new_player_map),
            server_params: server_params.clone()
        }
    }

    /* WEBSOCKET CONNECTIONS HASHMAP FUNCTIONS */
    pub fn add_user_endpoint(&mut self, endpoint: Endpoint) -> Uuid {
        let user_id = Uuid::new_v4();
        self.user_endpoint_map.insert(user_id, endpoint);
        self.endpoint_user_map.insert(endpoint, user_id);
        user_id
    }

    pub fn get_all_user_endpoints(&self) -> Vec<Endpoint> {
        self.user_endpoint_map.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get_user_endpoint(&self, user_id: &Uuid) -> Option<Endpoint> {
        match self.user_endpoint_map.get(user_id) {
            Some(entry) => Some(entry.value().clone()),
            _ => None
        }
    }

    pub fn get_user(&self, endpoint: Endpoint) -> Option<Uuid> {
        match self.endpoint_user_map.get(&endpoint) {
            Some(entry) => Some(entry.value().clone()),
            _ => None
        }
    }

    pub fn remove_user_endpoint(&mut self, endpoint: Endpoint) {
        let user_id = self.get_user(endpoint);

        if let Some(user_id) = user_id {
            self.user_endpoint_map.remove(&user_id);
        }

        self.endpoint_user_map.remove(&endpoint);
    }

    /* GLOBAL LOBBIES HASHMAP FUNCTIONS */
    pub fn create_lobby(&self, outer_key: Region, inner_key: Uuid, inner_value: Lobby) {
        let inner_map = self
            .global_lobby_map
            .entry(outer_key)
            .or_insert_with(|| Arc::new(DashMap::new()));
   
        inner_map.clone().insert(inner_key, inner_value);
    }

    pub fn print_global_lobby_map(&self) {
        for outer_entry in self.global_lobby_map.iter() {
            println!("Region: {:?}", outer_entry.key());
            for inner_entry in outer_entry.value().iter() {
                println!("    LobbyID: {}, Lobby: {:?}", inner_entry.key(), inner_entry.value());
            }
        }
    }

    pub fn edit_lobby(&self, outer_key: Region, inner_key: Uuid, new_value: Lobby) -> Result<(), GameSyncError> {
        if let Some(inner_map) = self.global_lobby_map.get(&outer_key) {
            inner_map.insert(inner_key, new_value);
            Ok(())
        } else {
            return Err(GameSyncError::LobbyFindError)
        }
    }

    pub fn delete_lobby(&self, outer_key: Region, inner_key: Uuid) -> Result<(), GameSyncError> {
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

    /* <LOBBY_ID, REGION> HASHMAP FUNCTIONS */
    pub fn create_region_lobby(&self, lobby_id: Uuid, region: Region) {
        self.region_lobby_map.insert(lobby_id, region);
    }

    pub fn get_region_lobby(&self, lobby_id: &Uuid) -> Option<Region> {
        match self.region_lobby_map.get(&lobby_id) {
            Some(entry) => Some(entry.value().clone()),
            _ => None    
        }
    }

    pub fn delete_region_lobby(&self, lobby_id: Uuid) -> Result<(), GameSyncError> {
        match self.region_lobby_map.remove(&lobby_id) {
            Some(_) => Ok(()),
            None => return Err(GameSyncError::LobbyFindError)
        }
    }

    pub fn print_region_lobby_map(&self) {
        for entry in self.region_lobby_map.iter() {
            println!("Lobby_ID {:?} Region {:?}", entry.key(), entry.value());
        }
    }

    /* <PLAYER_ID, (PLAYER, LOBBY)> HASHMAP FUNCTIONS */
    pub fn add_player(&self, player_id: Uuid, player: Player) {
        self.player_map.insert(player_id, (player, None));
    }

    pub fn edit_player(&self, player_id: Uuid, player: Option<Player>, lobby: Option<Lobby>) {
        self.player_map.entry(player_id)
        .and_modify(|tuple| {
            match player {
                Some(player) => {
                    tuple.0 = player;
                },
                None => ()
            }
            match lobby {
                Some(lobby) => {
                    tuple.1 = Some(lobby)
                },
                None => ()
            }
        });
    }

    pub fn remove_player_lobby(&self, player_id: Uuid) {
        self.player_map.entry(player_id)
        .and_modify(|tuple| {
            tuple.1 = None
        });
    }

    pub fn get_player_info(&self, player_id: Uuid) -> Option<(Player, Option<Lobby>)> {
        match self.player_map.get(&player_id) {
            Some(entry) => Some(entry.value().clone()),
            _ => None
        }
    }

    pub fn delete_player(&self, player_id: Uuid) {
        self.player_map.remove(&player_id);
    }

    pub fn print_player_map(&self) {
        for entry in self.player_map.iter() {
            println!("Player {:?} Lobby {:?}", entry.value().clone().0, entry.value().clone().1);

        }
    }

    /* MISCELLANEOUS FUNCTIONS */

    pub fn lobby_size(&self) -> usize {
        self.server_params.player_count
    }

}
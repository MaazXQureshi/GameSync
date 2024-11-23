use message_io::network::Endpoint;
use strum::IntoEnumIterator;
use uuid::Uuid;
use crate::lobby::{Lobby, Player, Region, Visibility, PlayerID, LobbyID};
use crate::server_params::ServerParams;
use dashmap::DashMap;
use std::cmp::Ordering;
use std::sync::Arc;
use crate::error::GameSyncError;
use std::collections::VecDeque;

pub struct DataStore {
    user_endpoint_map: Arc<DashMap<Uuid, Endpoint>>,
    endpoint_user_map: Arc<DashMap<Endpoint, Uuid>>,
    global_lobby_map: Arc<DashMap<Region, Arc<DashMap<Uuid, Lobby>>>>,
    region_lobby_map: Arc<DashMap<LobbyID, Region>>,
    player_map: Arc<DashMap<PlayerID, (Player, Option<LobbyID>)>>,
    competitive_queue_map: Arc<DashMap<Region, Vec<Lobby>>>,
    casual_queue_map: Arc<DashMap<Region, VecDeque<Lobby>>>,
    server_params: ServerParams,
}

impl DataStore {
    pub fn new(server_params: ServerParams) -> DataStore {
        let new_user_endpoint_map: Arc<DashMap<Uuid, Endpoint>> = Arc::new(DashMap::new());
        let new_endpoint_user_map: Arc<DashMap<Endpoint, Uuid>> = Arc::new(DashMap::new());
        let new_global_lobby_map: Arc<DashMap<Region, Arc<DashMap<Uuid, Lobby>>>> = Arc::new(DashMap::new());
        let new_competitive_queue_map: Arc<DashMap<Region, Vec<Lobby>>> = Arc::new(DashMap::new());
        let new_casual_queue_map: Arc<DashMap<Region, VecDeque<Lobby>>> = Arc::new(DashMap::new());

        for region in Region::iter() {
            new_global_lobby_map.entry(region).or_insert_with(|| Arc::new(DashMap::new()));
            new_competitive_queue_map.entry(region).or_insert_with(|| Vec::new());
            new_casual_queue_map.entry(region).or_insert_with(|| VecDeque::new());
        }
        let new_region_lobby_map: Arc<DashMap<Uuid, Region>> = Arc::new(DashMap::new());
        let new_player_map: Arc<DashMap<PlayerID, (Player, Option<LobbyID>)>> = Arc::new(DashMap::new());


        Self {
            user_endpoint_map: Arc::clone(&new_user_endpoint_map),
            endpoint_user_map: Arc::clone(&new_endpoint_user_map),
            global_lobby_map: Arc::clone(&new_global_lobby_map),
            region_lobby_map: Arc::clone(&new_region_lobby_map),
            player_map: Arc::clone(&new_player_map),
            competitive_queue_map: Arc::clone(&new_competitive_queue_map),
            casual_queue_map: Arc::clone(&new_casual_queue_map),
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
                    // println!("Inner key '{:?}' not found.", inner_key);
                    None
                }
            }
        } else {
            // println!("Outer key '{:?}' not found.", outer_key);
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

    pub fn edit_player(&self, player_id: Uuid, player: Option<Player>, lobby_id: Option<LobbyID>) {
        self.player_map.entry(player_id)
        .and_modify(|tuple| {
            match player {
                Some(player) => {
                    tuple.0 = player;
                },
                None => ()
            }
            match lobby_id {
                Some(lobby_id) => {
                    tuple.1 = Some(lobby_id)
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

    pub fn get_player_info(&self, player_id: Uuid) -> Option<(Player, Option<LobbyID>)> {
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

    /* CASUAL QUEUE FUNCTIONS */
    
    pub fn add_casual_lobby(&mut self, region: Region, lobby: Lobby) {
        if let Some(mut region_lobbies) = self.casual_queue_map.get_mut(&region) {
            region_lobbies.push_back(lobby);
        }
    }

    pub fn get_casual_lobbies(&self, region: Region) -> Option<VecDeque<Lobby>> {
        self.casual_queue_map.get(&region).map(|lobbies| lobbies.clone())
    }

    pub fn check_casual_lobby(&self, region: Region, lobby_id: Uuid) -> Option<(Lobby, Lobby)> {
        if let Some(mut lobbies) = self.casual_queue_map.get_mut(&region) {
            if lobbies.len() < 2 { // This should not happen
                return None;
            }
            let target_index = lobbies.iter().position(|l| l.lobby_id == lobby_id)?;

            // Temporarily remove the target lobby from the queue
            let lobby1 = lobbies.remove(target_index).unwrap();

            // Try to match with the next available lobby (from the front or back of the queue)
            let lobby2 = if !lobbies.is_empty() {
                lobbies.pop_front() // Remove from the front
            } else {
                None
            };

            if let Some(lobby2) = lobby2 {
                return Some((lobby1, lobby2));
            } else {
                // Reinsert the lobby if no match was found
                lobbies.push_back(lobby1);
            }
        }
        None
    }

    pub fn remove_casual_lobby(&self, region: Region, lobby_id: Uuid) { // This is done this way to avoid shifting the entire VecDeque
        if let Some(mut region_lobbies) = self.casual_queue_map.get_mut(&region) {
            if let Some(index) = region_lobbies.iter().position(|l| l.lobby_id == lobby_id) {
                // Use split_off to remove the element
                let mut split_off = region_lobbies.split_off(index);
                split_off.pop_front(); // Remove the first element of the split-off part
                region_lobbies.append(&mut split_off); // Rejoin the remaining elements
            }
        }
    }

    pub fn print_casual_lobbies(&self) {
        for entry in self.casual_queue_map.iter() {
            println!("Region {:?}", entry.key());
            for lobby in entry.value() {
                println!("  Lobby: {:?}", lobby);
            }
        }
    }

    /* COMPETITIVE QUEUE FUNCTIONS */
    // Lobby vector is ordered by average rating
    pub fn add_competitive_lobby(&mut self, region: Region, lobby: Lobby) {
        if let Some(mut lobbies) = self.competitive_queue_map.get_mut(&region) {
            let avg_rating = self.get_lobby_average_rating(region, lobby.lobby_id);
            let position = lobbies
                .binary_search_by(|l| self.get_lobby_average_rating(region, l.lobby_id).partial_cmp(&avg_rating).unwrap_or(Ordering::Equal))
                .unwrap_or_else(|e| e);
            lobbies.insert(position, lobby);
        }
    }

    pub fn get_competitive_lobbies(&self, region: Region) -> Option<Vec<Lobby>> {
        self.competitive_queue_map.get(&region).map(|lobbies| lobbies.clone())
    }

    pub fn check_competitive_lobby(&self, region: Region, lobby_id: Uuid, threshold: usize) -> Option<(Lobby, Lobby)> {
        if let Some(mut lobbies) = self.competitive_queue_map.get_mut(&region) {
            if lobbies.is_empty() { // This should ideally never happen
                return None;
            }

            let target_index = lobbies.iter().position(|lobby| lobby.lobby_id == lobby_id)?;

            let lobby1 = lobbies[target_index].clone();
            let avg_rating1 = self.get_lobby_average_rating(region, lobby1.lobby_id);
            let range_min = avg_rating1 - threshold;
            let range_max = avg_rating1 + threshold;

            let lower_bound = lobbies
                .binary_search_by(|l| {
                    if self.get_lobby_average_rating(region, l.lobby_id) < range_min {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                })
                .unwrap_or_else(|e| e);

            let upper_bound = lobbies
                .binary_search_by(|l| {
                    if self.get_lobby_average_rating(region, l.lobby_id) > range_max {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                })
                .unwrap_or_else(|e| e);

            for i in lower_bound..upper_bound {
                if i == target_index {
                    continue; // Skip the target lobby itself
                }

                let lobby2 = &lobbies[i];
                let avg_rating2 = self.get_lobby_average_rating(region, lobby2.lobby_id);
                let range_min2 = avg_rating2 - lobby2.queue_threshold;
                let range_max2 = avg_rating2 + lobby2.queue_threshold;

                if range_min <= range_max2 && range_min2 <= range_max {
                    let matched_lobby2 = lobbies.remove(i);
                    let matched_lobby1 = lobbies.remove(target_index);
                    return Some((matched_lobby1.clone(), matched_lobby2.clone()));
                }
            }
        }
        None  // This could either mean match not found or match was already found before (hence removed from queue). Client's responsibility for stop searching once MatchFound is received OR do a check for if lobby is already InGame state
    }

    pub fn get_lobby_average_rating(&self, region: Region, lobby_id: Uuid) -> usize {
        if let Some(lobby) = self.get_lobby(region, lobby_id) {
            if lobby.player_list.len() != 0 {
                return lobby.player_list.iter().map(|player_id| {
                    match self.get_player_info(player_id.clone()) {
                        Some(player) => {
                            player.0.rating
                        },
                        None => {
                            0
                        }
                    }
                }).sum::<usize>() / lobby.player_list.len();
            } 
            else {
                return 0
            }
        }
        0 // Lobby not found. Should not happen -> error checking done prior to this
    }

    pub fn remove_competitive_lobby(&mut self, region: Region, lobby_id: Uuid) {
        if let Some(mut region_lobbies) = self.competitive_queue_map.get_mut(&region) {
            if let Some(index) = region_lobbies.iter().position(|l| l.lobby_id == lobby_id) {
                region_lobbies.remove(index);
            }
        }
    }

    pub fn print_competitive_lobbies(&self) {
        for entry in self.competitive_queue_map.iter() {
            println!("Region {:?}", entry.key());
            for lobby in entry.value() {
                println!("  Lobby: {:?}", lobby);
            }
        }
    }


    /* MISCELLANEOUS FUNCTIONS */

    pub fn lobby_size(&self) -> usize {
        self.server_params.player_count
    }

}
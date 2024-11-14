use message_io::network::Endpoint;
use std::collections::HashMap;
use uuid::Uuid;

pub struct DataStore {
    user_endpoint_map: HashMap<Uuid, Endpoint>,
    endpoint_user_map: HashMap<Endpoint, Uuid>,
}

impl DataStore {
    pub fn new() -> DataStore {
        Self {
            user_endpoint_map: HashMap::new(),
            endpoint_user_map: HashMap::new(),
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
        if self.endpoint_user_map.contains_key(&endpoint) {
            self.endpoint_user_map.remove(&endpoint);
        }

        let user_id = self.endpoint_user_map.get(&endpoint);

        if let Some(user_id) = user_id {
            self.user_endpoint_map.remove(user_id);
        }
    }
}
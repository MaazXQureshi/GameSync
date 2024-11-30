use std::io::{self, BufRead};
use std::io::Write;
use std::thread;
use gamesync_client::client::{GameSyncClient, MessageHandler};
use gamesync_client::lobby::{GameMode, Lobby, LobbyParams, Player, Region, Visibility};
use gamesync_client::server_events::ServerEvent;
use uuid::Uuid;
use tokio::task;
use std::str::FromStr;

#[derive(Clone)]
pub struct MyMessageHandler {
    client: GameSyncClient,
    players: Vec<String>,
}

impl MessageHandler for MyMessageHandler {
    fn handle_message(&mut self, message: ServerEvent) {
        match message {
            ServerEvent::LobbyCreated ( lobby ) => {
                println!("Lobby Created: {:?}", lobby.clone());
            },
            ServerEvent::NewPlayer(id) => {
                println!("New player id: {}", id);
                self.add_player(id.clone());
                let msg = String::from("Welcome ") + &*id;
                self.client.send_to(Uuid::parse_str(&*id).unwrap(), msg).unwrap();
            },
            ServerEvent::UserMessage(id, msg) => {
                let message = format!("[{}]: {}",id,  msg);
                println!("[{}]: {}",id,  msg);
            },
            ServerEvent::LobbyDeleted(id) => {
                println!("Lobby {id} successfully deleted");
            },
            ServerEvent::LobbyJoined(player_id, lobby_id) => {
                println!("New player {player_id} has joined lobby {lobby_id}");
            },
            ServerEvent::LobbyQueued(lobby_id) => {
                println!("Lobby {lobby_id} has been queued");
            },
            ServerEvent::PublicLobbies(lobbies) => {
                println!("All public lobbies: ");
                println!("{:?}", lobbies);
            },
            ServerEvent::MatchFound(lobby) => {
                println!("Match was found for lobby: ");
                println!("{:?}", lobby);
            },
            ServerEvent::MatchNotFound => {
                println!("No match was found");
            },
            ServerEvent::LobbyInfo(lobby) => {
                println!("Lobby info: {:?}", lobby);
            },
            // Handle other message types
            _ => {}
        }
    }
}

impl MyMessageHandler {
    pub fn add_player(&mut self, id: String) {
        self.players.push(id);
    }
}

fn parse_region(input: &str) -> Result<Region, String> {
    match input.to_uppercase().as_str() {
        "NA" => Ok(Region::NA),
        "EU" => Ok(Region::EU),
        "SA" => Ok(Region::SA),
        "MEA" => Ok(Region::MEA),
        "AS" => Ok(Region::AS),
        "AU" => Ok(Region::AU),
        _ => Err(format!("'{}' is not a valid Region", input)),
    }
}

#[tokio::main]
async fn main() {
    let server_url= "ws://127.0.0.1:8080/ws/";

    // Connect to the WebSocket server
    let mut client = GameSyncClient::connect(server_url).unwrap();

    let handler = MyMessageHandler {
        client: client.clone(),
        players: Vec::new()
    };

    client.register_callback(handler).expect("Error registering game callback");

    let id = client.get_self().unwrap();

    let msg = String::from("Hi everyone! I am ") + id.to_string().as_str();
    client.send_to_all_clients(msg).unwrap();

    let lobby_params = LobbyParams{
        name: String::from("lobby1"),
        visibility: Visibility::Public,
        region: Region::AU,
        mode: GameMode::Casual
    };

    let result = client.create_lobby(lobby_params);

    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            // Read user input
            let input = get_user_input();
        
            let parts: Vec<&str> = input.split_whitespace().collect();
        
            let result = match parts.as_slice() {
                ["broadcast", msg] => Ok(client.send_to_all_clients(msg.to_string()).expect("Failed to broadcast")),
                ["sendto", recipient, msg] => Ok(client.send_to(Uuid::parse_str(recipient).unwrap(), msg.to_string()).expect("Failed to send message")),
                ["create_lobby", lobby_params] => Ok(client.create_lobby(serde_json::from_str::<LobbyParams>(lobby_params).unwrap()).expect("Failed to send message")),
                ["join_lobby", lobby_id] => Ok(client.join_lobby(Uuid::parse_str(lobby_id).unwrap()).expect("Failed to send message")),
                ["delete_lobby", lobby_id] => Ok(client.delete_lobby(Uuid::parse_str(lobby_id).unwrap()).expect("Failed to send message")),
                ["queue_lobby", lobby_id] => Ok(client.queue_lobby(Uuid::parse_str(lobby_id).unwrap()).expect("Failed to send message")),
                ["check_match", lobby_id, threshold] => Ok(client.check_match(Uuid::parse_str(lobby_id).unwrap(), Some(threshold.parse::<usize>().unwrap())).expect("Failed to send message")),
                ["edit_player", rating] => Ok(client.edit_player(Player { player_id: id, rating: rating.parse::<usize>().unwrap() }).expect("Failed to send message")),
                ["get_public_lobbies", region] => Ok(client.get_public_lobbies(parse_region(&region).unwrap()).expect("Failed to send message")),
                ["get_lobby_info", lobby_id] => Ok(client.get_lobby_info(Uuid::parse_str(lobby_id).unwrap()).expect("Failed to send message")),

                // Add similar entries for the other events
                _ => Err(format!("Unknown command: {}", input)),
            };
            }
    });


    loop {
        std::thread::park();
    }
}

fn print_lobby_state(lobby_state: Option<Lobby>) {
    // Implement this function to display the updated lobby state.
    println!("{:#?}", lobby_state);
}

fn get_user_input() -> String {
    let mut input = String::new();
    print!("> ");
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

use std::io::{self, BufRead};
use std::io::Write;
use std::sync::atomic::AtomicUsize;
use std::thread;
use gamesync_client::client::{GameSyncClient, MessageHandler};
use gamesync_client::lobby::{GameMode, Lobby, LobbyParams, Player, Region, Visibility};
use gamesync_client::server_events::ServerEvent;
use uuid::Uuid;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use tokio::task;
use std::str::FromStr;

static QUEUE_ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct MyMessageHandler {
    client: GameSyncClient,
    players: Vec<String>,
}

impl MessageHandler for MyMessageHandler {
    fn handle_message(&mut self, message: ServerEvent) {
        match message {
            ServerEvent::NewPlayer(id) => {
                // println!("New player id: {}", id);
                self.add_player(id.clone());
                let msg = String::from("Welcome ") + &*id;
                self.client.send_to(Uuid::parse_str(&*id).unwrap(), msg).unwrap();
            },
            ServerEvent::UserMessage(id, msg) => {
                // let message = format!("[{}]: {}",id,  msg);
                println!("[{}]: {}",id,  msg);
            },
            ServerEvent::LobbyCreated ( lobby ) => {
                println!("Lobby Created Successfully");
                print_lobby(lobby);
            },
            ServerEvent::LobbyJoined(player_id, lobby_id) => {
                println!("New player {player_id} has joined lobby {lobby_id}");
            },
            ServerEvent::LobbyDeleted(id) => {
                println!("Lobby {id} successfully deleted");
            },
            ServerEvent::LobbyLeft(player_id, lobby_id) => {
                println!("Player {player_id} has left lobby {lobby_id}");
            },
            ServerEvent::LobbyInvited(lobby_id) => {
                println!("You have been invited to lobby {lobby_id}");
            },
            ServerEvent::PublicLobbies(lobbies) => {
                println!("All public lobbies: ");
                for lobby in lobbies {
                    print_lobby(lobby);
                }
            },
            ServerEvent::PlayerEdited(player_id) => {
                println!("Player {player_id} successfully edited");
            },
            ServerEvent::LobbyMessage(player_id, msg) => {
                println!("<LOBBY> [{}]: {}", player_id, msg);
            },
            ServerEvent::LobbyQueued(lobby_id) => {
                println!("Lobby {lobby_id} has been queued");
                println!("Starting match search...");
                QUEUE_ACTIVE.store(true, Ordering::SeqCst);
            },
            ServerEvent::MatchFound(lobby) => {
                println!("Match found against lobby: {}", lobby.lobby_id);
                QUEUE_ACTIVE.store(false, Ordering::SeqCst);
            },
            ServerEvent::MatchNotFound => {
                // println!("No match was found");
            },
            ServerEvent::QueueStopped(lobby_id) => {
                println!("Queue was stopped for lobby {}", lobby_id);
                QUEUE_ACTIVE.store(false, Ordering::SeqCst);
            },
            ServerEvent::LeftGame(lobby_id) => {
                println!("Lobby {} left the game", lobby_id);
            },
            ServerEvent::LobbyInfo(lobby) => {
                print_lobby(lobby);
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
    println!("Client ID: {id}");
    // let queue_active = Arc::new(AtomicBool::new(false));
    // let queue_active_clone = queue_active.clone();
    let queue_threshold = Arc::new(AtomicUsize::new(0));

    // let msg = String::from("Hi everyone! I am ") + id.to_string().as_str();
    // client.send_to_all_clients(msg).unwrap();

    // let lobby_params = LobbyParams{
    //     name: String::from("lobby1"),
    //     visibility: Visibility::Public,
    //     region: Region::AU,
    //     mode: GameMode::Casual
    // };

    // let result = client.create_lobby(lobby_params);

    let start_ping_thread = |mut client: GameSyncClient, lobby_id: Uuid, threshold: Arc<AtomicUsize>| {
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(2)); // To give time for the server to return a successful MatchQueued
            while QUEUE_ACTIVE.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(10));
                let current_threshold = threshold.load(Ordering::SeqCst);
                client.check_match(lobby_id, Some(current_threshold)).expect("Failed to send message");
                // println!("Ping sent to server");
            }
            // println!("Ping thread exiting...");
        })
    };

    let mut ping_thread = start_ping_thread(client.clone(), id, queue_threshold.clone());

    thread::spawn(move || {
        loop {
            // Read user input
            let input = get_user_input();
        
            let parts: Vec<&str> = input.split_whitespace().collect();
        
            match parts.as_slice() {
                ["broadcast", msg @ ..] => {
                    let message = msg.join(" ");
                    client.send_to_all_clients(message).expect("Failed to broadcast");
                },
                ["sendto", recipient, msg @ ..] => {
                    let message = msg.join(" ");
                    match Uuid::parse_str(recipient) {
                        Ok(player_id) => {
                            client.send_to(player_id, message).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", recipient),
                    }
                },
                ["join_lobby", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.join_lobby(lobby_id).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["delete_lobby", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.delete_lobby(lobby_id).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["leave_lobby", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.leave_lobby(lobby_id).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["queue_lobby", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.queue_lobby(lobby_id).expect("Failed to send message");
                            if !QUEUE_ACTIVE.load(Ordering::SeqCst) {
                                ping_thread = start_ping_thread(client.clone(), lobby_id, queue_threshold.clone());
                            }
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["stop_queue", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.stop_queue(lobby_id).expect("Failed to send message");
                            QUEUE_ACTIVE.store(false, Ordering::SeqCst);
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["leave_game", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.leave_game_as_lobby(lobby_id).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["get_lobby_info", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.get_lobby_info(lobby_id).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["message_lobby", lobby_id, msg @ ..] => {
                    let message = msg.join(" ");
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.message_lobby(lobby_id, message).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["invite_lobby", lobby_id, player_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            match Uuid::parse_str(player_id) {
                                Ok(player_id) => {
                                    client.invite_lobby(lobby_id, player_id).expect("Failed to send message");
                                },
                                Err(_) => println!("Invalid UUID: {}", player_id),
                            }
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["check_match", lobby_id] => {
                    match Uuid::parse_str(lobby_id) {
                        Ok(lobby_id) => {
                            client.check_match(lobby_id, None).expect("Failed to send message");
                            queue_threshold.store(0, Ordering::SeqCst);
                        },
                        Err(_) => println!("Invalid UUID: {}", lobby_id),
                    }
                },
                ["check_match", lobby_id, threshold] => {
                    match (Uuid::parse_str(lobby_id), threshold.parse::<usize>()) {
                        (Ok(lobby_uuid), Ok(threshold)) => {
                            client.check_match(lobby_uuid, Some(threshold)).expect("Failed to check match with number");
                            queue_threshold.store(threshold, Ordering::SeqCst);
                        },
                        (Err(_), _) => println!("Invalid UUID: {}", lobby_id),
                        (_, Err(_)) => println!("Invalid threshold {threshold}")
                    }
                },
                ["update_threshold", threshold] => { // Local command, only to update threshold for queueing loop thread
                    match threshold.parse::<usize>() {
                        Ok(threshold) => {
                            queue_threshold.store(threshold, Ordering::SeqCst);
                        },
                        Err(_) => println!("Invalid threshold: {}", threshold),
                    }
                },
                ["edit_player", rating] => {
                    match rating.parse::<usize>() {
                        Ok(rating) => {
                            client.edit_player(Player { player_id: id, rating}).expect("Failed to send message");
                        },
                        Err(_) => println!("Invalid rating: {}", rating),
                    }
                },
                ["get_public_lobbies", region] => {
                    match parse_region(region) {
                        Ok(region) => {
                            client.get_public_lobbies(region).expect("Failed to send message");
                        },
                        Err(e) => println!("{e}"),
                    }
                },
                ["create_lobby", name, visibility, region, mode] => {
                    let visibility = match parse_visibility(visibility) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    };
                    let region = match parse_region(region) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    };
                    let mode = match parse_mode(mode) {
                        Ok(v) => v,
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    };
                    let params = LobbyParams {
                        name: name.to_string(),
                        visibility,
                        region,
                        mode
                    };
                    client.create_lobby(params).expect("Failed to check match with number");
                },
                _ => println!("Unknown command: {}", input),
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

fn parse_visibility(input: &str) -> Result<Visibility, String> {
    match input.to_lowercase().as_str() {
        "private" => Ok(Visibility::Private),
        "public" => Ok(Visibility::Public),
        _ => Err(format!("'{}' is not a valid Visibility", input)),
    }
}

fn parse_mode(input: &str) -> Result<GameMode, String> {
    match input.to_lowercase().as_str() {
        "casual" => Ok(GameMode::Casual),
        "competitive" => Ok(GameMode::Competitive),
        _ => Err(format!("'{}' is not a valid GameMode", input)),
    }
}

fn print_lobby(lobby: Lobby) {
    println!("##################");
    println!("ID:         {}", lobby.lobby_id);
    println!("Name:       {}", lobby.params.name);
    println!("Visibility: {:?}", lobby.params.visibility);
    println!("Region:     {:?}", lobby.params.region);
    println!("Mode:       {:?}", lobby.params.mode);
    println!("Leader:     {}", lobby.leader);
    println!("Status:     {:?}", lobby.status);
    println!("Threshold:  {:?}", lobby.queue_threshold);
    println!("------------------");
    println!("Players");
    println!("------------------");
    for player in lobby.player_list {
        println!("{player}");
    }
    println!("##################");
}
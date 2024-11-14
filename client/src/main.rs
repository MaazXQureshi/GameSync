use futures::SinkExt;
use futures_util::{future, pin_mut, StreamExt};
use gamesync_client::client::GameSyncClient;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;


#[tokio::main]
async fn main() {
    let server_url= "ws://127.0.0.1:8080/ws/";

    // Connect to the WebSocket server
    let mut client = GameSyncClient::connect(server_url).unwrap();

    let id = client.get_self().unwrap();
    let mut players = client.get_players().unwrap();
    let mut count = players.len();

    loop {
        players = client.get_players().unwrap();
        if players.len() > count {
            count = players.len();

            let msg = String::from("Hi everyone! I am ") + id.to_string().as_str();
            client.send_to_all_clients(msg).unwrap();

            let msg = String::from("Hi ") + players[0].to_string().as_str();
            client.send_to(players[0], msg).unwrap();
        }
    }

    // let num_clients = 5;

    // let mut handles = vec![];
    // for i in 0..num_clients {
    //     handles.push(tokio::spawn(async move {
    //         let mut client = GameSyncClient::new("ws://127.0.0.1:8080/ws/").unwrap();
    //         client.process_messages();
    //         client.send_chat_message(String::from("Hello, world!"));
    //     }));
    // }
    //
    // for handle in handles {
    //     handle.await.unwrap();
    // }


    // let (ws_stream, _) = connect_async(server_url).await.expect("Failed to connect");
    println!("Connected to the server");

    // let (mut write, read) = ws_stream.split();
    //
    // // Create a BufReader to read from stdin asynchronously
    // let stdin = BufReader::new(io::stdin());
    // let mut lines = stdin.lines();
    //
    // // Spawn a task to handle writing messages
    // let write_task = tokio::spawn(async move {
    //     loop {
    //         let result = lines.next_line().await;
    //         match result {
    //             Ok(Some(input)) => {
    //                 // Send the input to the WebSocket server
    //                 let msg = Message::Text(input);
    //                 write.send(msg).await.unwrap();
    //             }
    //             Ok(None) => {
    //                 // End of input stream, break out of the loop
    //                 break;
    //             }
    //             Err(e) => {
    //                 eprintln!("Error reading from stdin: {}", e);
    //                 break;
    //             }
    //         }
    //     }
    // });
    //
    // // Spawn a task to handle reading messages
    // let read_task = tokio::spawn(async move {
    //     read.for_each(|message| async {
    //         match message {
    //             Ok(msg) => match msg {
    //                 Message::Text(text) => {
    //                     println!("Received: {}", text);
    //                 }
    //                 _ => (),
    //             },
    //             Err(e) => {
    //                 eprintln!("Error: {}", e);
    //             }
    //         }
    //     })
    //     .await;
    // });
    //
    // // Wait for both tasks to finish
    // pin_mut!(write_task, read_task);
    // future::select(write_task, read_task).await;
}

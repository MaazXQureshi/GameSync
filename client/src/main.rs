use futures::SinkExt;
use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

#[tokio::main]
async fn main() {
    let server_url = Url::parse("ws://127.0.0.1:8080/ws/").unwrap();

    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(server_url).await.expect("Failed to connect");
    println!("Connected to the server");

    let (mut write, read) = ws_stream.split();

    // Create a BufReader to read from stdin asynchronously
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    // Spawn a task to handle writing messages
    let write_task = tokio::spawn(async move {
        loop {
            let result = lines.next_line().await;
            match result {
                Ok(Some(input)) => {
                    // Send the input to the WebSocket server
                    let msg = Message::Text(input);
                    write.send(msg).await.unwrap();
                }
                Ok(None) => {
                    // End of input stream, break out of the loop
                    break;
                }
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    break;
                }
            }
        }
    });

    // Spawn a task to handle reading messages
    let read_task = tokio::spawn(async move {
        read.for_each(|message| async {
            match message {
                Ok(msg) => match msg {
                    Message::Text(text) => {
                        println!("Received: {}", text);
                    }
                    _ => (),
                },
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        })
        .await;
    });

    // Wait for both tasks to finish
    pin_mut!(write_task, read_task);
    future::select(write_task, read_task).await;
}

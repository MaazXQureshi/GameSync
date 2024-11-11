use message_io::node::{self, NodeEvent, NodeHandler};
use message_io::network::{Endpoint, NetEvent, Transport};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub message: String,
    pub lobby_id: String
}

pub enum GameEvent {
    UserMessage(ChatMessageRequest),
}

pub struct Websocket {
    handler: NodeHandler<GameEvent>,
    server: Endpoint
}

impl Websocket {
    pub fn connect(url: &str) -> Result<Self, String> {
        let (handler, listener) = node::split();
        let (server, _) = handler.network().connect(Transport::Ws, url).unwrap();

        let h = handler.clone();
        listener.for_each(move |event| match event {
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Accepted(_, _) => {},
                NetEvent::Connected(_, _) => {},
                NetEvent::Message(_, message) => {
                    let payload = serde_json::from_slice(&message).unwrap();
                    h.signals().send(GameEvent::UserMessage(payload));
                },
                NetEvent::Disconnected(_) => {}
            },
            NodeEvent::Signal(msg) => {
                match msg {
                    GameEvent::UserMessage(payload) => {
                        println!("Received message: {}", serde_json::to_string_pretty(&payload).unwrap());
                    }
                }
            }
        });
        Ok(Websocket { handler, server })
    }

    pub fn send_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::UserMessage(payload) => {
                let payload = serde_json::to_string(&payload).unwrap();
                self.handler.network().send(self.server, payload.as_ref());
            }
        }
    }
}

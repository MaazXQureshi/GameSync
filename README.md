# GameSync
#### A multiplayer game networking library

---
###### Elly Wong 1006313954 ellycn.wong@mail.utoronto.ca

###### Maaz Qureshi 1006319761 maazx.qureshi@mail.utoronto.ca
---
## Video Demo
[![IMAGE ALT TEXT HERE](https://img.youtube.com/vi/gYBVlbUnfMo/0.jpg)](https://youtu.be/gYBVlbUnfMo)

https://youtu.be/gYBVlbUnfMo

## Motivation
Over the last decade, gaming has become more mainstream than ever due to the ease of access to technology and an overall increase in public awareness through successful franchises, advertisements, events, and conventions facilitated by gaming companies. In recent years there has been a subsequent rampant increase in multiplayer games due to the monetization benefits they offer and the larger player base they are able to reach. Consequently, there has been an increase in indie game developers working in small teams, or often alone, aiming to get into this stream by creating their own games from scratch. Many developers have therefore found themselves in need of building basic websocket interfaces and network infrastructure for their games from the ground up in order to implement features that are expected at the minimum in modern games, such as lobby systems and group chats. As games continue to increase in scope and complexity, developer time could be much better spent on more challenging tasks and pursuing creative avenues rather than on the reimplementation of basic networking features that are all but shared between most games nowadays.

This project was also selected because the team is interested in networks, having prior experience working in companies as network engineers, as well as completing their Capstone project which involved significant networking programming in the form of websockets and IoT network communication. The team has also previously worked with networking sockets in C which required extensive effort due to the input parsing, error handling and packet creation required in implementing the application. Debugging errors involving memory safety was especially difficult in this environment due to the dynamic nature of the messages that users could send due to the use of pointers. Furthermore, several data structures and interfaces had to be designed from scratch due to the limited library support available. The team is interested in building upon this idea by implementing a similar interface in Rust that seeks to benefit both game and web application developers by offering them an easy way to implement lobby and chat related features for their use cases. Rust, being both memory and type safe, along with offering extensive crate support with error handling seems the perfect candidate for developing a networking library. As a result, the team’s development time can be much more productive and instead focused more on implementing features rather than worrying about frustrating factors such as data races and deadlocks when working concurrency. Lastly, Rust’s performance is rated among the best. While a similar library can be implemented within Python with its expansive library support, which the team has worked with previously, the performance would suffer greatly. Rust offers similar features without sacrificing on the performance which the team feels would greatly benefit developers using our library.

By creating this library, the team would be addressing a gap in the Rust ecosystem. Due to the many benefits Rust offers, there have been many libraries and game engines developed for game creators to leverage as detailed on arewegameyet such as macroquad, ggez and bevy. There exist some networking libraries to help multiplayer client and server communication, though these are more focused on game state information such as user inputs involving character movements and interactions. While these could indeed be modified to be used for implementing lobby features, it would require further development effort which the team aims to abstract away into simple interfaces for developers to use directly similar to JavaScript’s colyseus library. The team believes this would be a useful addition to new developers looking to leverage the Rust gaming library ecosystem.

## Objective
The project’s objective is to create a networking game library focused on features such as lobby creation and management, messaging and queueing. The project involves abstracting away most of the websocket layer and implementations by providing simple interfaces and functions for use in creating basic multiplayer features. It aims to help developers that do not necessarily have extensive websocket experience re-use these features across all of their games rather than building them from scratch each time. The goal is to give developers the flexibility in their user-facing feature implementations while our library provides complete communication between server and clients with information processing and data management in the background. For instance, instead of developing an entire lobby system for messaging, the developers would only have to provide a message and lobby identifier to the library and let it handle the websocket implementation.


## Features
This library implements high-level client and server side libraries that offer a suite of pre-built solutions to common multiplayer game development functionalities. The client-side library facilitates player interactions by providing methods to send relevant player events and register callback methods for specific events. These callback methods allow developers to handle server event responses without blocking execution. The server-side library processes these events and updates game states accordingly. The project employs a low-level networking library to handle underlying client and server websocket connections and message delivery. 

The key features offered by the library are grouped as follows:

1. **Lobby management**: This feature provides methods to create, join, and manage lobbies. Lobby states are stored in-memory, keeping track of relevant attributes such as a unique identifier, a list of players, lobby status, game parameters, etc. On any new incoming events, the server notifies the relevant connected clients with the updated lobby state.

2. **Matchmaking**: Designed for large-scale games, the matchmaking feature provides methods to manage player distribution across multiple lobbies or game sessions based on player skill levels, game mode preferences, or region. The library offers two separate queues per region: competitive and casual. The competitive queue involves a matchmaking queue based on the average player skill ratings and lobby threshold parameters, while the casual queue is indifferent to skill ratings.  

3. **Information and messaging**: The library implements an event-driven model to handle real-time communication between clients and the server. It supports both group lobby messaging and one-on-one chats, with additional interfaces for lobby information querying.

The library exposes a set of public modules that encapsulate the data and methods associated with each core feature. We adopted a modular and scalable design approach that allows developers to easily extend the game state structures based on their game requirements and integrate this library with their own state management strategy. This approach promotes flexibility and simplicity, helping developers efficiently and easily build scalable multiplayer games.

## Developer Guide
The library is split into two separate crates: client and server. These share the same underlying core Structs and Enums but differ in their applications and data structure implementations.

### Struct and Enums

#### Lobby Struct

---

    pub struct Lobby {
        pub lobby_id: Uuid,
        pub params: LobbyParams,
        pub leader: PlayerID,
        pub status: LobbyStatus,
        pub player_list: Vec<PlayerID>,
        pub queue_threshold: usize
    }

This is the core struct that the entire library is structured around

`lobby_id`: unique lobby identifier

`params`: contains lobby preferences and information **(see Lobby Parameters Struct)**

`leader`: unique player identifier for lobby creator

`status`: Enum that specifies whether the lobby is Idle, Queueing or Ingame **(see LobbyStatus Enum)**

`player_list`: list of all unique identifiers for players currently in the lobby

`queue_threshold`: current threshold to use when queueing for a competitive match **(see Matchmaking section)**

#### Lobby Parameters Struct

---

    pub struct LobbyParams {
        pub name: String,
        pub visibility: Visibility,
        pub region: Region,
        pub mode: GameMode
    }

Contains lobby information and preferences

`name`: lobby name

`visibility`: Enum that specifies the lobby’s visibility **(see Visibility Enum)**

`region`: Enum that specifies the lobby’s region **(see Region Enum)**

`mode`: Enum that specifies the lobby’s game mode **(see GameMode Enum)**

#### Player Struct

---

    pub struct Player {
        pub player_id: Uuid,
        pub rating: usize
    }

Contains information about the player

`player_id`: unique player identifier

`rating`: player skill rating used in competitive matchmaking **(see Matchmaking section)**

#### Visibility Enum

---

    pub enum Visibility {
        Private,
        Public
    }

`Private`: Lobby is not publicly visible. Players must be invited to lobby to join

`Public`: Lobby is publicly visible to search for and join

#### Region Enum

---

    pub enum Region {
        NA,
        EU,
        SA,
        MEA,
        AS,
        AU
    }

`NA`: North America

`EU`: Europe

`SA`: South America

`MEA`: Middle East and Africa

`AS`: Asia

`AU`: Australia

#### GameMode Enum

---

    pub enum GameMode {
        Casual,
        Competitive
    }

`Casual`: Lobby queues for casual matchmaking

`Competitive`: Lobby queues for competitive matchmaking 

**(see Matchmaking section for more information)**

#### LobbyStatus Enum

---

    pub enum LobbyStatus {
        Idle,
        Queueing,
        Ingame
    }

`Idle`: Default state when creating lobby

`Queueing`: Lobby is queueing for a match

`Ingame`: Lobby is currently in-game

##### Lobby Restrictions
`Idle`: Lobby can be joined. Players can be edited

`Queueing/Ingame`: Lobby cannot be joined. Players cannot be edited.

##### Transitioning between lobby states:
`queue_match`: Idle -> Queueing

`stop_queue`: Queueing -> Idle

`check_match`: Queueing -> Ingame

`leave_game_as_lobby`: Ingame -> Idle

#### ServerParams Struct

---

    pub struct ServerParams {
        pub player_count: usize,
    }

`player_count`: number of players per lobby. Enforced when joining lobby and queueing

### Interfaces
The crate offers several interfaces on the client side to use to communicate between the client and the server. These are called in the following manner if the client is initialized as described in the **Initialization section**:

    client.func_name(parameters);

These functions are grouped into the following three sections:

#### Lobby Management

---


`create_lobby(params: LobbyParams)`
- Creates a lobby based on the parameters provided and makes the client the lobby leader. Can only create a lobby if the player does not currently belong to a lobby.

`join_lobby(lobby_id: Uuid)`
- Joins the lobby specified by `lobby_id`. Can only join a lobby that is not full and is in Idle state. Cannot join a lobby if the player is already part of a lobby - must leave or delete the current lobby first.

`delete_lobby(lobby_id: Uuid)`
- Deletes the lobby specified by `lobby_id`. Only lobby leaders can issue this command. On deletion, all players are evicted from the lobby. Can only delete a lobby in `Idle` state. 

`leave_lobby(lobby_id: Uuid)`
- Leaves the lobby specified by `lobby_id`. Leaving a lobby which is in the `Queueing` state will transition it to `Idle`. If a lobby leader leaves, the lobby is deleted and all players are evicted.

`invite_lobby(lobby_id: Uuid, invitee_id: Uuid)`
- Invites the player specified by invitee to the lobby specified by `lobby_id`.

`leave_game_as_lobby(lobby_id: Uuid)`
- Leaves the current game for the entire specified `lobby_id`. Transitions lobby state to `Idle`.
 
#### Matchmaking

---

`queue_lobby(lobby_id: Uuid)`

- Queues specified `lobby_id` for the appropriate matchmaking queue based on lobby’s `GameMode` parameter (`Competitive` or `Casual`). Only lobby leaders can issue this command. Transitions lobby state to `Queuing`. 

`stop_queue(lobby_id: Uuid)`

- Stops queueing the specified `lobby_id` for a match. Only lobby leaders can issue this command. Transitions lobby state to `Idle`.

`check_match(lobby_id: Uuid, threshold: Option<usize>)`

- Checks whether a match is found for the specified `lobby_id`. 
- `threshold` determines the range in which to check for a match based on average skill rating.
- For casual matchmaking, threshold is ignored (can provide `None` to the interface)
- For instance, if the average skill rating of the lobby is 1000, and the threshold is set to 500, the server will check for other lobbies with average skill ratings between 500 and 1500, accounting for the threshold conditions. 
- A `MatchFound` server event Enum will be returned in case of a match found, and `MatchNotFound` in case of no match found.

- Only lobby leaders can issue this command. Transitions lobby state to `Ingame` if a match is found. 

`edit_player(player: Player)`

- Modifies the given player with information provided in the `Player` struct (this contains the player’s skill rating).

#### Information and Messaging

---

`send_to(player_id: Uuid, message: String)`

- Sends the string `message` to a specified `player_id`

`message_lobby(lobby_id: Uuid, message: String)`

- Sends the string `message` to all players in the client’s current lobby specified by `lobby_id`

`broadcast(message: String)`

- Sends the string `message` to all currently connected clients

`get_public_lobbies(region: Region)`

- Returns all public lobbies for the specified `region`

`get_lobby_info(lobby_id: Uuid)`

- Returns `Lobby` struct for the specified `lobby_id`

### Events

The crate allows developers to register their own callback functions in response to server events.

`register_callback<F>(&mut self, callback: F) -> Result<(), GameSyncError> where F: MessageHandler + Send + 'static,`

Register a callback function that implements the `MessageHandler` trait to handle different server event messages. 

Refer to struct `MyMessageHandler` under file `gamesync_demo/client/src/main.rs` for implementation details.

### Initialization

#### Server
    async fn main(){
        let mut server = GameServer::new("8080", ServerParams { player_count: 2 }).unwrap();
        server.process_messages();
    }
 
#### Client
    async fn main(){
        let mut client = GameSyncClient::connect(server_url).unwrap();
        // Call GameSync interfaces here
    }

For more information on usage, see the example demo CLI under the **Reproducibility Guide section**.

## Reproducibility Guide
### Usage
Add the GameSync client library to your Cargo.toml file on the client-side application

    [dependencies]
    gamesync_client = "0.1.3"

Add the GameSync server library to your Cargo.toml file on the server-side application

    [dependencies]
    gamesync_server = "0.1.3"

Refer to our **Developer Guide** for the initialization and method interfaces.

### Example Usage

This section details how to build and run a comprehensive demo example which highlights the library usage. The demo involves running a server along with one or more clients. We would like to preface that our project focuses on creating a networking game library, and that user input parsing and related errors are left to be handled by the developer’s implementations since we intend to give as much flexibility as possible. The current implementation is only meant to highlight library usage and functionality, and is therefore not focused on catching errors not related to the library. Having said that, the demo does feature basic input parsing and error checking.

The following steps involve running a simple game application with two game lobbies of size 2 each, queueing for a competitive match. If the commands do not 

To run the demo, execute the following commands in order:

    cd gamesync_demo 
    cargo build
    cargo run --package server
    cargo run --package client

The last command (`cargo run --package client`) must be run multiple times to simulate multiple clients. **For the purposes of this demo, run this command 4 times to open 4 terminals** (these will be referred to as `A, B, C, D` below). 
When the client initializes, it will print a unique identifier for the player - these will be referred as `A_playerid, B_playerid, C_playerid, D_playerid`. Please replace these in the below commands with the Client ID that is printed on the terminal.

Now, commands can be issued in order as defined below. Note that `A/B/C/D` refer to the **terminal** in which the command (e.g. `invite_lobby A_lobbyid B_playerid`) must be issued.

For instance, the instruction (A - `invite_lobby A_lobbyid B_playerid`) means to replace the appropriate IDs accordingly with A's lobby ID (as detailed below), B's client ID (as detailed above) and to type this command in **terminal A**. If unclear, please refer to the demo video which follows these instructions exactly.

Continuing the demo (i.e. can issue these commands as detailed above in the terminals):

A - `create_lobby Lobby1 Private NA Competitive`

This will return the lobby information with a unique lobby identifier (labelled `ID`), and will be referred to as `A_lobbyid` from here on. Please replace this in the below commands.

A - `invite_lobby A_lobbyid B_playerid`

B - `get_lobby_info A_lobbyid`

B - `join_lobby A_lobbyid`

A - `sendto B_playerid Welcome!`

A - `edit_player 1000`

B - `edit_player 1000`

At this point, this first lobby is ready to queue. We can move on to creating the second lobby.

C - `create_lobby Lobby2 Public NA Competitive`

Same as before, this will return the lobby information with a unique lobby identifier (labelled `ID`), and will be referred to as `C_lobbyid` from here on. Please replace this in the below commands.

D - `get_public_lobbies NA`

D - `join_lobby C_lobbyid`

D - `message_lobby C_lobbyid Hello!`

C - `broadcast Ready to queue!`

C - `edit_player 3000`

D - `edit_player 3000`

Now both lobbies are ready to queue for matchmaking

A - `queue_lobby A_lobbyid`

C - `queue_lobby C_lobbyid`

Despite queueing, a game is not found due to the thresholds and average rating.

A - `update_threshold 2000`

Now Lobby1 is searching in the range of 0 to 3000, but since Lobby2’s range is 3000 to 3000 (since threshold is set to 0 by default), no match is found yet. Lobby2 has to increase their threshold as well so that the search ranges overlap.

C - `update_threshold 2000`

After a few seconds, a match should be found between both lobbies - a confirmation message will be printed to the screen stating that a match has been found against another lobby.

The below commands can be issued to leave the games and lobbies.

A - `leave_game A_lobbyid`

A - `delete_lobby A_lobbyid`

C - `leave_game C_lobbyid`

C - `leave_lobby C_lobbyid`

## Contributions

### Elly

I was primarily responsible for the set up and client-side implementation of the library. Maaz and I worked together initially to plan out the project dependencies and method interfaces. Following this, I established the foundation of the project by implementing the basic websocket connections and message handling on both sides of the library. During the feature development phase, I focused on implementing the client-side methods and callback registration. We both contributed to the testing of the libraries, which includes the development of the demo CLI tool to test the client-server websocket interactions.

### Maaz

I primarily worked on the server portion of the library. This included setting up the different data structures that the library uses concurrently to store lobby and player related information in the backend for managing different operations. Some of the core Structs and Enums were also set up or expanded upon Elly’s implementations of them. Additionally, I implemented the server functions for the three aforementioned key feature groups - lobby management, matchmaking, and information and messaging. 

I also worked alongside Elly to implement a demo CLI application that highlights our library usage (as detailed in the Reproducibility section). This included basic user input parsing for using the GameSync library interfaces and a matchmaking queueing thread implementation.


## Lessons learned and concluding remarks

The team learned that it is important, especially when creating a library, to keep all the data structures, Structs, Enums, and functions organized in all stages of development. Having the code sprawled across multiple files with no clear boundaries between different types of functions only hinders development time and reduces overall productivity as a sizable portion of time is spent trying to hunt down a particular piece of code with no idea of where to find it. Keeping the code organized into different files with a clear hierarchy helped avoid a lot of technical debt - it may have seemed slow at first but it benefitted the overall development in the long run. To this end, many of Rust’s design patterns and paradigms were followed - the library makes use of Structs, Enums, fearless concurrency and traits. 

Another lesson learned was that brainstorming and spending some time before beginning coding helps greatly as it ensures the team is on the same page regarding what the project is supposed to do, how different interfaces interact and what the purpose of each one is. We created a Google colab document that details the data structures, interfaces and overall direction of the project. This helped us stay synchronized in our individual implementations, kept us from straying away from the core purpose and ensured that each of us were updated on project’s progress.

The current state of the library covers the basic multiplayer communication needs, but there’s definitely room for further development and enhancements. Some potential future opportunities for this library include adding more flexibility for developers to integrate our library with their server-side game state management, supporting player state synchronisation, and exploring more Rust features to improve the library’s scalability and security.

In conclusion, the team found the project to be an intellectually stimulating, unique and rewarding experience having only previously worked on projects in other languages such as C/C++, Python, JavaScript. It forced us to think in a different way than we are used to, and it overall improved productivity due to the way the codebase was organized, especially when compared to OOP. The team would also like to point out that dealing with concurrency was made much easier in Rust as all the errors were converted to compile time errors. While the team may have spent more time trying to compile the library, there was very little debugging time spent on runtime errors which made the entire process all the more productive. The team has learnt a lot of great programming and design practices designing an entire Rust crate, and hopes to continue developing in Rust in the future.

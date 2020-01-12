// Common code shared by client and server.
//
// The messages sent between client and server, as well as any shared
// logic, go here. It's just a chat server right now so there's no
// logic, but the intent is to use this for a game, where logic that
// runs on both client and server would be defined here.

// License: Apache-v2.0

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum ServerToClientMessage {
    Chat{from: String, text: String},
    SetName{name: String},
    SetConnectionCount{count: u32},
}

#[derive(Serialize, Deserialize)]
pub enum ClientToServerMessage {
    Chat{text: String},
}

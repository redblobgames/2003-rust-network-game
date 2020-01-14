/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 *
 * Common code shared by client and server.
 */

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

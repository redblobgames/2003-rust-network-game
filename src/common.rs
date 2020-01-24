/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 *
 * Common code shared by client and server.
 */

use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Dir {
    North = 0, East = 1, South = 2, West = 3,
}

#[derive(Copy, Clone)]
pub enum Command {
    Move(Dir),
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub facing: Dir,
}

pub const INITIAL_PLAYER_POS: Position = Position{x: 127, y: 154, facing: Dir::South};

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerToClientMessage {
    Initialize{id: String},
    Chat{id: String, text: String},
    UpdatePlayer{id: String, pos: Position},
    DeletePlayer{id: String},
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientToServerMessage {
    Chat{text: String},
    MoveTo{pos: Position},
}

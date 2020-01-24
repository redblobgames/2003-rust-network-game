/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 *
 * Example game client for use on a web page.
 *
 * The game client handles the game logic, but not the UI or
 * networking. Those are handled by the Javascript side.
 */

#![allow(dead_code)]
#![allow(unused_macros)]

mod common;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate lazy_static;

#[cfg(target_arch = "wasm32")]
mod client {
    use wasm_bindgen::prelude::*;
    use std::collections::{HashSet, HashMap};
    use std::sync::Mutex;
    use crate::common::*;

    /* These are the JS functions we want to call from Rust */
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        fn log(s: &str);

        #[wasm_bindgen(js_namespace = connection)]
        fn send_to_server(msg: &[u8]);

        #[wasm_bindgen(js_namespace = output, js_name = push)]
        fn add_to_output(from: &str, text: &str);

        #[wasm_bindgen(js_namespace = output)]
        fn set_name(name: &str);
        
        #[wasm_bindgen(js_namespace = output)]
        fn set_connection_count(count: u32);

        #[wasm_bindgen]
        fn draw_map(player_facing: i32, player_x: i32, player_y: i32, other_player_pos: Vec<i32> /* x, y, facing */);
    }

    macro_rules! console_log {
        /* https://rustwasm.github.io/wasm-bindgen/examples/console-log.html */
        ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
    }
    
    struct World {
        needs_redraw: bool,
        keys_down: HashSet<i32>,
        map: Vec<u32>,
        map_size: (usize, usize),
        player_id: String,
        player_pos: Position,
        other_players: HashMap<String, Position>,
    }

    const KEY_UP: i32 = 38;
    const KEY_LEFT: i32 = 37;
    const KEY_DOWN: i32 = 40;
    const KEY_RIGHT: i32 = 39;
    lazy_static! {
        static ref KEYBINDINGS: HashMap<i32, Command> = [
            ('W' as i32 , Command::Move(Dir::North)),
            (KEY_UP     , Command::Move(Dir::North)),
            ('A' as i32 , Command::Move(Dir::West)),
            (KEY_LEFT   , Command::Move(Dir::West)),
            ('S' as i32 , Command::Move(Dir::South)),
            (KEY_DOWN   , Command::Move(Dir::South)),
            ('D' as i32 , Command::Move(Dir::East)),
            (KEY_RIGHT  , Command::Move(Dir::East)),
        ].iter().cloned().collect();
    }
    
    lazy_static! {
        static ref WORLD: Mutex<World> = Mutex::new(
            World {
                needs_redraw: false,
                keys_down: HashSet::new(),
                map: vec!(),
                map_size: (0, 0),
                player_id: String::from(""),
                player_pos: INITIAL_PLAYER_POS,
                other_players: HashMap::new(),
            }
        );
    }

    fn clamp(x: i32, lo: i32, hi: i32) -> i32 {
        if x < lo { lo } else if x > hi { hi } else { x }
    }
    
    // return true to redraw
    fn set_player_pos(world: &mut World, pos: Position) {
        if pos != world.player_pos {
            // TODO: check for map being passable
            world.player_pos = Position{
                x: clamp(pos.x, 0, world.map_size.0 as i32 - 1),
                y: clamp(pos.y, 0, world.map_size.1 as i32 - 1),
                facing: pos.facing,
            };
            world.needs_redraw = true;
            console_log!("player moved to {:?}", world.player_pos);
            
            let msg = ClientToServerMessage::MoveTo{pos: world.player_pos};
            let encoded: Vec<u8> = bincode::serialize(&msg).unwrap();
            send_to_server(&encoded);
        }
    }

    fn run_player_command(_world: &mut World, _command: Command) {
        // TODO: act immediately instead of waiting for tick
    }

    
    #[wasm_bindgen]
    pub fn connected() {
    }

    #[wasm_bindgen]
    pub fn set_mapdata(map_width: usize, map_height: usize, map: Vec<u32>) {
        let mut world = WORLD.lock().unwrap();
        world.map_size = (map_width, map_height);
        world.map = map;
        world.needs_redraw = true;
    }
    
    #[wasm_bindgen]
    pub fn game_loop() {
        let mut world = WORLD.lock().unwrap();
        let mut dx: i32 = 0;
        let mut dy: i32 = 0;
        let mut new_facing: Dir = world.player_pos.facing;
        for key in world.keys_down.iter() {
            if let Some(Command::Move(dir)) = KEYBINDINGS.get(&key) {
                new_facing = *dir;
                match dir {
                    Dir::North => dy -= 1,
                    Dir::East => dx += 1,
                    Dir::South => dy += 1,
                    Dir::West => dx -= 1,
                }
            }
        }
        // Clamp to -1:+1. Pressing 'A' + Left causes dx to be -2, and we want it to be -1.
        let new_pos = Position{
            x: world.player_pos.x + clamp(dx, -1, 1),
            y: world.player_pos.y + clamp(dy, -1, 1),
            facing: new_facing,
        };
        set_player_pos(&mut world, new_pos);

        if world.needs_redraw {
            world.needs_redraw = false;
            let mut other_player_pos: Vec<i32> = Vec::new();
            for (other_id, other_player) in world.other_players.iter() {
                if *other_id != world.player_id {
                    other_player_pos.push(other_player.x);
                    other_player_pos.push(other_player.y);
                    other_player_pos.push(other_player.facing as i32);
                }
            }
            draw_map(world.player_pos.facing as i32,
                     world.player_pos.x,
                     world.player_pos.y,
                     other_player_pos);
            // TODO: draw other players, excluding self
        }
    }
    
    #[wasm_bindgen]
    pub fn handle_keydown(key: i32) -> bool {
        let mut world = WORLD.lock().unwrap();
        world.keys_down.insert(key);
        if let Some(command) = KEYBINDINGS.get(&key) {
            run_player_command(&mut world, *command);
            return true;
        }
        return false;
    }

    #[wasm_bindgen]
    pub fn handle_keyup(key: i32) -> bool {
        let mut world = WORLD.lock().unwrap();
        world.keys_down.remove(&key);
        return KEYBINDINGS.get(&key).is_some()
    }
    
    #[wasm_bindgen]
    pub fn handle_text_entry(text: &str) {
        let msg = ClientToServerMessage::Chat{text: String::from(text)};
        let encoded: Vec<u8> = bincode::serialize(&msg).unwrap();
        send_to_server(&encoded);
    }

    #[wasm_bindgen]
    pub fn handle_message(data: &[u8]) {
        let mut world = WORLD.lock().unwrap();
        use ServerToClientMessage::*;
        
        let request: ServerToClientMessage = bincode::deserialize(&data).unwrap();
        console_log!("{:?}", request);
        match request {
            Initialize{id} => {
                set_name(&id);
                world.player_id = id;
                set_connection_count(1);
            },
            Chat{id, text} => {
                add_to_output(&id, &text);
            },
            UpdatePlayer{id, pos} => {
                world.other_players.insert(id, pos);
                set_connection_count(world.other_players.len() as u32);
                world.needs_redraw = true;
            },
            DeletePlayer{id} => {
                world.other_players.remove(&id);
                set_connection_count(world.other_players.len() as u32);
                world.needs_redraw = true;
            },
        };
    }
}

#[allow(dead_code)]
fn main() {}

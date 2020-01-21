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

mod common;

#[cfg(target_arch = "wasm32")]
#[macro_use]
extern crate lazy_static;

#[cfg(target_arch = "wasm32")]
mod client {
    use wasm_bindgen::prelude::*;
    use std::collections::HashSet;
    use std::sync::Mutex;
    use crate::common::*;

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
    }

    struct World {
        keys_down: HashSet<i32>,
    }
        
    lazy_static! {
    static ref WORLD: Mutex<World> = Mutex::new(
        World {
            keys_down: HashSet::new(),
        }
    );
}

    #[wasm_bindgen]
    pub fn connected() {
    }

    #[wasm_bindgen]
    pub fn handle_keydown(key: i32) {
        let mut world = WORLD.lock().unwrap();
        world.keys_down.insert(key);
        let s = format!("keys down({:?})", world.keys_down);
        log(&s);
    }

    #[wasm_bindgen]
    pub fn handle_keyup(key: i32) {
        let mut world = WORLD.lock().unwrap();
        world.keys_down.remove(&key);
        let s = format!("keyup({})", key);
        log(&s);
    }
    
    #[wasm_bindgen]
    pub fn handle_text_entry(text: &str) {
        let reply = ClientToServerMessage::Chat{text: String::from(text)};
        let encoded: Vec<u8> = bincode::serialize(&reply).unwrap();
        send_to_server(&encoded);
    }

    #[wasm_bindgen]
    pub fn handle_message(data: &[u8]) {
        let request: ServerToClientMessage = bincode::deserialize(&data).unwrap();
        match request {
            ServerToClientMessage::Chat{from, text} => add_to_output(&from, &text),
            ServerToClientMessage::SetName{name} => set_name(&name),
            ServerToClientMessage::SetConnectionCount{count} => set_connection_count(count),
        };
    }
}

#[allow(dead_code)]
fn main() {}

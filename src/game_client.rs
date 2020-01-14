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
mod client {
    use wasm_bindgen::prelude::*;
    use crate::common::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console, js_name = log)]
        fn log(s: &str);

        #[wasm_bindgen]
        fn send_to_server(msg: &[u8]);

        #[wasm_bindgen]
        fn add_to_output(from: &str, text: &str);

        #[wasm_bindgen]
        fn set_name(name: &str);
        
        #[wasm_bindgen]
        fn set_connection_count(count: u32);
    }

    #[wasm_bindgen]
    pub fn connected() {
    }

    #[wasm_bindgen]
    pub fn handle_input(text: &str) {
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

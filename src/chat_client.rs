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
        fn add_to_output(msg: &str);
    }

    #[wasm_bindgen]
    pub fn connected() {
    }

    #[wasm_bindgen]
    pub fn handle_input(text: &str) {
        let reply = Message {text: String::from(text)};
        let encoded: Vec<u8> = bincode::serialize(&reply).unwrap();
        send_to_server(&encoded);
    }

    #[wasm_bindgen]
    pub fn handle_message(data: &[u8]) {
        let request: Message = bincode::deserialize(&data).unwrap();
        add_to_output(&request.text);
    }
}

#[allow(dead_code)]
fn main() {}

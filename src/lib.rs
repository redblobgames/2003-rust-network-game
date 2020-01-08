use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(s: &str);

    #[wasm_bindgen]
    fn send_to_server(msg: &[u8]);
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    text: String,
}

#[wasm_bindgen]
pub fn connected() {
    let reply = Message {text: String::from("hello from client")};
    let encoded: Vec<u8> = bincode::serialize(&reply).unwrap();
    send_to_server(&encoded);
}

#[wasm_bindgen]
pub fn handle_message(data: &[u8]) {
    let request: Message = bincode::deserialize(&data).unwrap();
    console_log!("message received: {:?}", request.text);
}

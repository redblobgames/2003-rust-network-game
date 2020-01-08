#[cfg(target_arch = "x86_64")]
mod server {
    use std::net::TcpListener;
    use std::thread::spawn;

    use tungstenite::accept_hdr;
    use tungstenite::handshake::server::{Request};

    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct Message {
        text: String,
    }

    pub fn run() {
        let server = TcpListener::bind("localhost:9001").unwrap();
        for stream in server.incoming() {
            spawn (move || {
                let callback = |req: &Request| {
                    /* I don't really need the headers except for logging */
                    println!("Received a new ws handshake");
                    println!("The request's path is: {}", req.path);
                    println!("The request's headers are:");
                    for &(ref header, ref value) in req.headers.iter() {
                        println!("  * {} = {:?}", header, String::from_utf8((*value).to_vec()).expect("invalid utf-8"));
                    }

                    Ok(None)
                };
                
                let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();

                loop {
                    let msg = websocket.read_message().unwrap();
                    if msg.is_binary() || msg.is_text() {
                        let request: Message = bincode::deserialize(&(msg.into_data())).unwrap();
                        println!("received from client: {}", request.text);
                        
                        let reply = Message { text: String::from("reply from server") };
                        let encoded = bincode::serialize(&reply).unwrap();
                        
                        websocket.write_message(tungstenite::Message::Binary(encoded)).unwrap();
                    }
                }
            });
        }
    }
}

fn main() {
    server::run();
}


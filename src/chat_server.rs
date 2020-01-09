mod common;
    
#[cfg(target_arch = "x86_64")]
mod server {
    use std::net::TcpListener;
    use std::thread;
    use std::sync::mpsc;
    
    use tungstenite::accept_hdr;
    use tungstenite::handshake::server::{Request};

    use crate::common::*;

    pub fn run() {
        let (sim_tx, sim_rx) = mpsc::channel::<Message>();

        thread::Builder::new().name("Simulation".to_string()).spawn(move || {
            println!("SIM begin");
            for received in sim_rx {
                println!("SIM: {:?}", received.text);
            }
            println!("SIM end");
        }).unwrap();
               
        let server = TcpListener::bind("localhost:9001").unwrap();
        for stream in server.incoming() {
            let sim_tx = sim_tx.clone();
            thread::spawn(move || {
                let callback = |req: &Request| {
                    /* I don't really need the headers except for logging */
                    println!("Connection headers:");
                    if let Some(s) = req.headers.find_first("User-Agent") {
                        println!("  * User-Agent = {:}", String::from_utf8((*s).to_vec()).expect("invalid utf-8"));
                    }
                    /* TODO: how do I find out the IP of the connection? */
                    Ok(None)
                };
                
                let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();
                    
                loop {
                    match websocket.read_message() {
                        Ok(msg) => {
                            if msg.is_binary() || msg.is_text() {
                                let request: Message = bincode::deserialize(&(msg.into_data())).unwrap();
                                println!("received from client: {}", request.text);
                                
                                let reply = Message { text: String::from("reply from server") };
                                let encoded = bincode::serialize(&reply).unwrap();
                                
                                websocket.write_message(tungstenite::Message::Binary(encoded)).unwrap();
                                sim_tx.send(request).unwrap();
                            }
                        },
                        Err(err) => {
                            println!("SOCKET Error {}", err);
                            break;
                        },
                    }
                }
            });
        }
    }
}

fn main() {
    server::run();
}


mod common;
    
#[cfg(target_arch = "x86_64")]
mod server {
    use std::collections::HashMap;
    use std::{thread, time};
    use std::net::TcpListener;
    use std::sync::mpsc;
    
    use crate::common::*;

    enum Request {
        Connect(String, mpsc::Sender<Reply>),
        Player(String, Message),
        Disconnect(String),
    }

    enum Reply {
        ChatText(String),
    }
    
    pub fn run() {
        let socket_timeout = time::Duration::new(0, 500 * 1_000_000);

        let (sim_tx, sim_rx) = mpsc::channel::<Request>();

        thread::spawn(move || {
            let mut channels = HashMap::new();
            println!("SIM: begin");
            for request in sim_rx {
                match request {
                    Request::Connect(addr, net_tx) => {
                        println!("SIM: connect from {}", addr);
                        channels.insert(addr, net_tx);
                    },
                    Request::Player(addr, Message{text}) => {
                        println!("SIM: message from {}: {}", addr, text);
                        for (_addr, net_tx) in &channels {
                            net_tx.send(Reply::ChatText(format!("[{}] {}", addr, text))).unwrap();
                        }
                    },
                    Request::Disconnect(addr) => {
                        println!("SIM: disconnect from {}", addr);
                        channels.remove(&addr);
                    },
                };
            }
            println!("SIM: end");
        });
               
        let server = TcpListener::bind("localhost:9001").unwrap();
        for stream in server.incoming() {
            let stream = stream.unwrap();
            stream.set_read_timeout(Some(socket_timeout)).unwrap();
            let sim_tx = sim_tx.clone();
            let addr = format!("{:?}", stream.peer_addr().unwrap());
            let (net_tx, net_rx) = mpsc::channel::<Reply>();
            
            thread::spawn(move || {
                let mut websocket = tungstenite::accept(stream).unwrap();
                sim_tx.send(Request::Connect(addr.clone(), net_tx)).unwrap();

                // This loop handles both player events (from the
                // network, over websocket) and simulation events
                // (from the simulation thread, over the message
                // passing queue)
                loop {
                    match websocket.read_message() {
                        Ok(msg) if msg.is_binary() => {
                            let request: Message = bincode::deserialize(&(msg.into_data())).unwrap();
                            sim_tx.send(Request::Player(addr.clone(), request)).unwrap();
                        },
                        Ok(msg) if msg.is_close() => {
                            println!("Closed");
                            break;
                        },
                        Ok(msg) => {
                            println!("received unexpected msg {:?}", msg);
                        },
                        Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // The socket read timeout gives me a chance to check the message queue
                        },
                        Err(err) => {
                            println!("{:?} ERRRRRRRR", err);
                            break;
                        },
                    }

                    for message in net_rx.try_iter() {
                        match message {
                            Reply::ChatText(text) => {
                                let reply = Message{text};
                                let encoded = bincode::serialize(&reply).unwrap();
                                websocket.write_message(tungstenite::Message::Binary(encoded)).unwrap();
                            },
                        };
                    }
                }
                sim_tx.send(Request::Disconnect(addr)).unwrap();
            });
        }
    }
}

fn main() {
    server::run();
}


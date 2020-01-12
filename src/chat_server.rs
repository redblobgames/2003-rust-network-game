// Example chat server
//
// I decided to use one thread per network connection instead of using
// mio/tokio. The network thread both listens to the network and
// listens to a message queue. The simulation thread (currently chat,
// but intended to be a game) listens for messages from the network
// threads, and can write back to the network threads.

// License: Apache-v2.0

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
        Player(String, ClientToServerMessage),
        Disconnect(String),
    }

    enum Reply {
        Player(ServerToClientMessage),
    }

    // https://en.wikipedia.org/wiki/List_of_Egyptian_deities
    const NAMES: &[&str] = &["Anubis", "Wosret", "Pakhet", "Aten", "Anuket", "Isis", "Maat",
                             "Nefertum", "Ra", "Thoth", "Khepri", "Kek", "Ba'alat", "Mafdet",
                             "Qerhet", "Satet", "Esna", "Thmei", "Tafner", "Unnit", "Apep"];
    
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
                        net_tx.send(Reply::Player(ServerToClientMessage::SetName{name: addr.clone()})).unwrap();
                        channels.insert(addr, net_tx);
                        for net_tx in channels.values() {
                            net_tx.send(Reply::Player(ServerToClientMessage::SetConnectionCount{count: channels.len() as u32})).unwrap();
                        }
                    },
                    Request::Player(addr, ClientToServerMessage::Chat{text}) => {
                        println!("SIM: message from {}: {}", addr, text);
                        for net_tx in channels.values() {
                            net_tx.send(Reply::Player(ServerToClientMessage::Chat{from: addr.clone(), text: text.clone()})).unwrap();
                        }
                    },
                    Request::Disconnect(addr) => {
                        println!("SIM: disconnect from {}", addr);
                        channels.remove(&addr);
                        for net_tx in channels.values() {
                            net_tx.send(Reply::Player(ServerToClientMessage::SetConnectionCount{count: channels.len() as u32})).unwrap();
                        }
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
            let port: u16 = stream.peer_addr().unwrap().port();
            let addr = {
                let id: usize = ((port >> 8) | ((port & 0xff) << 8)) as usize;
                format!("{}{}", NAMES[id % NAMES.len()], id / NAMES.len())
            };
            println!("Network connection from {:?} mapped to {}", stream.peer_addr().unwrap(), addr);
            let (net_tx, net_rx) = mpsc::channel::<Reply>();

            thread::spawn(move || {
                let callback = |req: &tungstenite::handshake::server::Request| {
                    /* I don't really need the headers except for logging */
                    if let Some(s) = req.headers.find_first("X-Real-IP") {
                        println!("IP for {} = {}", addr, String::from_utf8((*s).to_vec()).expect("invalid utf-8"));
                    }
                    Ok(None)
                };
            
                let mut websocket = tungstenite::accept_hdr(stream, callback).unwrap();
                sim_tx.send(Request::Connect(addr.clone(), net_tx)).unwrap();

                // This loop handles both player events (from the
                // network, over websocket) and simulation events
                // (from the simulation thread, over the message
                // passing queue)
                loop {
                    match websocket.read_message() {
                        Ok(msg) if msg.is_binary() => {
                            let request: ClientToServerMessage = bincode::deserialize(&(msg.into_data())).unwrap();
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
                            Reply::Player(reply) => {
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

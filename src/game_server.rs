/*
 * From https://www.redblobgames.com/x/2003-rust-network-game/
 * Copyright 2020 Red Blob Games <redblobgames@gmail.com>
 * License: Apache-2.0 <http://www.apache.org/licenses/LICENSE-2.0.html>
 *
 * Example game server
 *
 * Each connection is handle by a network thread, which both listens
 * to the network and listens to a message queue. The simulation
 * thread listens for messages from the network threads, and can write
 * back to the network threads.
 */

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

    struct PlayerData {
        channel: mpsc::Sender<Reply>,
        player_pos: Position,
    }
    
    // https://en.wikipedia.org/wiki/List_of_Egyptian_deities
    const NAMES: &[&str] = &[
        "Anubis", "Wosret", "Pakhet", "Aten", "Anuket", "Isis", "Maat",
        "Nefertum", "Ra", "Thoth", "Khepri", "Kek", "Ba'alat", "Mafdet",
        "Qerhet", "Satet", "Esna", "Thmei", "Tafner", "Unnit", "Apep"
    ];
    
    pub fn run() {
        let socket_timeout = time::Duration::new(0, 500 * 1_000_000);

        let (sim_tx, sim_rx) = mpsc::channel::<Request>();

        thread::spawn(move || {
            use ServerToClientMessage::*;
            
            let mut players: HashMap<String, PlayerData> = HashMap::new();
            println!("SIM: begin");
            for request in sim_rx {
                match request {
                    Request::Connect(addr, net_tx) => {
                        println!("SIM: connect from {}", addr);
                        net_tx.send(Reply::Player(Initialize{id: addr.clone()})).unwrap();
                        for (other_id, other_player) in players.iter() {
                            net_tx.send(Reply::Player(UpdatePlayer{id: other_id.clone(), pos: other_player.player_pos})).unwrap();
                        }
                        players.insert(addr.clone(), PlayerData{channel: net_tx, player_pos: INITIAL_PLAYER_POS});
                        for player in players.values() {
                            player.channel.send(Reply::Player(UpdatePlayer{id: addr.clone(), pos: INITIAL_PLAYER_POS})).unwrap();
                        }
                    },
                    Request::Player(addr, ClientToServerMessage::Chat{text}) => {
                        println!("SIM: message from {}: {}", addr, text);
                        for player in players.values() {
                            player.channel.send(Reply::Player(Chat{id: addr.clone(), text: text.clone()})).unwrap();
                        }
                    },
                    Request::Player(addr, ClientToServerMessage::MoveTo{pos}) => {
                        println!("SIM: move {} to {:?}", addr, pos);
                        let mut player = players.get_mut(&addr).unwrap();
                        player.player_pos = pos;
                        for player in players.values() {
                            player.channel.send(Reply::Player(UpdatePlayer{id: addr.clone(), pos})).unwrap();
                        }
                    },
                    Request::Disconnect(addr) => {
                        println!("SIM: disconnect from {}", addr);
                        players.remove(&addr);
                        for player in players.values() {
                            match player.channel.send(Reply::Player(DeletePlayer{id: addr.clone()})) {
                                Ok(_) => (),
                                Err(e) => println!("   error {}", e), // TODO: "sending on a closed channel" if two clients disconnect at same time
                            };
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

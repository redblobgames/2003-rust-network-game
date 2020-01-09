mod common;
    
#[cfg(target_arch = "x86_64")]
mod server {
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
    }
    
    pub fn run() {
        let socket_timeout = time::Duration::new(0, 500 * 1_000_000);

        let (sim_tx, sim_rx) = mpsc::channel::<Request>();

        thread::spawn(move || {
            println!("SIM: begin");
            for request in sim_rx {
                match request {
                    Request::Connect(addr, _net_tx) => println!("SIM: connect from {}", addr), /* TODO: save the net_tx in a map */
                    Request::Player(addr, Message{text}) => println!("SIM: message from {}: {}", addr, text),
                    Request::Disconnect(addr) => println!("SIM: disconnect from {}", addr), /* TODO: remove corresponding net_tx */
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
            let (net_tx, _net_rx) = mpsc::channel::<Reply>();
            
            thread::spawn(move || {
                let mut websocket = tungstenite::accept(stream).unwrap();
                sim_tx.send(Request::Connect(addr.clone(), net_tx)).unwrap();

                let reply = Message { text: String::from("hello from server") };
                let encoded = bincode::serialize(&reply).unwrap();
                websocket.write_message(tungstenite::Message::Binary(encoded)).unwrap();
                
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
                            // TODO: This gives me a chance to check the message queue!!
                            println!("{:?} no bytes to read", time::SystemTime::now());
                        },
                        Err(err) => {
                            println!("{:?} ERRRRRRRR", err);
                            break;
                        },
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


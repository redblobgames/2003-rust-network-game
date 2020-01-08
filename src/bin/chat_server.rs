#[cfg(target_arch = "x86_64")]
mod server {
    use std::net::TcpListener;
    use std::thread::spawn;

    use tungstenite::accept_hdr;
    use tungstenite::handshake::server::{Request};

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
                        let msg2 = tungstenite::Message::Binary(vec!(1u8, 2u8, 3u8));
                        websocket.write_message(msg2).unwrap();
                    }
                }
            });
        }
    }
}

fn main() {
    server::run();
}


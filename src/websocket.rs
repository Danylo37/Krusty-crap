use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use tungstenite::{accept, Message, Utf8Bytes};
use crossbeam_channel::Receiver;

pub fn start_websocket_server(rx: Receiver<String>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("WebSocket server started on ws://0.0.0.0:8080");

    // Use SocketAddr as unique identifier
    let clients = Arc::new(Mutex::new(HashMap::<std::net::SocketAddr, TcpStream>::new()));

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let clients = Arc::clone(&clients);
                    let rx = rx.clone();

                    // Store connection
                    {
                        let mut clients = clients.lock().unwrap();
                        clients.insert(addr, stream.try_clone().unwrap());
                    }

                    thread::spawn(move || {
                        let mut websocket = accept(stream).unwrap();

                        while let Ok(msg) = rx.recv() {
                            if let Err(e) = websocket.write_message(Message::Text(Utf8Bytes::from(msg))) {
                                eprintln!("Write error: {}", e);
                                break;
                            }
                        }

                        // Remove on disconnect
                        let mut clients = clients.lock().unwrap();
                        clients.remove(&addr);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}
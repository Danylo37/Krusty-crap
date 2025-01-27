use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use log::info;
use tungstenite::{accept, Message, Utf8Bytes};
use crate::general_use::{ClientId, DroneId, ServerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsCommand {
    WsUpdateData,
    WsDroneCommand(DroneId, DroneCommandWs),
    WsClientCommand(ClientId, ClientCommandWs),
    WsServerCommand(ServerId, ServerCommandWs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DroneCommandWs {
    Crash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientCommandWs {
    UpdateMonitoringData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerCommandWs {
    UpdateMonitoringData,
}

/*
pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand> ) {
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
                            println!("Message: {:?}", msg);
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
}*/

/*
// Start WebSocket server with bidirectional communication
pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand> ) {
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
                            println!("Message: {:?}", msg);
                            if let Err(e) = websocket.write_message(Message::Text(Utf8Bytes::from(msg))) {
                                eprintln!("Write error: {}", e);
                                break;
                            }
                        }

                        // READ LOOP
                        while let Ok(msg) = websocket.read_message() {
                            if let Message::Text(text) = msg {
                                if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                                    cmd_tx.send(cmd).unwrap();
                                }
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
}*/
/*
pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("WebSocket server started on ws://0.0.0.0:8080");

    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let clients = Arc::clone(&clients);
                    let cmd_tx = cmd_tx.clone();
                    let rx = rx.clone();

                    // Store connection
                    {
                        let mut clients = clients.lock().unwrap();
                        clients.insert(addr, stream.try_clone().unwrap());
                    }

                    thread::spawn(move || {
                        let websocket = Arc::new(Mutex::new(accept(stream).unwrap()));

                        // Spawn a thread for reading from the WebSocket
                        let websocket_read = Arc::clone(&websocket);
                        let cmd_tx_clone = cmd_tx.clone();
                        let read_thread = thread::spawn(move || {
                            let mut websocket = websocket_read.lock().unwrap();
                            while let Ok(msg) = websocket.read_message() {
                                eprintln!("We are in the read loop");
                                if let Message::Text(text) = msg {
                                    eprintln!("The command is {}", text);
                                    if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                                        eprintln!("The command is sent {:?}", cmd);
                                        cmd_tx_clone.send(cmd).unwrap();
                                    }
                                }
                            }
                        });

                        // Spawn a thread for writing to the WebSocket
                        let websocket_write = Arc::clone(&websocket);
                        let write_thread = thread::spawn(move || {
                            while let Ok(msg) = rx.recv() {
                                println!("Message: {:?}", msg);
                                let mut websocket = websocket_write.lock().unwrap();
                                if let Err(e) = websocket.write_message(Message::text(msg)) {
                                    eprintln!("Write error: {}", e);
                                    break;
                                }
                            }
                        });

                        // Wait for both threads to finish
                        read_thread.join().unwrap();
                        write_thread.join().unwrap();

                        // Cleanup on disconnect
                        let mut clients = clients.lock().unwrap();
                        clients.remove(&addr);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}*/

pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("WebSocket server started on ws://0.0.0.0:8080");

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let cmd_tx = cmd_tx.clone();
                    let rx = rx.clone();

                    thread::spawn(move || {
                        let mut websocket = accept(stream).unwrap();
                        println!("Client connected: {}", addr);

                        // Main loop to handle WebSocket communication
                        loop {
                            // Read message from WebSocket
                            match websocket.read_message() {
                                Ok(Message::Text(text)) => {
                                    println!("Received: {}", text);
                                    // Parse the command
                                    if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                                        if let WsCommand::WsUpdateData = cmd {
                                            println!("WsUpdateData command received");

                                            // Notify the backend to prepare updated data
                                            if let Err(e) = cmd_tx.send(cmd) {
                                                eprintln!("Failed to send command to backend: {}", e);
                                                break;
                                            }

                                            // Wait for updated data from the backend
                                            match rx.recv() {
                                                Ok(updated_data) => {
                                                    println!("Sending updated data: {}", updated_data);
                                                    if let Err(e) = websocket.write_message(Message::text(updated_data)) {
                                                        eprintln!("Failed to send data to WebSocket: {}", e);
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    eprintln!("Failed to receive updated data: {}", e);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    println!("Client disconnected: {}", addr);
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("WebSocket error: {}", e);
                                    break;
                                }
                                _ => {} // Ignore other message types
                            }
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}

use crossbeam_channel::{select, select_biased, unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::net::{TcpListener, TcpStream};
use std::thread;
use log::{info, warn};
use tungstenite::{accept, Message};
use tungstenite::error::Error as WsError;
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
                                                    if let Err(e) = websocket.send(Message::text(updated_data)) {
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
}*/
pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("WebSocket server started on ws://0.0.0.0:8080");

    let (ws_tx, ws_rx) = unbounded(); // Unbounded WebSocket command channel

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let cmd_tx = cmd_tx.clone();
                    let rx = rx.clone();
                    let ws_rx = ws_rx.clone();
                    let ws_tx = ws_tx.clone();

                    thread::spawn(move || {
                        let mut websocket = accept(stream).unwrap();
                        info!("Client connected: {}", addr);

                        // Set WebSocket to non-blocking mode
                        websocket.get_mut().set_nonblocking(true).unwrap();

                        loop {
                            select! {
                                // Process backend messages (send updates to the WebSocket)
                                recv(rx) -> wrapped_data => {
                                    match wrapped_data {
                                        Ok(data) => {
                                            info!("Sending updated data: {}", data);
                                            if let Err(e) = websocket.send(Message::text(data)) {
                                                warn!("Failed to send data to WebSocket: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Error receiving backend data: {}", e);
                                            break;
                                        }
                                    }
                                },

                                // Process WebSocket messages (handle commands from clients)
                                recv(ws_rx) -> wrapped_command => {
                                    match wrapped_command {
                                        Ok(cmd) => {
                                            if let Err(e) = cmd_tx.send(cmd) {
                                                warn!("Failed to send command to backend: {}", e);
                                                break;
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Error receiving WebSocket command: {}", e);
                                            break;
                                        }
                                    }
                                },

                                // Non-blocking WebSocket read
                                default => {
                                    match websocket.read() {
                                        Ok(Message::Text(text)) => {
                                            if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                                                ws_tx.send(cmd).unwrap();
                                            }
                                        }
                                        Err(WsError::Io(ref err)) if err.kind() == std::io::ErrorKind::WouldBlock => {
                                            // No message received, continue loop (without blocking)
                                        }
                                        Err(e) => {
                                            warn!("WebSocket error: {}", e);
                                            break;
                                        }
                                    _ => {}}
                                }
                            }
                        }

                        println!("Client disconnected: {}", addr);
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}
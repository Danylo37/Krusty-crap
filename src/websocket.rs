use crossbeam_channel::{select, unbounded, Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use log::{info, warn};
use tungstenite::{accept, Message, Utf8Bytes};
use tungstenite::error::Error as WsError;
use crate::general_use::{ClientId, DroneId, FileRef, MediaRef, ServerId};

// Helper module for handling u64 as strings in JSON
mod stringified_u8 {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de::Error;
    pub fn serialize<S: Serializer>(value: &u8, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u8, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(Error::custom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsCommand {
    WsUpdateData,

    WsAskListRegisteredClientsToServer {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        #[serde(with = "stringified_u8")]
        server_id: ServerId,
    },

    WsSendMessage {
        #[serde(with = "stringified_u8")]
        source_client_id: ClientId,
        #[serde(with = "stringified_u8")]
        dest_client_id: ClientId,
        message: String,
    },

    WsAskFileList {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        #[serde(with = "stringified_u8")]
        server_id: ServerId,
    },

    WsAskFileContent {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        #[serde(with = "stringified_u8")]
        server_id: ServerId,
        file_ref: FileRef,
    },

    WsAskMedia {
        #[serde(with = "stringified_u8")]
        client_id: ClientId,
        media_ref: MediaRef,
    },

    WsCrashDrone{
        #[serde(with = "stringified_u8")]
        drone_id: DroneId,
    }
}

// A type alias for the global list of client inboxes.
// Each client (internet WebSocket connection) gets its own Sender<String> for receiving broadcast updates.
type ClientList = Arc<Mutex<Vec<Sender<String>>>>;

pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    let clients: ClientList = Arc::new(Mutex::new(Vec::new()));

    // Spawn a dedicated broadcaster thread.
    // This thread will receive every backend update from `rx`
    // and then send a copy to each client's sender.
    {
        let clients = Arc::clone(&clients);
        thread::spawn(move || {
            // Note: we use rx.recv() here so that every update is handled exactly once.
            while let Ok(data) = rx.recv() {
                // Send the update to every client.
                let mut client_list = clients.lock().unwrap();
                // retain only those clients for which sending was successful.
                client_list.retain(|client| client.send(data.clone()).is_ok());
            }
        });
    }

    // Spawn a thread that listens for incoming WebSocket connections.
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let cmd_tx = cmd_tx.clone();
                    let clients = Arc::clone(&clients);

                    thread::spawn(move || {
                        // Accept the WebSocket handshake.
                        let mut websocket = accept(stream).unwrap();
                        println!("Client connected: {}", addr);

                        // Create a channel that will serve as this client's inbox.
                        let (client_tx, client_rx) = unbounded::<String>();

                        // Add this client to the global list.
                        {
                            let mut client_list = clients.lock().unwrap();
                            client_list.push(client_tx.clone());
                        }

                        // Set nonblocking so that websocket.read() does not block indefinitely.
                        websocket.get_mut().set_nonblocking(true).unwrap();

                        // Main loop for handling this client.
                        loop {
                            select! {
                                // Here we receive messages from our dedicated inbox.
                                // These messages have been broadcast from the backend or from other WebSocket clients.
                                recv(client_rx) -> msg => {
                                    if let Ok(data) = msg {
                                        // Send the received broadcast update to the websocket.
                                        if let Err(e) = websocket.send(Message::Text(Utf8Bytes::from(data))) {
                                            warn!("Failed to send data to WebSocket: {}", e);
                                            break;
                                        }
                                    }
                                },

                                // Default branch: check for incoming messages from the WebSocket.
                                default => {
                                    match websocket.read() {
                                        Ok(Message::Text(text)) => {
                                            info!("WebSocket Message Received: {}", text);
                                            // Parse incoming text as a WsCommand.
                                            if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                                                info!("Parsed command: {:?}", cmd);
                                                // Forward the command to your backend.
                                                if let Err(e) = cmd_tx.send(cmd.clone()) {
                                                    warn!("Failed to send command to backend: {}", e);
                                                    break;
                                                }
                                                // Optionally, you might want to broadcast the command as well.
                                                let broadcast_msg = serde_json::to_string(&cmd).unwrap();
                                                let mut client_list = clients.lock().unwrap();
                                                client_list.retain(|client| client.send(broadcast_msg.clone()).is_ok());
                                            } else {
                                                info!("Failed to parse message: {}", text);
                                            }
                                        }
                                        // If no message is ready, continue the loop.
                                        Err(WsError::Io(ref err)) if err.kind() == std::io::ErrorKind::WouldBlock => {},
                                        Err(e) => {
                                            warn!("WebSocket error: {}", e);
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }

                        println!("Client disconnected: {}", addr);
                        // Remove this client from the global list upon disconnect.
                        let mut client_list = clients.lock().unwrap();
                        client_list.retain(|client| !std::ptr::eq(client, &client_tx));
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}

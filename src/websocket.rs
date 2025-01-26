use crossbeam_channel::{ Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use tungstenite::{accept, Message};
use wg_2024::controller::DroneCommand;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use crate::general_use::{ClientCommand, ClientId, DroneId, ServerCommand, ServerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsCommand {
    WsUpdateData,  //in general, it asks all the nodes to send the data to the monitor
    WsDroneCommand(DroneId, DroneCommandWs),
    WsClientCommand(ClientId ,ClientCommandWs),
    WsServerCommand(ServerId, ServerCommandWs),
}

//Just this to implement the Serialize and Deserialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DroneCommandWs{
    //for now just crash, we will see when we have more buttons
    Crash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientCommandWs{
    //for now just update the monitoring data, we will see when we have more buttons
    UpdateMonitoringData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerCommandWs{
    //for now just update the monitoring data, we will see when we have more buttons
    UpdateMonitoringData,
}


pub fn start_websocket_server(rx: Receiver<String>, cmd_tx: Sender<WsCommand>) {
    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("WebSocket server started on ws://0.0.0.0:8080");

    let clients = Arc::new(Mutex::new(HashMap::<SocketAddr, TcpStream>::new()));

    // Spawn thread to handle incoming connections
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let addr = stream.peer_addr().unwrap();
                    let clients = Arc::clone(&clients);
                    let cmd_tx = cmd_tx.clone();

                    // Store connection
                    {
                        let mut clients = clients.lock().unwrap();
                        clients.insert(addr, stream.try_clone().unwrap());
                    }

                    // Spawn separate thread for this connection
                    thread::spawn(move || {
                        let websocket = accept(stream).unwrap();
                        let websocket = Arc::new(Mutex::new(websocket)); // Wrap in Arc<Mutex<>> for shared access

                        // Channel for sending messages to the WebSocket
                        let (tx, rx) = crossbeam_channel::unbounded::<Message>();

                        // READ LOOP (handle incoming commands)
                        let cmd_tx_clone = cmd_tx.clone();
                        let websocket_reader = Arc::clone(&websocket);
                        let tx_clone = tx.clone();
                        thread::spawn(move || {
                            while let Ok(msg) = websocket_reader.lock().unwrap().read() {
                                if let Message::Text(text) = msg {
                                    if let Ok(cmd) = serde_json::from_str::<WsCommand>(&text) {
                                        cmd_tx_clone.send(cmd).unwrap();
                                    }
                                }
                            }
                            // If the read loop ends, signal the write loop to stop
                            let _ = tx_clone.send(Message::Close(None));
                        });

                        // WRITE LOOP (send updates to the client)
                        let websocket_writer = Arc::clone(&websocket);
                        thread::spawn(move || {
                            for msg in rx.iter() {
                                if let Err(e) = websocket_writer.lock().unwrap().write(msg) {
                                    eprintln!("Write error: {}", e);
                                    break;
                                }
                            }

                            // Cleanup after the connection is closed
                            let mut clients = clients.lock().unwrap();
                            clients.remove(&addr);
                        });
                    });

                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    });
}
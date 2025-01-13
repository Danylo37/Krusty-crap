//Mod components
mod network_initializer;
mod servers;

mod simulation_controller;
mod general_use;
mod ui;

mod clients;
pub mod ui_traits;
mod connecting_websocket;

use crossbeam_channel;
use crossbeam_channel::unbounded;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::simulation_controller::SimulationController;
use crate::ui::start_ui;

#[tokio::main]  //HERE YOU ARE EXPLICITLY USING MULTITHREADED RUNTIME WITH TOKIO SO SURE THAT YOU ARE
                //NOT RUNNING EVERYTHING IN ONE THREAD.
async fn main() {
    let url = "ws://localhost:8000";

    println!("Connecting to {}", url);
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to WebSocket");
    let (mut write, _) = ws_stream.split();

  
    // Create an asynchronous channel for communication between clients and WebSocket writer task
    let (tx, mut rx) = mpsc::channel::<String>(1000);

    // Network initializer instance
    let mut my_net = network_initializer::NetworkInitializer::new(tx.clone());
    my_net.initialize_from_file("input.toml");

    // Spawn a task for writing messages to the WebSocket
    let ws_handle = tokio::spawn(async move {
        loop {
            while let Some(msg) = rx.recv().await {
                if let Err(err) = write.send(Message::Text(msg)).await {
                    eprintln!("Error writing to WebSocket: {:?}", err);
                    break;
                }
            }
        }
    });

    // Spawn a task for the UI interaction
    let ui_handle = tokio::spawn(async move {
        start_ui(my_net.simulation_controller).await; // Access the controller field directly
    });

    // Wait for either task to complete or for a shutdown signal
    tokio::select! {
        _ = ws_handle => eprintln!("WebSocket task terminated"),
        _ = ui_handle => eprintln!("UI task terminated"),
        _ = tokio::signal::ctrl_c() => {
            println!("Received shutdown signal. Terminating...");
        }
    }
}


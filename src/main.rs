mod network_initializer;
mod servers;
mod simulation_controller;
mod general_use;
mod ui;
mod clients;
pub mod ui_traits;
mod connecting_websocket;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use crossbeam_channel::{unbounded, Sender, Receiver, TryRecvError};
use tungstenite::{connect, Message, Utf8Bytes};
//use crate::simulation_controller::SimulationController;
use crate::ui::start_ui;

fn main() {
    let url = "ws://localhost:8000";

    // Create communication channels
    let (tx, rx) = unbounded();

    // Initialize network
    let mut my_net = network_initializer::NetworkInitializer::new(tx.clone());
    my_net.initialize_from_file("input.toml");

    // Shared flag for graceful shutdown
    let running = Arc::new(Mutex::new(true));
    let r = running.clone();

    // WebSocket writer thread
    let ws_thread = thread::spawn(move || {
        let mut socket = connect(url)
            .expect("Failed to connect")
            .0;

        while *running.lock().unwrap() {
            match rx.try_recv() {
                Ok(msg) => {
                    socket.send(Message::Text(Utf8Bytes::from(msg)))
                        .expect("Failed to write message");
                }
                Err(TryRecvError::Empty) => {
                    thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(_) => break,
            }
        }
    });

    // UI thread
    let ui_thread = thread::spawn(move || {
        start_ui(my_net.simulation_controller);
    });

    // Handle Ctrl+C
    ctrlc::set_handler(move || {
        *r.lock().unwrap() = false;
    }).expect("Error setting Ctrl+C handler");

    // Wait for threads
    ws_thread.join().expect("WebSocket thread panicked");
    ui_thread.join().expect("UI thread panicked");

    println!("Shutting down gracefully");
}
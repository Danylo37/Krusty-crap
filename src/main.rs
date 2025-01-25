mod network_initializer;
mod servers;
mod simulation_controller;
mod general_use;
mod ui;
mod clients;
pub mod ui_traits;
mod websocket;

extern crate rouille;
use std::thread;
use std::time::Duration;
use crossbeam_channel::{unbounded, Sender};


// Modified main function
fn main() {
    // Initialize the logger
    env_logger::init();

    let (tx, rx) = unbounded();
    let mut my_net = network_initializer::NetworkInitializer::new(tx.clone());
    my_net.initialize_from_file("input.toml");

    // Start WebSocket server

    let (sender_from_ws, receiver_from_ws) = unbounded();

    websocket::start_websocket_server(rx, sender_from_ws);

    // Start HTTP server for web interface
    thread::spawn(|| {
        println!("HTTP server started on http://0.0.0.0:8000");
        rouille::start_server("0.0.0.0:8000", move |request| {
            rouille::match_assets(&request, "static")
        });
    });

    // UI thread
    let ui_thread = thread::spawn(move || {
        ui::start_ui(my_net.simulation_controller);
    });

    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}


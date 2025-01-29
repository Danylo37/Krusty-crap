pub mod network_initializer;
pub mod servers;
pub mod simulation_controller;
pub mod general_use;
pub mod ui;
pub mod clients;
pub mod ui_traits;
pub mod websocket;
pub mod terminal_ui;

extern crate rouille;

use std::thread;
use std::time::Duration;
use crossbeam_channel::{unbounded};
use crate::ui_traits::Monitoring;

// Modified main function
fn main() {
    // Initialize the logger
    env_logger::init();

    // Create channels for communication
    let (tx, rx) = unbounded();
    let (sender_from_ws, receiver_from_ws) = unbounded();

    // Run the simulation controller
    thread::spawn(move || {
        // Initialize the network
        let mut my_net = network_initializer::NetworkInitializer::new(tx.clone(), receiver_from_ws);
        my_net.initialize_from_file("input.toml");
        // Clone the shared simulation controller
        let mut simulation_controller = my_net.simulation_controller;
        eprintln!("Simulation Controller is running");
        simulation_controller.run_with_monitoring(tx.clone());
    });


    // Start WebSocket server
    websocket::start_websocket_server(rx, sender_from_ws);

    // Start HTTP server for web interface
    thread::spawn(|| {
        println!("HTTP server started on http://0.0.0.0:8000");
        rouille::start_server("0.0.0.0:8000", move |request| {
            rouille::match_assets(&request, "static")
        });
    });

    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
/*
fn main() {
    let (tx, rx) = unbounded();
    let (sender_from_ws, receiver_from_ws) = unbounded();

    // Initialize the network
    let mut my_net = network_initializer::NetworkInitializer::new(tx.clone(), receiver_from_ws);
    my_net.initialize_from_file("input.toml");
    // Spawn UI thread
    eprintln!("UI thread is running");
    ui::start_ui(my_net.simulation_controller);

}*
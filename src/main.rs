pub mod network_initializer;
pub mod servers;
pub mod simulation_controller;
pub mod general_use;
pub mod clients;
pub mod ui_traits;
pub mod websocket;
pub mod initialization_file_checker;
pub mod terminal_messages;

extern crate rouille;

use std::thread;
use std::time::Duration;
use crossbeam_channel::{unbounded};
use crate::ui_traits::{SimulationControllerMonitoring};

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
        let mut my_net = network_initializer::NetworkInitializer::new(receiver_from_ws);
        my_net.initialize_from_file("topologies/tree.toml");
        // Clone the shared simulation controller
        let mut simulation_controller = my_net.simulation_controller;
        println!(
            "\n\
    ┌──────────────────────────────────────────────────┐\n\
    │   🚀 To use the application, visit:              │\n\
    │   🌍 http://localhost:8000/index.html            │\n\
    └──────────────────────────────────────────────────┘\n"
        );
        simulation_controller.run_with_monitoring(tx.clone());
    });


    // Start WebSocket server
    websocket::start_websocket_server(rx, sender_from_ws);

    // Start HTTP server for web interface
    thread::spawn(|| {
        //println!("HTTP server started on http://0.0.0.0:8000");
        rouille::start_server("0.0.0.0:8000", move |request| {
            rouille::match_assets(&request, "static")
        });
    });

    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

mod network_initializer;
mod servers;
mod simulation_controller;
mod general_use;
mod ui;
mod clients;
pub mod ui_traits;
mod websocket;

extern crate rouille;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crossbeam_channel::{unbounded, Sender};
use log::info;
use crate::ui_traits::Monitoring;

// Modified main function
fn main() {
    // Initialize the logger
    env_logger::init();

    let (tx, rx) = unbounded();
    let (sender_from_ws, receiver_from_ws) = unbounded();

    thread::spawn(move || {
        let mut my_net = network_initializer::NetworkInitializer::new(tx.clone(), receiver_from_ws);
        my_net.initialize_from_file("input.toml");

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

    /*// UI thread
    let ui_thread = thread::spawn(move || {
        ui::start_ui(my_net.simulation_controller);
    });*/


    // Keep main thread alive
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
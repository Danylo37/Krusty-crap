use std::thread;
use crossbeam_channel::unbounded;
use crate::network_initializer;
use crate::ui;
fn main() {
    let (tx, rx) = unbounded();
    let (sender_from_ws, receiver_from_ws) = unbounded();

    // Initialize the network
    let mut my_net = network_initializer::NetworkInitializer::new(tx.clone(), receiver_from_ws);
    my_net.initialize_from_file("input.toml");
    // Spawn UI thread
    thread::spawn(move || {
        eprintln!("UI thread is running");
        ui::start_ui(my_net.simulation_controller);
    });
}
use std::io::{self, BufRead};
use wg_2024::network::NodeId;
use crate::simulation_controller::SimulationController;
use crate::general_use::{ClientCommand, ClientType, ServerType};

/// Main UI function that runs in a blocking loop
pub fn start_ui(mut controller: SimulationController) {
    let stdin = io::stdin();

    loop {
        println!(
            "Choose an option:\n\
             1. Use clients\n\
             2. Crash a drone\n\
             3. Exit"
        );

        let user_choice = ask_input_user(&stdin);

        match user_choice {
            1 => use_clients(&mut controller, &stdin),
            2 => crash_drone(&mut controller, &stdin),
            3 => break,
            _ => println!("Invalid option, please try again"),
        }
    }
}

/// Gets validated user input
fn ask_input_user(stdin: &io::Stdin) -> usize {
    loop {
        match take_user_input_and_parse(stdin) {
            Ok(n) => return n,
            Err(_) => println!("Invalid input, please try again"),
        }
    }
}

/// Reads and parses user input
fn take_user_input_and_parse(stdin: &io::Stdin) -> Result<usize, std::num::ParseIntError> {
    let mut input = String::new();
    stdin.lock().read_line(&mut input).expect("Failed to read input");
    input.trim().parse()
}

/// Handles drone crashing
fn crash_drone(controller: &mut SimulationController, stdin: &io::Stdin) {
    println!("Enter the ID of the drone to crash:");

    let mut input = String::new();
    stdin.lock().read_line(&mut input).expect("Failed to read input");

    let drone_id: NodeId = match input.trim().parse() {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid drone ID");
            return;
        }
    };

    match controller.request_drone_crash(drone_id) {
        Ok(()) => println!("Crash command sent to drone {}", drone_id),
        Err(e) => println!("Error: {}", e),
    }
}

/// Handles client selection
fn use_clients(controller: &mut SimulationController, stdin: &io::Stdin) {
    println!("\nAvailable Clients:");

    let clients = controller.get_list_clients();
    if clients.is_empty() {
        println!("No clients registered.");
        return;
    }

    for (i, (client_type, client_id)) in clients.iter().enumerate() {
        println!("{}. {} Client (ID: {})", i+1, client_type, client_id);
    }

    let choice = ask_input_user(stdin);
    if let Some((client_type, client_id)) = clients.get(choice-1) {
        choose_server(*client_type, *client_id, controller, stdin);
    } else {
        println!("Invalid selection");
    }
}

/// Server selection logic
fn choose_server(
    client_type: ClientType,
    client_id: NodeId,
    controller: &mut SimulationController,
    stdin: &io::Stdin
) {
    println!("\nRequesting known servers for client {}...", client_id);

    if let Err(e) = controller.request_known_servers(client_id) {
        eprintln!("Error: {}", e);
        return;
    }

    loop {
        let servers = controller.get_list_servers();

        if servers.is_empty() {
            println!("No servers found. Press Enter to discover.");
            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();
            controller.start_flooding_on_client(client_id).unwrap();
            continue;
        }

        println!("\nAvailable Servers:");
        for (i, (server_type, server_id)) in servers.iter().enumerate() {
            println!("{}. {} Server (ID: {})", i+1, server_type, server_id);
        }
        println!("0. Back to client selection");

        let choice = ask_input_user(stdin);
        if choice == 0 {
            return;
        }

        if let Some((server_type, server_id)) = servers.get(choice-1) {
            match server_type {
                ServerType::Communication => ask_comm_server(client_id, *server_id, controller),
                ServerType::Text => handle_text_server(client_id, &servers, controller),
                ServerType::Media => handle_media_server(client_id, &servers, controller),
                _ => println!("Unsupported server type"),
            }
        } else {
            println!("Invalid selection");
        }
    }
}

/// Text server handling
fn handle_text_server(
    client_id: NodeId,
    servers: &[(ServerType, NodeId)],
    controller: &mut SimulationController
) {
    println!("\nAvailable Text Servers:");
    for (i, (_, server_id)) in servers.iter().enumerate() {
        println!("{}. Server ID: {}", i+1, server_id);
    }

    // Implementation for text server commands...
}

/// Media server handling
fn handle_media_server(
    client_id: NodeId,
    servers: &[(ServerType, NodeId)],
    controller: &mut SimulationController
) {
    println!("\nAvailable Media Servers:");
    for (i, (_, server_id)) in servers.iter().enumerate() {
        println!("{}. Server ID: {}", i+1, server_id);
    }

    // Implementation for media server commands...
}

/// Communication server handling
fn ask_comm_server(client_id: NodeId, server_id: NodeId, controller: &mut SimulationController) {
    println!("Communication server actions for client {} -> server {}", client_id, server_id);
    // Implementation for communication server...
}
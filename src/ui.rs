use tokio::io::{self, AsyncBufReadExt, BufReader};
use wg_2024::network::NodeId;
use crate::simulation_controller::SimulationController;
use crate::general_use::{ClientCommand, ClientType, ServerType};

/// Main UI function that runs in an asynchronous loop.
pub async fn start_ui(mut controller: SimulationController) {
    loop {
        // Choosing base options
        println!(
            "Choose an option
1. Use clients
2. Crashing a drone
3. Nothing"
        );

        let user_choice = ask_input_user().await;

        match user_choice {
            1 => use_clients(&mut controller).await,
            2 => crash_drone(&mut controller).await,
            3 => break, // Exit the loop
            _ => println!("Not a valid option, choose again"),
        }
    }
}

/// Asks the user for input and returns the parsed value.
async fn ask_input_user() -> usize {
    loop {
        let user_input = take_user_input_and_parse().await;
        if user_input != usize::MAX {
            // usize::MAX is the error value
            return user_input;
        }
    }
}

/// Reads user input asynchronously and parses it into a usize.
async fn take_user_input_and_parse() -> usize {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();

    reader.read_line(&mut input).await.expect("Failed to read input");
    input.trim().parse().unwrap_or_else(|e| {
        println!("Error in parse: {} \n Try again \n", e);
        usize::MAX
    })
}

/// Handles the "Crash a drone" option.
async fn crash_drone(controller: &mut SimulationController) {
    println!("Enter the ID of the drone to crash:");

    let mut input = String::new();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    reader.read_line(&mut input).await.expect("Failed to read input");

    let drone_id: NodeId = match input.trim().parse() {
        Ok(id) => id,
        Err(_) => {
            println!("Invalid input. Please enter a valid drone ID.");
            return; // Or handle the error differently (e.g., loop until valid input)
        }
    };

    match controller.request_drone_crash(drone_id) {
        Ok(()) => println!("Crash command sent to drone {}", drone_id),
        Err(e) => println!("Error: {}", e), // Display the specific error returned by request_drone_crash
    }
}

/// Handles the "Use clients" option.
async fn use_clients(controller: &mut SimulationController) {
    println!("\nAvailable Clients:\n");

    let clients_with_types = controller.get_list_clients();
    if clients_with_types.is_empty() {
        println!("No clients registered.");
        return;
    }

    let mut client_options = clients_with_types.clone(); // Clone to sort
    client_options.sort_by_key(|(_, id)| *id);

    for (i, (client_type, client_id)) in client_options.iter().enumerate() {
        println!("{}. {} Client with Node ID {}", i + 1, client_type, client_id); // Display type
    }

    let user_choice = ask_input_user().await;

    if let Some((client_type, client_id)) = client_options.get(user_choice - 1) {
        choose_server(*client_type, *client_id, controller).await; // Choose server right after selecting the client
    } else {
        println!("Invalid client choice.");
    }
}

/// Handles server selection for a specific client.
async fn choose_server(
    client_type: ClientType,
    client_id: NodeId,
    controller: &mut SimulationController,
) {
    println!("\nRequesting known servers for client {}...", client_id);

    // Request the list of known servers from the client
    if let Err(e) = controller.request_known_servers(client_id) {
        eprintln!("Error requesting known servers: {}", e);
        return;
    }

    loop {
        // Loop for server selection
        println!("\nChoose action for {} Client {}:", client_type, client_id);

        let servers_with_types = controller.get_list_servers();

        if servers_with_types.is_empty() {
            println!("No servers found. Press Enter to discover.");

            let mut input = String::new();
            let stdin = io::stdin();
            let mut reader = BufReader::new(stdin);
            reader.read_line(&mut input).await.expect("Failed to read input");

            controller
                .start_flooding_on_client(client_id)
                .expect("panic message");
            continue;
        }

        for (i, (server_type, server_id)) in servers_with_types.iter().enumerate() {
            println!("{}. {} Server with ID {}", i + 1, server_type, server_id);
        }
        println!("\nChoose a server (or 0 to go back):");

        let user_choice = ask_input_user().await;

        if let Some((server_type, server_id)) = servers_with_types.get(user_choice - 1) {
            match *server_type {
                ServerType::Communication => ask_comm_server(client_id, *server_id, controller).await,
                ServerType::Text => request_text_from_server(client_id, &servers_with_types, controller).await,
                ServerType::Media => request_media_from_server(client_id, &servers_with_types, controller).await,
                _ => println!("Cannot send request to undefined server!"),
            }
        } else if user_choice == 0 {
            // Go back option
            return; // Go back to client selection
        } else {
            println!("Invalid server choice.");
        }
    }
}

/// Handles text server requests.
async fn request_text_from_server(
    client_id: NodeId,
    server_list: &Vec<(ServerType, NodeId)>,
    controller: &mut SimulationController,
) {
    for (i, &(_, server_id)) in server_list.iter().enumerate() {
        println!("{}. Text server with ID {}", i + 1, server_id);
    }

    println!("\nChoose a server:");
    let user_choice = ask_input_user().await;

    if let Some(&(_, server_id)) = server_list.get(user_choice - 1) {
        if let Some((client_sender, _)) = controller.command_senders_clients.get(&client_id) {
            if let Err(e) = client_sender.send(ClientCommand::RequestText(server_id)) {
                eprintln!("Failed to send RequestText command: {:?}", e);
            }
        }
    }
}

/// Handles media server requests.
async fn request_media_from_server(
    client_id: NodeId,
    server_list: &Vec<(ServerType, NodeId)>,
    controller: &mut SimulationController,
) {
    for (i, &(_, server_id)) in server_list.iter().enumerate() {
        println!("{}. Media server with ID {}", i + 1, server_id);
    }

    println!("\nChoose a server:");
    let user_choice = ask_input_user().await;

    if let Some(&(_, server_id)) = server_list.get(user_choice - 1) {
        if let Some((client_sender, _)) = controller.command_senders_clients.get(&client_id) {
            if let Err(e) = client_sender.send(ClientCommand::RequestMedia(server_id)) {
                eprintln!("Failed to send RequestMedia command: {:?}", e);
            }
        }
    }
}

/// Placeholder for communication server actions.
async fn ask_comm_server(
    client_id_chose: NodeId,
    sever_id_chose: NodeId,
    controller: &mut SimulationController,
) {
    // TODO: Implement communication server actions
    println!("Communication server actions not yet implemented.");
}
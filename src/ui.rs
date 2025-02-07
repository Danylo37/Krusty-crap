use std::io::{self, BufRead};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use wg_2024::network::NodeId;
use crate::simulation_controller::SimulationController;
use crate::general_use::{ClientCommand, ClientType, ServerType, Query};

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
    loop {
        println!("\nChoose action for {} Client {}:", client_type, client_id);

        let server_options = match controller.request_known_servers(client_id) {
            Ok(servers) => servers,
            Err(e) => {
                eprintln!("Error getting known servers: {}. Starting discovery...", e);
                match controller.start_flooding_on_client(client_id) {                    //Start flooding
                    Ok(_) => println!("Discovery process has started, please wait until it finishes..."),
                    Err(err) => { println!("Error during flooding: {:?}", err); return; }    //Handle error
                }
                wait_for_discovery(controller, client_id, stdin);  // Wait for flood responses and UI input
                match controller.request_known_servers(client_id) {                            //Call again for available servers
                    Ok(servers) => servers,
                    Err(err) => {                                                            //If no servers found return to prev menu
                        eprintln!("Couldn't get servers from client after flooding: {:?}", err);
                        return;
                    },
                }
            }
        };

        if server_options.is_empty() {
            println!("No servers found. Press Enter to discover.");
            return;
            // let mut input = String::new();
            // stdin.lock().read_line(&mut input).unwrap();
            // controller.start_flooding_on_client(client_id).unwrap();
            // continue;
        }

        match client_type {
            ClientType::Chat => {  // Chat client actions
                println!("1. Start Flooding");
                println!("2. Ask Server Something");
                println!("3. Go Back");

                let choice = ask_input_user(stdin);
                match choice {
                    1 => {
                        if let Err(e) = controller.start_flooding_on_client(client_id) {
                            eprintln!("Error starting flooding: {}", e);
                        }
                    }
                    2 => {
                        loop {
                            println!("\nAvailable Servers:");
                            for (i, (server_type, server_id)) in server_options.iter().enumerate() {
                                println!("{}. {} Server with ID {}", i + 1, server_type, server_id);
                            }
                            println!("0. Go back");

                            let server_choice = ask_input_user(stdin);

                            if server_choice == 0 {
                                break; // Exit the inner loop to choose another client action
                            }
                            if let Some((server_type, server_id)) = server_options.get(server_choice - 1) {  // Server interactions (moved from ask_server_action)

                                match *server_type {
                                    ServerType::Communication => ask_comm_server(client_id, *server_id, controller, stdin),
                                    ServerType::Text => handle_text_server(client_id, &server_options, controller, stdin),
                                    ServerType::Media => handle_media_server(client_id, &server_options, controller, stdin),

                                    _ => println!("Cannot send request to undefined server!")    //Handle error if server is undefined
                                }
                            } else {
                                println!("Invalid server choice.");        //Handle error
                            }
                        }
                    }
                    3 => return, // Go back to client selection
                    _ => println!("Invalid choice."),
                }
            },
            ClientType::Web => {
                println!("\nAvailable Servers:");  //Display server list
                for (i, (server_type, server_id)) in server_options.iter().enumerate() {
                    println!("{}. {} Server with ID {}", i + 1, server_type, server_id);
                }
                println!("\nChoose a server (or 0 to go back):");

                let choice = ask_input_user(stdin);
                if choice == 0 {        // Go back option
                    return;
                }
                if let Some((server_type, server_id)) = server_options.get(choice - 1) {  //Handle Server Selection
                    match server_type {
                        ServerType::Communication => ask_comm_server(client_id, *server_id, controller, stdin),
                        ServerType::Text => handle_text_server(client_id, &server_options, controller, stdin),
                        ServerType::Media => handle_media_server(client_id, &server_options, controller, stdin),
                        ServerType::Undefined => {
                            if let Err(e) = controller.ask_server_type_with_client_id(client_id, *server_id) {
                                eprintln!("Error requesting server type: {}", e);
                            }
                        },
                        _ => println!("Unsupported server type"),
                    }
                } else {
                    println!("Invalid server choice.");
                }
            },

        }
    }
}

fn wait_for_discovery(controller: &mut SimulationController, client_id: NodeId, stdin: &io::Stdin) {
    let mut input = String::new();
    let mut servers: Vec<(ServerType, NodeId)> = Vec::new();    //Store servers here for now
    let mut try_counter = 0;                                        //It will count to 10, if still no servers then returns

    while servers.is_empty(){        //Check if controller received any servers
        println!("Waiting for discovery to finish. Please press Enter to proceed when ready.");
        io::stdin().read_line(&mut input).expect("Failed to read input.");
        servers = controller.get_list_servers();                //Check if controller has got any servers

        try_counter += 1;
        if(try_counter >= 10){                                  //Return to prev menu if timeout
            println!("Discovery didn't find any server. Returning to client selection.");
            return;
        }
        sleep(Duration::from_millis(100));
    }
        println!("Discovery complete. Found {} servers.", servers.len()); // Print the number of servers discovered
}

fn choose_action_client(client_type: ClientType, client_id: NodeId, server_options: Vec<(ServerType, NodeId)>, controller: &mut SimulationController, stdin: &io::Stdin) {
    loop { // Loop for actions (flooding, server interaction, or going back)
        println!("\nChoose action for {} Client {}:", client_type, client_id);

        match client_type {
            ClientType::Chat => {
                println!(
                    "1. Start Flooding\n\
                    2. Ask Server Something\n\
                    3. Go Back"
                );

                let user_choice = ask_input_user(stdin);
                match user_choice {
                    1 => { // Start Flooding
                        if let Err(e) = controller.start_flooding_on_client(client_id) {
                            eprintln!("Error starting flooding: {}", e);
                        }
                    }
                    2 => { // Ask Server Something (now handles server selection and actions)
                        loop { // Loop for server selection and actions
                            println!("\nAvailable Servers:");

                            for (i, (server_type, server_id)) in server_options.iter().enumerate() {
                                println!("{}. {} Server with ID {}", i + 1, server_type, server_id);
                            }
                            println!("0. Go back");

                            let server_choice = ask_input_user(stdin);  // Get user input for server selection
                            if server_choice == 0 {
                                break; //Go back to client actions menu
                            }
                            if let Some((server_type, server_id)) = server_options.get(server_choice - 1) { //Server interactions
                                match *server_type {
                                    ServerType::Communication => ask_comm_server(client_id, *server_id, controller, stdin),
                                    ServerType::Text => handle_text_server(client_id, &server_options, controller, stdin),
                                    ServerType::Media => handle_media_server(client_id, &server_options, controller, stdin),
                                    _ => println!("Cannot send request to undefined server!")    //Handle error if server type is undefined
                                }
                            } else {
                                println!("Invalid server choice.");    //Error handling
                            }
                        }
                    }
                    3 => break, // Exit the loop and go back to client selection
                    _ => println!("Invalid choice."),
                }
            }
            ClientType::Web => { }
            }
        }
    }

/// Text server handling
fn handle_text_server(
    client_id: NodeId,
    servers: &[(ServerType, NodeId)],
    controller: &mut SimulationController,
    stdin: &io::Stdin
) {
    let text_servers: Vec<_> = servers
        .iter()
        .filter(|(server_type, _)| *server_type == ServerType::Text)
        .cloned()
        .collect();

    if text_servers.is_empty() {
        println!("No Text servers available.");
        return;
    }

    loop {
        println!("\nText Server Actions:");
        println!("1. Ask list of files");
        println!("2. Ask file from server");
        println!("3. Go back");

        let choice = ask_input_user(stdin);

        match choice {
            1 => {
                println!("Choose a text server:");
                for (i, &(_, server_id)) in text_servers.iter().enumerate() { // Print servers list
                    println!("{}. Server ID: {}", i+1, server_id);
                }

                let server_choice = ask_input_user(stdin);
                if let Some(&(_, server_id)) = text_servers.get(server_choice - 1) {
                    match controller.ask_list_files(client_id, server_id) {
                        Ok(_) => println!("Request for file list sent."),
                        Err(e) => eprintln!("Error requesting file list: {}", e),
                    }
                } else {
                    println!("Invalid server selection.");
                    continue; // Continue to the next iteration of the server selection loop
                }
                return; //Or continue the loop here
                //TODO
            }
            2 => {
                println!("Choose a text server:");
                for (i, &(_, server_id)) in text_servers.iter().enumerate() {
                    println!("{}. Server ID: {}", i+1, server_id);
                }

                let server_choice = ask_input_user(stdin);
                if let Some(&(_, server_id)) = text_servers.get(server_choice - 1) {
                    println!("\nRequest a file from server {}:", server_id);

                    if let Ok(file) = take_user_input_and_parse(stdin){

                        let query = Query::AskFile(file.to_string());

                        match controller.ask_file_from_server(client_id, server_id, query) {
                            Ok(_) => println!("File request sent."),
                            Err(e) => eprintln!("Error requesting file: {}", e),
                        }
                    } else {
                        println!("Invalid file index. Please enter a valid number.");
                        continue; //Continue the loop to let the user choose again. Use return if you need to go back after invalid input.
                        //TODO
                    }
                } else {
                    println!("Invalid server selection.");
                    continue; //Continue the loop to let the user choose again. Use return if you need to go back after invalid input.
                    //TODO
                }
                return;     //Or continue the loop here
                //TODO
            }
            3 => return,
            _ => println!("Invalid choice."),
        }
    }
}

/// Media server handling
fn handle_media_server(
    client_id: NodeId,
    servers: &[(ServerType, NodeId)],
    controller: &mut SimulationController,
    stdin: &io::Stdin
) {
    let media_servers: Vec<_> = servers
        .iter()
        .filter(|(server_type, _)| *server_type == ServerType::Media)
        .cloned()
        .collect();

    if media_servers.is_empty() {
        println!("No Media servers available.");
        return;
    }

    loop {
        println!("\nMedia Server Actions:");
        println!("1. Ask media from server");
        println!("2. Go back");

        let choice = ask_input_user(stdin);
        match choice {
            1 => {
                println!("Choose a text server:");
                for (i, &(_, server_id)) in media_servers.iter().enumerate() {    //Print list of available media servers
                    println!("{}. Server ID: {}", i+1, server_id);
                }

                let server_choice = ask_input_user(stdin);
                if let Some(&(_, server_id)) = media_servers.get(server_choice-1) {

                    // Request media reference from the user
                    println!("Enter the media reference:");
                    let mut reference = String::new();
                    stdin.lock().read_line(&mut reference).expect("Failed to read reference");
                    let reference = reference.trim().to_string(); // Trim whitespace

                    let query = Query::AskMedia(reference);

                    /*match controller.ask_media_from_server(client_id, server_id, query) {
                        Ok(_) => println!("Media request sent."),
                        Err(e) => eprintln!("Error requesting media: {}", e),
                    }*/
                }else{
                    println!("Invalid server choice.");
                    continue;   //Or return here
                    //TODO
                }
                return; //Or continue loop here
                //TODO
            }
            2 => return,  //Go back option
            _ => println!("Invalid choice."),
        }
    }
}

/// Communication server handling
fn ask_comm_server(
    client_id: NodeId,
    server_id: NodeId,
    controller: &mut SimulationController,
    stdin: &io::Stdin) {
    loop {
        println!("\nCommunication Server Actions (Client {} - Server {}):", client_id, server_id);
        println!("1. Register with server");
        println!("2. Request list of registered clients");
        println!("3. Send message to client");
        println!("4. Go back");

        let choice = ask_input_user(stdin);

        match choice {
            1 => {
                match controller.register_client_on_server(client_id, server_id) {
                    Ok(_) => println!("Registration request sent."),
                    Err(e) => eprintln!("Error registering client: {}", e),
                }
            }
            2 => {
                match controller.request_clients_list(client_id, server_id) {
                    Ok(_) => println!("Request for client list sent."),
                    Err(e) => eprintln!("Error requesting client list: {}", e),
                }
            }
            3 => {
                println!("Enter the recipient client ID:");
                let recipient_id = ask_input_user(stdin) as NodeId;

                println!("Enter the message:");
                let mut message = String::new();
                stdin.lock().read_line(&mut message).expect("Failed to read message.");
                let message = message.trim().to_string();

                match controller.send_message(client_id, recipient_id, message) {
                    Ok(_) => println!("Message sent."),
                    Err(e) => eprintln!("Error sending message: {}", e),
                }
            },
            4 => return, // Go back to server selection
            _ => println!("Invalid choice."),
        }
    }
}
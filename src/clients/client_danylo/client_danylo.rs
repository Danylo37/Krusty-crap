// TODO: add sending events to the controller and add logging

use crossbeam_channel::{select_biased, Receiver, Sender};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    thread,
    time::{Duration, Instant},
};
use serde::Serialize;

use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType},
};
use crate::clients::client::Client;
use crate::general_use::{ClientCommand, ClientEvent, Message, Query, Response, ServerType};

use super::message_fragments::MessageFragments;

pub struct ChatClientDanylo {
    // ID
    pub id: NodeId,                                             // Client ID

    // Channels
    pub packet_send: HashMap<NodeId, Sender<Packet>>,           // Neighbor's packet sender channels
    pub packet_recv: Receiver<Packet>,                          // Packet receiver channel
    pub controller_send: Sender<ClientEvent>,                   // Event sender channel
    pub controller_recv: Receiver<ClientCommand>,               // Command receiver channel

    // Servers and users
    pub servers: Vec<(NodeId, ServerType)>,                     // IDs and types of the available servers
    pub is_registered: HashMap<NodeId, bool>,                   // Registration status on servers
    pub users: Vec<NodeId>,                                     // Available users

    // Used IDs
    pub session_ids: Vec<u64>,                                  // Used session IDs
    pub flood_ids: Vec<u64>,                                    // Used flood IDs

    // Network
    pub topology: HashMap<NodeId, HashSet<NodeId>>,             // Nodes and their neighbours
    pub routes: HashMap<NodeId, Vec<NodeId>>,                   // Routes to the servers

    // Message queues
    pub messages_to_send: HashMap<u64, MessageFragments>,       // Queue of messages to be sent for different sessions
    pub fragments_to_reassemble: HashMap<u64, Vec<Fragment>>,   // Queue of fragments to be reassembled for different sessions

    // Inbox
    pub inbox: Vec<(NodeId, Message)>,                          // Messages with their senders
    pub new_messages: usize,                                    // Count of new messages

    // Response statuses
    pub response_received: bool,                                // Status of the last sent query
    pub last_response_time: Option<Instant>,                    // Time of the last response
}

impl Client for ChatClientDanylo {
    fn new(
        id: NodeId,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        packet_recv: Receiver<Packet>,
        controller_send: Sender<ClientEvent>,
        controller_recv: Receiver<ClientCommand>,
    ) -> Self {
        Self {
            id,
            packet_send,
            packet_recv,
            controller_send,
            controller_recv,
            servers: Vec::new(),
            is_registered: HashMap::new(),
            users: Vec::new(),
            session_ids: Vec::new(),
            flood_ids: Vec::new(),
            topology: HashMap::new(),
            routes: HashMap::new(),
            messages_to_send: HashMap::new(),
            fragments_to_reassemble: HashMap::new(),
            inbox: Vec::new(),
            new_messages: 0,
            response_received: false,
            last_response_time: None,
        }
    }

    fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        self.handle_command(command);
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                    }
                },
            }
        }
    }
}

impl ChatClientDanylo {
    /// ###### Handles incoming packets and dispatches them to the appropriate handler based on the packet type.
    ///
    /// ###### Arguments
    /// * `packet` - The incoming packet to be processed.
    fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type.clone() {
            PacketType::Ack(ack) => self.handle_ack(ack.fragment_index, packet.session_id),
            PacketType::Nack(nack) => self.handle_nack(nack, packet.session_id),
            PacketType::MsgFragment(fragment) => {
                self.send_ack(fragment.fragment_index, packet.session_id, packet.routing_header.clone());
                let server_id = packet.routing_header.hops.last().unwrap();
                self.handle_fragment(fragment, packet.session_id, *server_id)
            }
            PacketType::FloodRequest(flood_request) => self.handle_flood_request(flood_request, packet.session_id),
            PacketType::FloodResponse(flood_response) => {
                let initiator = flood_response.path_trace.first().unwrap().0;
                if initiator != self.id {
                    self.send_to_next_hop(packet);
                } else {
                    self.handle_flood_response(flood_response);
                }
            },
        }
    }

    /// ###### Handles a client command by performing the appropriate action based on the command type.
    ///
    /// ###### Arguments
    ///
    /// * `command` - The `ClientCommand` to be processed. It can be one of the following:
    ///   - `AddSender(id, sender)`: Adds a sender to the `packet_send` map with the given `id`.
    ///   - `RemoveSender(id)`: Removes a sender associated with the given `id` from the `packet_send` map.
    ///   - `AskTypeTo(server_id)`: Requests the type of the specified server using `server_id`.
    fn handle_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::AddSender(id, sender) => {
                self.packet_send.insert(id, sender);
            }
            ClientCommand::RemoveSender(id) => {
                self.packet_send.remove(&id);
            }
            ClientCommand::AskTypeTo(server_id) => {
                self.request_server_type(server_id);
            }
            ClientCommand::StartFlooding => {
                self.discovery();
            }
        }
    }

    /// todo
    fn handle_ack(&mut self, fragment_index: u64, session_id: u64) {
        let message= self.messages_to_send.get_mut(&session_id).unwrap();

        if let Some(next_fragment) = message.get_fragment_packet(fragment_index as usize) {
            // Prepare and send the next fragment if available.
            message.increment_last_index();
            self.send_to_next_hop(next_fragment);
        } else {
            // All fragments are acknowledged; remove session
            self.messages_to_send.remove(&session_id);

            println!("Message sent successfully!");
            self.response_received = true;
        }
    }

    /// todo
    fn handle_nack(&mut self, nack: Nack, session_id: u64) {
        match nack.nack_type {
            NackType::ErrorInRouting(id) => {
                println!("Error: ErrorInRouting; drone doesn't have neighbor with id {}", id);
                self.response_received = true;
            }
            NackType::DestinationIsDrone => {
                println!("Error: DestinationIsDrone");
                self.response_received = true;
            }
            NackType::UnexpectedRecipient(recipient_id) => {
                println!("Error: UnexpectedRecipient (node with id {})", recipient_id);
                self.response_received = true;
            }
            // Resend the last packet.
            NackType::Dropped => self.resend_last_packet(session_id),
        }
    }

    /// ###### Resends the last packet of a session to the next hop.
    ///
    /// This method retrieves the last fragment packet of the specified session and sends it to the next hop.
    ///
    /// ###### Arguments
    /// * `session_id` - The ID of the session whose last packet should be resent.
    fn resend_last_packet(&mut self, session_id: u64) {
        let message = self.messages_to_send.get(&session_id).unwrap();
        let packet = message.get_fragment_packet(message.get_last_fragment_index()).unwrap();
        self.send_to_next_hop(packet);
    }

    /// ###### Handles an incoming message fragment, storing it and reassembling the message if all fragments are received.
    ///
    /// This method adds the fragment to the collection of fragments for the specified session.
    /// If the fragment is the last one, the message is reassembled and processed.
    ///
    /// ###### Arguments
    /// * `fragment` - The incoming fragment to be processed.
    /// * `session_id` - The ID of the session associated with the fragment.
    fn handle_fragment(&mut self, fragment: Fragment, session_id: u64, server_id: NodeId) {
        // Retrieve or create a vector to store fragments for the session.
        let fragments = self.fragments_to_reassemble.entry(session_id).or_insert_with(Vec::new);

        // Add the current fragment to the collection.
        fragments.push(fragment.clone());

        // Check if the current fragment is the last one in the sequence.
        if fragment.fragment_index == fragment.total_n_fragments - 1 {
            // Reassemble the fragments into a complete message and process it.
            let message = self.reassemble(session_id);
            self.handle_server_response(message, server_id);
        }
    }

    /// TODO
    fn handle_server_response(&mut self, response: Option<Response>, server_id: NodeId) {
        if let Some(response) = response {
            match response {
                Response::ServerType(server_type) => {
                    self.handle_server_type(server_id, server_type);
                },
                Response::UserAdded => {
                    self.handle_client_added(server_id);
                }
                Response::ListUsers(users) => {
                    self.handle_users_list(users);
                }
                Response::MessageFrom(from, message) => {
                    self.inbox.insert(0, (from, message));
                    self.new_messages += 1;
                }
                Response::Err(error) => {
                    eprintln!("Occurred an error: {}", error);
                    self.response_received = true;
                }
                _ => {}
            }
        }
    }

    /// todo
    fn handle_server_type(&mut self, server_id: NodeId, server_type: ServerType) {
        println!("Server type is: {}", &server_type);

        self.servers.push((server_id, server_type));

        if server_type == ServerType::Communication {
            self.is_registered.insert(server_id, false);
        }

        self.response_received = true;
    }

    /// todo
    fn handle_client_added(&mut self, server_id: NodeId) {
        println!("You have registered successfully!");
        self.is_registered.insert(server_id, true);
        self.response_received = true;
    }

    /// todo
    fn handle_users_list(&mut self, users: Vec<NodeId>) {
        println!("Users list:");
        for user_id in &users {
            println!("User {}", user_id);
        }
        self.users = users;
        self.response_received = true;
    }

    /// TODO
    pub fn wait_response(&mut self) {
        let timeout = Duration::from_secs(5);
        let start_time = Instant::now();

        while !self.response_received {
            if start_time.elapsed() > timeout {
                eprintln!("Timeout waiting for server response");
                return;
            }
            thread::sleep(Duration::from_millis(100));
        }
        self.response_received = false;
    }

    /// ###### Sends an Ack packet for a received fragment.
    ///
    /// This method creates an Ack packet for the specified fragment and session,
    /// using the provided routing header, and sends it to the next hop.
    ///
    /// ###### Arguments
    /// * `fragment_index` - The index of the fragment being acknowledged.
    /// * `session_id` - The ID of the session associated with the fragment.
    /// * `routing_header` - The routing information required to send the ACK packet.
    fn send_ack(&mut self, fragment_index: u64, session_id: u64, routing_header: SourceRoutingHeader) {
        let ack = Packet::new_ack(routing_header, session_id, fragment_index);
        self.send_to_next_hop(ack)
    }

    /// todo
    fn handle_flood_request(&mut self, mut flood_request: FloodRequest, session_id: u64) {
        // Add client to the flood request's path trace.
        flood_request.increment(self.id, NodeType::Client);

        // Generate and send the flood response
        let response = flood_request.generate_response(session_id);
        self.send_to_next_hop(response);
    }

    /// ###### Sends a packet to the next hop in the route.
    ///
    /// This method retrieves the sender for the next hop, increments the hop index in the packet's routing header,
    /// and attempts to send the packet. If the sender is not found or the send operation fails, an error is logged.
    ///
    /// ###### Arguments
    /// * `packet` - The packet to be sent to the next hop.
    fn send_to_next_hop(&mut self, mut packet: Packet) {
        // Attempt to find the sender for the next hop.
        let Some(sender) = self.get_sender_of_next(packet.routing_header.clone()) else {
            eprintln!("There is no sender to the next hop.");
            self.response_received = true;
            return;
        };

        // Increment the hop index in the routing header to reflect progress through the route.
        packet.routing_header.increase_hop_index();

        // Attempt to send the updated fragment packet to the next hop.
        if sender.send(packet).is_err() {
            eprintln!("Error sending the packet to next hop.");
            self.response_received = true;
        }
    }

    /// ###### Retrieves the sender for the next hop in the routing header.
    ///
    /// This method verifies that the client is the expected recipient of the packet and retrieves
    /// the sender associated with the next hop in the routing header. If any required information is missing
    /// or the client is not the intended recipient, `None` is returned.
    ///
    /// ###### Arguments
    /// * `routing_header` - The source routing header containing hop information.
    ///
    /// ###### Returns
    /// * `Option<&Sender<Packet>>` - A reference to the sender for the next hop, or `None` if unavailable.
    fn get_sender_of_next(&self, routing_header: SourceRoutingHeader) -> Option<&Sender<Packet>> {
        // Attempt to retrieve the current hop ID from the routing header.
        // If it is missing, return `None` as we cannot proceed without it.
        let Some(current_hop_id) = routing_header.current_hop() else {
            return None;
        };

        // Check if the current hop ID matches the client's ID.
        // If it doesn't match, return `None` because the client is not the expected recipient.
        if self.id != current_hop_id {
            return None;
        }

        // Attempt to retrieve the next hop ID from the routing header.
        // If it is missing, return `None` as there is no valid destination to send the packet to.
        let Some(next_hop_id) = routing_header.next_hop() else {
            return None;
        };

        // Use the next hop ID to look up the associated sender in the `packet_send` map.
        // Return a reference to the sender if it exists, or `None` if not found.
        self.packet_send.get(&next_hop_id)
    }

    /// Handles a flood response by updating routes, servers, and topology.
    fn handle_flood_response(&mut self, flood_response: FloodResponse) {
        let path = &flood_response.path_trace;

        self.update_routes_and_servers(path);
        self.update_topology(path);

        // todo
        self.last_response_time = Some(Instant::now());
    }

    /// Updates the routing table and server information based on the path trace.
    ///
    /// - Adds the server to the `servers` map if the last node in the path is a server.
    /// - Updates the route if the new path is shorter than the existing one.
    fn update_routes_and_servers(&mut self, path: &[(NodeId, NodeType)]) {
        if let Some((id, NodeType::Server)) = path.last() {
            if self
                .routes
                .get(id)
                .map_or(true, |prev_path| prev_path.len() > path.len())
            {
                // Add the server to the servers list with an undefined type.
                self.servers.push((*id, ServerType::Undefined));

                // Update the routing table with the new, shorter path.
                self.routes.insert(
                    *id,
                    path.iter().map(|entry| entry.0.clone()).collect(),
                );
            }
        }
    }

    /// Updates the topology graph by adding connections between nodes in the path trace.
    ///
    /// - Adds bidirectional connections between each pair of consecutive nodes.
    fn update_topology(&mut self, path: &[(NodeId, NodeType)]) {
        for i in 0..path.len() - 1 {
            let current = path[i].0;
            let next = path[i + 1].0;

            // Add the connection between the current and next node in both directions.
            self.topology
                .entry(current)
                .or_insert_with(HashSet::new)
                .insert(next);

            self.topology
                .entry(next)
                .or_insert_with(HashSet::new)
                .insert(current);
        }
    }

    /// Initiates the discovery process by clearing current state, generating new identifiers,
    /// and broadcasting a flood request to all neighbors.
    pub fn discovery(&mut self) {
        // Clear all current data structures related to users, routes, servers, and topology.
        self.users.clear();
        self.routes.clear();
        self.servers.clear();
        self.topology.clear();

        // Generate a new flood ID, incrementing the last one or starting at 1 if none exists.
        let flood_id = self.flood_ids.last().map_or(1, |last| last + 1);
        self.flood_ids.push(flood_id);

        // Create a new flood request initialized with the generated flood ID, the current node's ID, and its type.
        let flood_request = FloodRequest::initialize(
            flood_id,
            self.id,
            NodeType::Client,
        );

        // Generate a new session ID, incrementing the last one or starting at 1 if none exists.
        let session_id = self.session_ids.last().map_or(1, |last| last + 1);
        self.session_ids.push(session_id);

        // Create a new packet with the flood request and session ID.
        let packet = Packet::new_flood_request(
            SourceRoutingHeader::empty_route(),
            session_id,
            flood_request,
        );

        // Attempt to send the flood request to all neighbors.
        for sender in &self.packet_send {
            if let Err(_) = sender.1.send(packet.clone()) {
                eprintln!("Failed to send FloodRequest to the drone with id {}.", sender.0);
                return;
            }
        }

        // todo
        self.last_response_time = Some(Instant::now());
        self.wait_discovery_end(Duration::from_secs(1));
    }

    /// TODO
    pub fn wait_discovery_end(&mut self, timeout: Duration) {
        while let Some(last_response) = self.last_response_time {
            if Instant::now().duration_since(last_response) > timeout {
                println!("Discovery complete!");
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    /// ###### Sends a request to a server asking for its type.
    pub fn request_server_type(&mut self, server_id: NodeId) {
        self.create_and_send_message(Query::AskType, server_id);
    }

    /// ###### Sends a request to register the current client to the specified server.
    pub fn request_to_register(&mut self, server_id: NodeId) {
        self.create_and_send_message(Query::AddUser(self.id), server_id);
    }

    /// ###### Requests the server to provide a list of all users.
    pub fn request_users_list(&mut self, server_id: NodeId) {
        self.create_and_send_message(Query::AskListUsers, server_id);
    }

    /// ###### Sends a message to a specific client through the server.
    pub fn send_message_to(&mut self, to: NodeId, message: Message, server_id: NodeId) {
        self.create_and_send_message(Query::SendMessageTo(to, message), server_id);
    }

    /// ###### Creates and sends a serialized message to a specified server.
    ///
    /// This method finds or creates a route to the server, generates a new session ID, splits the message into fragments,
    /// and sends the first fragment to the next hop. If the route to the server is not available, an error is logged.
    ///
    /// ###### Arguments
    /// * `data` - The data to be serialized and sent as the message.
    /// * `server_id` - The ID of the server to which the message will be sent.
    fn create_and_send_message<T: Serialize>(&mut self, data: T, server_id: NodeId) {
        // Find or create a route.
        let hops = if let Some(route) = self.routes.get(&server_id) {
            route.clone()
        } else {
            eprintln!("No routes to the server with id {}", server_id);
            self.response_received = true;
            return;
        };

        // Generate a new session ID.
        let session_id = self.session_ids.last().map_or(1, |last| last + 1);
        self.session_ids.push(session_id);

        // Create message (split the message into fragments) and send first fragment.
        let mut message = MessageFragments::new(session_id, hops);
        if message.create_message_of(data) {
            self.send_to_next_hop(message.get_fragment_packet(0).unwrap());
        } else {
            eprintln!("Failed to create message.");
            self.response_received = true;
        }
    }

    #[deprecated]
    /// ###### Finds a route from the current node to the specified server using breadth-first search.
    ///
    /// This method explores the network topology starting from the current node, and returns the shortest path
    /// (in terms of hops) to the specified server if one exists. It uses a queue to explore nodes level by level,
    /// ensuring that the first valid path found is the shortest. If no path is found, it returns `None`.
    ///
    /// ###### Arguments
    /// * `server_id` - The ID of the server to which the route is being sought.
    ///
    /// ###### Returns
    /// * `Option<Vec<NodeId>>` - An optional vector representing the path from the current node to the server.
    /// If no route is found, `None` is returned.
    fn find_route_to(&self, server_id: NodeId) -> Option<Vec<NodeId>> {
        // Initialize a queue for breadth-first search and a set to track visited nodes.
        let mut queue: VecDeque<(NodeId, Vec<NodeId>)> = VecDeque::new();
        let mut visited: HashSet<NodeId> = HashSet::new();

        // Start from the current node with an initial path containing just the current node.
        queue.push_back((self.id, vec![self.id]));

        // Perform breadth-first search.
        while let Some((current, path)) = queue.pop_front() {
            // If the destination node is reached, return the path.
            if current == server_id {
                return Some(path);
            }

            // Mark the current node as visited.
            visited.insert(current);

            // Explore the neighbors of the current node.
            if let Some(neighbors) = self.topology.get(&current) {
                for &neighbor in neighbors {
                    // Only visit unvisited neighbors.
                    if !visited.contains(&neighbor) {
                        let mut new_path = path.clone();
                        new_path.push(neighbor); // Extend the path to include the neighbor.
                        queue.push_back((neighbor, new_path)); // Add the neighbor to the queue.
                    }
                }
            }
        }
        None    // Return None if no path to the server is found.
    }

    /// ###### Reassembles the fragments of a message for the given session ID and attempts to deserialize the data.
    ///
    /// This method retrieves the fragments for the specified session, checks that the number of fragments matches
    /// the expected total, and combines the fragments into a single string. The string is then deserialized into
    /// an object (using JSON). If any errors occur during these steps, an error message is logged and `None` is returned.
    ///
    /// ###### Arguments
    /// * `session_id` - The ID of the session whose fragments are to be reassembled.
    ///
    /// ###### Returns
    /// * `Option<String>` - The deserialized message as a string if successful, or `None` if any error occurs.
    fn reassemble(&mut self, session_id: u64) -> Option<Response> {
        // Retrieve the fragments for the given session.
        let fragments = match self.fragments_to_reassemble.get_mut(&session_id) {
            Some(fragments) => fragments,
            None => {
                eprintln!("No fragments found for session {}", session_id);
                return None;
            },
        };

        // Ensure all fragments belong to the same message by checking the total number of fragments.
        let total_n_fragments = match fragments.first() {
            Some(first) => first.total_n_fragments,
            None => {
                eprintln!("Fragment list is empty for session {}", session_id);
                return None;
            },
        };

        // Check if the number of fragments matches the expected total.
        if fragments.len() as u64 != total_n_fragments {
            eprintln!(
                "Incorrect number of fragments for session {}: expected {}, got {}",
                session_id,
                total_n_fragments,
                fragments.len()
            );
            return None;
        }

        // Collect data from all fragments.
        let mut result = Vec::new();
        for fragment in fragments {
            result.extend_from_slice(&fragment.data[..fragment.length as usize]);
        }

        // Convert the collected data into a string.
        let reassembled_string = match String::from_utf8(result) {
            Ok(string) => string,
            Err(err) => {
                eprintln!(
                    "Failed to convert data to string for session {}: {}",
                    session_id, err
                );
                return None;
            },
        };

        // Attempt to deserialize the string into an object.
        match serde_json::from_str(&reassembled_string) {
            Ok(deserialized) => Some(deserialized),
            Err(err) => {
                eprintln!(
                    "Failed to deserialize JSON for session {}: {}",
                    session_id, err
                );
                None
            },
        }
    }
}

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};

use crossbeam_channel::{select_biased, Receiver, Sender};
use log::{info, debug, warn, error};
use serde::Serialize;

use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType},
};

use crate::{
    general_use::{
        ClientCommand, ClientEvent, Message, Query, Response, ServerType,
        ClientId, ServerId, SessionId, FloodId, FragmentIndex
    },
    clients::Client
};
use super::MessageFragments;

pub type Node = (NodeId, NodeType);

pub struct ChatClientDanylo {
    // ID
    pub id: ClientId,                                                 // Client ID

    // Channels
    pub packet_send: HashMap<NodeId, Sender<Packet>>,                 // Neighbor's packet sender channels
    pub packet_recv: Receiver<Packet>,                                // Packet receiver channel
    pub controller_send: Sender<ClientEvent>,                         // Event sender channel
    pub controller_recv: Receiver<ClientCommand>,                     // Command receiver channel

    // Servers and clients
    pub servers: HashMap<ServerId, ServerType>,                       // IDs and types of the available servers
    pub is_registered: HashMap<ServerId, bool>,                       // Registration status on servers
    pub clients: HashMap<ServerId, Vec<ClientId>>,                    // Available clients on different servers

    // Used IDs
    pub session_ids: Vec<SessionId>,                                  // Used session IDs
    pub flood_ids: Vec<FloodId>,                                      // Used flood IDs

    // Network
    pub topology: HashMap<NodeId, HashSet<NodeId>>,                   // Nodes and their neighbours
    pub routes: HashMap<ServerId, Vec<NodeId>>,                       // Routes to the servers

    // Message queues
    pub messages_to_send: HashMap<SessionId, MessageFragments>,       // Queue of messages to be sent for different sessions
    pub fragments_to_reassemble: HashMap<SessionId, Vec<Fragment>>,   // Queue of fragments to be reassembled for different sessions

    // Inbox
    pub inbox: Vec<(ClientId, Message)>,                              // Messages with their senders
}

impl Client for ChatClientDanylo {
    fn new(
        id: NodeId,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        packet_recv: Receiver<Packet>,
        controller_send: Sender<ClientEvent>,
        controller_recv: Receiver<ClientCommand>,
    ) -> Self {
        info!("Starting ChatClientDanylo with ID: {}", id);
        Self {
            id,
            packet_send,
            packet_recv,
            controller_send,
            controller_recv,
            servers: HashMap::new(),
            is_registered: HashMap::new(),
            clients: HashMap::new(),
            session_ids: Vec::new(),
            flood_ids: Vec::new(),
            topology: HashMap::new(),
            routes: HashMap::new(),
            messages_to_send: HashMap::new(),
            fragments_to_reassemble: HashMap::new(),
            inbox: Vec::new(),
        }
    }

    fn run(&mut self) {
        info!("Running ChatClientDanylo with ID: {}", self.id);
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
    /// ###### Handles incoming packets and delegates them to the appropriate handler based on the packet type.
    pub(crate) fn handle_packet(&mut self, packet: Packet) {
        debug!("Handling packet: {:?}", packet);

        match packet.pack_type.clone() {
            PacketType::Ack(ack) => self.handle_ack(ack.fragment_index, packet.session_id),
            PacketType::Nack(nack) => self.handle_nack(nack, packet.session_id),
            PacketType::MsgFragment(fragment) => {
                // Send acknowledgment for the received fragment
                self.send_ack(fragment.fragment_index, packet.session_id, packet.routing_header.clone());

                // Get the server ID from the routing header and handle the fragment
                let server_id = packet.routing_header.hops.first().unwrap();
                self.handle_fragment(fragment, packet.session_id, *server_id)
            }
            PacketType::FloodRequest(flood_request) => self.handle_flood_request(flood_request, packet.session_id),
            PacketType::FloodResponse(flood_response) => self.handle_flood_response(flood_response),
        }
    }

    /// ###### Handles incoming commands.
    pub(crate) fn handle_command(&mut self, command: ClientCommand) {
        debug!("Handling command: {:?}", command);

        match command {
            ClientCommand::AddSender(id, sender) => {
                self.packet_send.insert(id, sender);
                info!("Added sender for node {}", id);
            }
            ClientCommand::RemoveSender(id) => {
                self.packet_send.remove(&id);
                info!("Removed sender for node {}", id);
            }
            ClientCommand::ShortcutPacket(packet) => {
                info!("Shortcut packet received from SC: {:?}", packet);
                self.handle_packet(packet);
            }
            ClientCommand::GetKnownServers => {
                debug!("Handling GetKnownServers command");
                let servers: Vec<(ServerId, ServerType, bool)> = self
                    .servers
                    .iter()
                    .map(|(&id, &server_type)| (
                        id,
                        server_type,
                        *self.is_registered.get(&id).unwrap_or(&false)))
                    .collect();
                self.send_event(ClientEvent::KnownServers(servers));
            }
            ClientCommand::StartFlooding => {
                match self.discovery() {
                    Ok(_) => info!("Discovery process started successfully"),
                    Err(err) => error!("Failed to start discovery process: {}", err),
                };
            }
            ClientCommand::AskTypeTo(server_id) => {
                match self.request_server_type(server_id) {
                    Ok(_) => info!("Server type request sent successfully"),
                    Err(err) => error!("Failed to request server type: {}", err),
                };
            }
            _ => {}
        }
    }

    /// ###### Handles the acknowledgment (ACK) for a given session and fragment.
    /// Processes the acknowledgment for a specific fragment in a session.
    /// If there are more fragments to send, it sends the next fragment.
    /// If all fragments are acknowledged, it removes the message from queue.
    fn handle_ack(&mut self, fragment_index: FragmentIndex, session_id: SessionId) {
        debug!("Handling ACK for session {} and fragment {}", session_id, fragment_index);

        // Retrieve the message fragments for the given session.
        let message = self.messages_to_send.get_mut(&session_id).unwrap();

        // Check if there is a next fragment to send.
        if let Some(next_fragment) = message.get_fragment_packet((fragment_index + 1) as usize) {
            // Prepare and send the next fragment if available.
            message.increment_last_index();
            match self.send_to_next_hop(next_fragment) {
                Ok(_) => info!("Sent next fragment for session {}", session_id),
                Err(err) => error!("Failed to send next fragment for session {}: {}", session_id, err),
            }
        } else {
            // All fragments are acknowledged; remove the message from queue.
            self.messages_to_send.remove(&session_id);
            info!("All fragments acknowledged for session {}", session_id);
        }
    }

    /// ###### Handles the negative acknowledgment (NACK) for a given session.
    /// Processes the NACK for a specific session and takes appropriate action based on the NACK type.
    fn handle_nack(&mut self, nack: Nack, session_id: SessionId) {
        warn!("Handling NACK for session {}: {:?}", session_id, nack);

        match nack.nack_type {
            NackType::ErrorInRouting(id) => {
                self.handle_error_in_routing(nack.fragment_index, id, session_id);
            }
            NackType::DestinationIsDrone => {
                // todo
            }
            NackType::UnexpectedRecipient(_recipient_id) => {
                // todo
            }
            NackType::Dropped => self.resend_fragment(nack.fragment_index, session_id),
        }
    }

    /// ###### Handles an error in routing.
    /// Updates the network topology and routes based on the error node.
    /// If a new route is found, it resends the fragment for the specified session.
    /// Else, it starts the discovery process to find a new route.
    fn handle_error_in_routing(&mut self,fragment_index: FragmentIndex, error_node: NodeId, session_id: SessionId) {
        self.update_topology_and_routes(error_node, &session_id);
        if self.messages_to_send.get(&session_id).unwrap().get_route().is_empty() {
            if self.discovery().is_err() {
                error!("Failed to find a new route after error in routing");
                return;
            };
        }
        self.resend_fragment(fragment_index, session_id);
    }

    /// ###### Updates the network topology and routes based on the received NACK.
    /// Removes the node that caused the error from the topology and routes.
    /// Finds new routes for the servers that need them.
    fn update_topology_and_routes(&mut self, error_node: NodeId, session_id: &SessionId) {
        // Remove the node that caused the error from the topology.
        for (_, neighbors) in self.topology.iter_mut() {
            neighbors.remove(&error_node);
        }
        self.topology.remove(&error_node);
        info!("Removed node {} from the topology", error_node);

        // Remove the routes that contain the node that caused the error.
        self.routes.retain(|_, path| !path.contains(&error_node));
        info!("Removed node {} from the routes", error_node);

        // Collect server IDs that need new routes.
        let servers_to_update: Vec<ServerId> = self
            .routes
            .iter()
            .filter(|(_, path)| path.is_empty())
            .map(|(server_id, _)| *server_id)
            .collect();

        // Find new routes for the collected server IDs.
        for server_id in servers_to_update {
            if let Some(new_path) = self.find_route_to(server_id) {
                if let Some(path) = self.routes.get_mut(&server_id) {
                    *path = new_path;
                    info!("Found new route to server {}: {:?}", server_id, path);
                }
            } else {
                warn!("No route found to server {}", server_id);
            }
        }

        let message = self.messages_to_send.get_mut(session_id).unwrap();
        let prev_route = message.get_route();

        // Check if the previous route in message to send contains the error node and update the route if necessary.
        if prev_route.contains(&error_node) {
            let dest_id = prev_route.last().unwrap();
            let new_route = self.routes.get(dest_id);

            match new_route {
                Some(route) => {
                    message.update_route(route.clone())
                }
                None => {
                    message.update_route(Vec::new())
                }
            }
        }
    }

    /// ###### Finds a route from the current node to the specified server using breadth-first search.
    ///
    /// This method explores the network topology starting from the current node, and returns the shortest path
    /// (in terms of hops) to the specified server if one exists. It uses a queue to explore nodes level by level,
    /// ensuring that the first valid path found is the shortest. If no path is found, it returns `None`.
    fn find_route_to(&self, server_id: ServerId) -> Option<Vec<NodeId>> {
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

    /// ###### Resends the fragment for the specified session.
    /// Retrieves the message and resends the fragment with the specified index.
    fn resend_fragment(&mut self, fragment_index: FragmentIndex, session_id: SessionId) {
        debug!("Resending fragment {} for session {}", fragment_index, session_id);

        let message = self.messages_to_send.get(&session_id).unwrap();
        let packet = message.get_fragment_packet(fragment_index as usize).unwrap();
        match self.send_to_next_hop(packet) {
            Ok(_) =>
                info!("Resent fragment {} for session {}", fragment_index, session_id),
            Err(err) =>
                error!("Failed to resend fragment {} for session {}: {}", fragment_index, session_id, err),
        }
    }

    /// ###### Handles a received message fragment.
    /// Adds the fragment to the collection for the session and checks if it is the last fragment.
    /// If it is the last fragment, reassembles the message and processes the server response.
    fn handle_fragment(&mut self, fragment: Fragment, session_id: SessionId, server_id: ServerId) {
        debug!("Handling fragment for session {}: {:?}", session_id, fragment);

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

    /// ###### Handles the server response.
    /// Processes the server response based on its type and takes appropriate actions.
    fn handle_server_response(&mut self, response: Option<Response>, server_id: ServerId) {
        debug!("Handling server response for server {}: {:?}", server_id, response);

        if let Some(response) = response {
            match response {
                Response::ServerType(server_type) => {
                    self.handle_server_type(server_id, server_type);
                },
                Response::ClientRegistered => {
                    self.handle_client_registered(server_id);
                }
                Response::ListClients(clients) => {
                    self.handle_clients_list(server_id, clients);
                }
                Response::MessageFrom(from, message) => {
                    info!("New message from {}: {:?}", from, &message);

                    self.inbox.insert(0, (from, message));
                }
                Response::Err(error) =>
                    error!("Error received from server {}: {:?}", server_id, error),
                _ => {}
            }
        }
    }

    /// ###### Handles the server type response.
    /// Updates the server type in the `servers` map and sets the registration status if the server is of type `Communication`
    /// and marks the response as received.
    fn handle_server_type(&mut self, server_id: ServerId, server_type: ServerType) {
        info!("Server type received successfully.");

        // Insert the server type into the servers map.
        self.servers.insert(server_id, server_type);

        // If the server is of type Communication, set the registration status to false.
        if server_type == ServerType::Communication {
            self.is_registered.insert(server_id, false);
        }
    }

    /// ###### Handles the client registration response.
    /// Updates the registration status for the specified server and marks the response as received.
    fn handle_client_registered(&mut self, server_id: ServerId) {
        info!("Client registered successfully.");

        self.is_registered.insert(server_id, true);
    }

    /// ###### Handles the list of clients received from the server.
    /// Updates the list of available clients and marks the response as received.
    fn handle_clients_list(&mut self, server_id: ServerId, clients: Vec<ClientId>) {
        info!("List of clients received successfully.");

        self.clients.insert(server_id, clients);
    }

    /// ###### Sends an acknowledgment (ACK) for a received fragment.
    /// Creates an ACK packet and sends it to the next hop.
    /// Logs the success or failure of the send operation.
    fn send_ack(&mut self, fragment_index: FragmentIndex, session_id: SessionId, mut routing_header: SourceRoutingHeader) {
        // Reverse the routing header and reset the hop index.
        routing_header.reverse();
        routing_header.reset_hop_index();

        let ack = Packet::new_ack(routing_header, session_id, fragment_index);

        // Attempt to send the ACK packet to the next hop.
        match self.send_to_next_hop(ack) {
            Ok(_) => {
                info!("ACK sent successfully for session {} and fragment {}", session_id, fragment_index);
            }
            Err(err) => {
                error!("Failed to send ACK for session {} and fragment {}: {}", session_id, fragment_index, err);
            }
        };
    }

    /// ###### Handles a flood request by adding the client to the path trace and generating a response.
    fn handle_flood_request(&mut self, mut flood_request: FloodRequest, session_id: SessionId) {
        debug!("Handling flood request for session {}: {:?}", session_id, flood_request);

        // Add client to the flood request's path trace.
        flood_request.increment(self.id, NodeType::Client);

        // Generate a response for the flood request.
        let response = flood_request.generate_response(session_id);

        // Send the response to the next hop.
        match self.send_to_next_hop(response) {
            Ok(_) => info!("FloodResponse sent successfully."),
            Err(err) => error!("Error sending FloodResponse: {}", err),
        }
    }

    /// ###### Sends the packet to the next hop in the route.
    ///
    /// Attempts to send the packet to the next hop in the route.
    /// If the packet is successfully sent, it returns `Ok(())`.
    /// If an error occurs during the send operation, it returns an error message.
    fn send_to_next_hop(&mut self, mut packet: Packet) -> Result<(), String> {
        // Attempt to retrieve the next hop ID from the routing header.
        // If it is missing, return an error as there is no valid destination to send the packet to.
        let Some(next_hop_id) = packet.routing_header.next_hop() else {
            return Err("No next hop in the routing header.".to_string());
        };

        // Attempt to find the sender for the next hop.
        let Some(sender) = self.packet_send.get(&next_hop_id) else {
            return Err("No sender to the next hop.".to_string());
        };

        // Increment the hop index in the routing header.
        packet.routing_header.increase_hop_index();

        debug!("Sending packet to next hop: {:?}", packet);
        // Attempt to send the packet to the next hop.
        if sender.send(packet.clone()).is_err() {
            return Err("Error sending packet to next hop.".to_string());
        } else {
            info!("Packet sent to next hop: {}", next_hop_id);
        }

        // Send the 'PacketSent' event to the simulation controller
        self.send_event(ClientEvent::PacketSent(packet));

        Ok(())
    }

    /// ###### Handles the flood response by updating routes and topology.
    ///
    /// This function processes the received `FloodResponse` by updating the routes and servers
    /// based on the path trace provided in the response. It also updates the network topology
    /// with the new path information and updates the time of the last response.
    fn handle_flood_response(&mut self, flood_response: FloodResponse) {
        debug!("Handling flood response: {:?}", flood_response);

        let path = &flood_response.path_trace;

        self.update_routes_and_servers(path);
        self.update_topology(path);

    }

    /// ###### Updates the routes and servers based on the provided path.
    /// If the path leads to a server, it updates the routing table and the servers list.
    fn update_routes_and_servers(&mut self, path: &[Node]) {
        if let Some((id, NodeType::Server)) = path.last() {
            if self
                .routes
                .get(id)
                .map_or(true, |prev_path| prev_path.len() > path.len())
            {
                // Add the server to the servers list with an undefined type if it is not already present.
                if !self.servers.contains_key(id) {
                    self.servers.insert(*id, ServerType::Undefined);
                }

                // Update the routing table with the new, shorter path.
                self.routes.insert(
                    *id,
                    path.iter().map(|entry| entry.0.clone()).collect(),
                );
                info!("Updated route to server {}: {:?}", id, path);
            }
        }
    }

    /// ###### Updates the network topology based on the provided path.
    /// Adds connections between nodes in both directions.
    fn update_topology(&mut self, path: &[Node]) {
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
        debug!("Updated topology with path: {:?}", path);
    }

    /// ###### Initiates the discovery process to find available servers and clients.
    /// Clears current data structures and sends a flood request to all neighbors.
    pub fn discovery(&mut self) -> Result<(), String> {
        info!("Starting discovery process");

        // Clear all current data structures related to topology.
        self.routes.clear();
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

        let mut error_drones = Vec::new();

        // Attempt to send the flood request to all neighbors.
        for sender in &self.packet_send {
            if let Err(_) = sender.1.send(packet.clone()) {
                error!("Failed to send FloodRequest to the drone {}.", sender.0);

                // Add the drone ID to the list of failed drones.
                error_drones.push(sender.0);
            } else {
                info!("FloodRequest sent to the drone with id {}.", sender.0);

                // Send the 'PacketSent' event to the simulation controller.
                self.send_event(ClientEvent::PacketSent(packet.clone()));
            }
        }

        let mut error_string = String::new();

        // Check number of errors and create an error message if necessary.
        if error_drones.len() == 1 {
            error_string = format!("Failed to send FloodRequest to drone {}", error_drones[0]);
        }
        if error_drones.len() > 1 {
            let formatted_drone_ids = error_drones.iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            error_string = format!("Failed to send FloodRequests to drones: {}", formatted_drone_ids);
        }

        if error_string.is_empty() {
            Ok(())
        } else {
            Err(error_string)
        }
    }

    /// ###### Requests the type of specified server.
    /// Sends a query to the server and waits for a response.
    pub fn request_server_type(&mut self, server_id: ServerId) -> Result<(), String> {
        info!("Requesting server type for server {}", server_id);

        let result = self.create_and_send_message(Query::AskType, server_id);

        match result {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                error!("Failed to receive server type: {}", err);
                Err(err)
            },
        }
    }

    /// ###### Requests to register the client on a specified server.
    /// Sends a registration query to the server and waits for a response.
    pub fn request_to_register(&mut self, server_id: ServerId) -> Result<(), String> {
        info!("Requesting to register on server {}", server_id);

        let result = self.create_and_send_message(Query::RegisterClient(self.id), server_id);

        match result {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                error!("Failed to register client: {}", err);
                Err(err)
            },
        }
    }

    /// ###### Requests the list of clients from a specified server.
    /// Sends a query to the server and waits for a response.
    pub fn request_clients_list(&mut self, server_id: ServerId) -> Result<(), String> {
        info!("Requesting clients list from server {}", server_id);

        let result = self.create_and_send_message(Query::AskListClients, server_id);

        match result {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                error!("Failed to get list of clients: {}", err);
                Err(err)
            },
        }
    }

    /// ###### Sends a message to a specified client via a specified server.
    /// Sends the message and waits for a response.
    pub fn send_message_to(&mut self, to: NodeId, message: Message, server_id: ServerId) -> Result<(), String> {
        info!("Sending message to client {} via server {}", to, server_id);

        let result = self.create_and_send_message(Query::SendMessageTo(to, message), server_id);

        match result {
            Ok(_) => {
                info!("Message sent successfully.");
                Ok(())
            }
            Err(err) => {
                error!("Failed to send message: {}", err);
                Err(err)
            },
        }
    }

    /// ###### Creates and sends a message to a specified server.
    /// Serializes the data, splits it into fragments, and sends the first fragment.
    fn create_and_send_message<T: Serialize + Debug>(&mut self, data: T, server_id: ServerId) -> Result<(), String> {
        debug!("Creating and sending message to server {}: {:?}", server_id, data);

        // Find or create a route.
        let hops = if let Some(route) = self.routes.get(&server_id) {
            route.clone()
        } else {
            return Err(format!("No routes to the server with id {server_id}"));
        };

        // Generate a new session ID.
        let session_id = self.session_ids.last().map_or(1, |last| last + 1);
        self.session_ids.push(session_id);

        // Create message (split the message into fragments) and send first fragment.
        let mut message = MessageFragments::new(session_id, hops);
        if message.create_message_of(data) {
            self.messages_to_send.insert(session_id, message.clone());
            self.send_to_next_hop(message.get_fragment_packet(0).unwrap())
        } else {
            Err("Failed to create message.".to_string())
        }
    }

    /// ###### Reassembles the fragments for a given session into a complete message.
    /// Returns the reassembled message or an error if reassembly fails.
    fn reassemble(&mut self, session_id: SessionId) -> Option<Response> {
        debug!("Reassembling message for session {}", session_id);

        // Retrieve the fragments for the given session.
        let fragments = match self.fragments_to_reassemble.get_mut(&session_id) {
            Some(fragments) => fragments,
            None => {
                error!("No fragments found for session {}", session_id);
                return None;
            },
        };

        // Ensure all fragments belong to the same message by checking the total number of fragments.
        let total_n_fragments = match fragments.first() {
            Some(first) => first.total_n_fragments,
            None => {
                error!("Fragment list is empty for session {}", session_id);
                return None;
            },
        };

        // Check if the number of fragments matches the expected total.
        if fragments.len() as u64 != total_n_fragments {
            error!(
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
                error!(
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
                error!(
                    "Failed to deserialize JSON for session {}: {}",
                    session_id, err
                );
                None
            },
        }
    }

    /// ###### Sends an event to the simulation controller.
    /// Logs the success or failure of the send operation.
    fn send_event(&self, event: ClientEvent) {
        let result = self.controller_send.send(event.clone());
        let event_name = match event {
            ClientEvent::PacketSent(_) => "PacketSent",
            ClientEvent::KnownServers(_) => "KnownServers",
        };

        match result {
            Ok(_) => info!("Sent '{}' event to controller", event_name),
            Err(_) => error!("Error sending '{}' event to controller", event_name),
        }
    }
}

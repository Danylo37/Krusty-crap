//I am a god

use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::{HashMap, HashSet, VecDeque};
use log::{debug, error, info};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{
        Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType,
    },
};
use crate::general_use::{FloodId, Message, Query, Response, ServerCommand, ServerEvent, ServerType};


///SERVER TRAIT
pub trait Server{
    fn get_id(&self) -> NodeId;
    fn get_server_type(&self) -> ServerType;

    fn get_session_id(&mut self) -> u64;
    fn get_flood_id(&mut self) -> u64;

    fn push_flood_id(&mut self, flood_id: FloodId);
    fn get_clients(&mut self) -> &mut HashSet<NodeId>;
    fn get_topology(&mut self) -> &mut HashMap<NodeId, HashSet<NodeId>>;
    fn get_nodes(&mut self) -> &mut HashMap<NodeId, NodeType>;
    fn get_routes(&mut self) -> &mut HashMap<NodeId, Vec<NodeId>>;

    fn get_event_sender(&self) -> &Sender<ServerEvent>;
    fn get_from_controller_command(&mut self) -> &mut Receiver<ServerCommand>;
    fn get_packet_recv(&mut self) -> &mut Receiver<Packet>;
    fn get_packet_send(&mut self) -> &mut HashMap<NodeId, Sender<Packet>>;
    fn get_packet_send_not_mutable(&self) -> &HashMap<NodeId, Sender<Packet>>;

    fn get_reassembling_messages(&mut self) -> &mut HashMap<u64, Vec<u8>>;
    fn process_query(&mut self, query: Query, src_id: NodeId);
    fn get_sending_messages(&mut self) -> &mut HashMap<u64, (Vec<u8>, u8)>;
    fn get_sending_messages_not_mutable(&self) -> &HashMap<u64, (Vec<u8>, u8)>;

    fn get_drops_counter(&mut self) -> &mut HashMap<u64, HashMap<NodeId, u8>>;

    fn get_queries_to_process(&mut self) -> &mut VecDeque<(NodeId, Query)>;

    fn run(&mut self) {
        info!("Running {} server with ID: {}", self.get_server_type(), self.get_id());
        loop {
            select_biased! {
                recv(self.get_from_controller_command()) -> command_res => {
                    if let Ok(command) = command_res {
                        info!("Server {}: Received command: {:?}", self.get_id(), command);
                        self.handle_command(command);
                    }
                },
                recv(self.get_packet_recv()) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        info!("Server {}: Received packet: {:?}", self.get_id(), packet);
                        self.handle_packet(packet)
                    }
                },
            }
        }
    }

    fn handle_command(&mut self, command: ServerCommand) {
        match command {
            ServerCommand::AddSender(id, sender) => {
                self.get_packet_send().insert(id, sender);
                info!("Server {}: Added sender for node {}", self.get_id(), id);
            }
            ServerCommand::StartFlooding => {
                self.discover();
            }
            ServerCommand::RemoveSender(id) => {
                self.get_packet_send().remove(&id);
                self.update_topology_and_routes(id);
                info!("Server {}: Removed sender for node {}", self.get_id(), id);
            }
            ServerCommand::ShortcutPacket(packet) => {
                info!("Server {}: Shortcut packet received from SC: {:?}", self.get_id(), packet);
                self.handle_packet(packet);
            }
            _ => {},
        }
    }

    fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::Nack(nack) => self.handle_nack(nack, packet.session_id, packet.routing_header.hops[0]),
            PacketType::Ack(ack) => self.handle_ack(ack),
            PacketType::MsgFragment(fragment) => self.handle_fragment(fragment, packet.routing_header, packet.session_id),
            PacketType::FloodRequest(flood_request) => self.handle_flood_request(flood_request, packet.session_id),
            PacketType::FloodResponse(flood_response) => self.handle_flood_response(flood_response),
        }
    }

    //FLOOD
    fn discover(&mut self) {
        info!("Server {}: Starting discovery process", self.get_id());

        self.get_routes().clear();
        self.get_topology().clear();

        let flood_id = self.generate_unique_flood_id();
        self.push_flood_id(flood_id);

        // Create a new flood request initialized with the generated flood ID, the current node's ID, and its type.
        let flood_request = FloodRequest::initialize(
            flood_id,
            self.get_id(),
            NodeType::Server,
        );

        let session_id = self.generate_unique_session_id();

        // Create a new packet with the flood request and session ID.
        let packet = Packet::new_flood_request(
            SourceRoutingHeader::empty_route(),
            session_id,
            flood_request,
        );

        // Attempt to send the flood request to all neighbors.
        for (_, sender_channel) in self.get_packet_send_not_mutable() {
            sender_channel.send(packet.clone()).expect("This error message shouldn't come out, (flooding error from server)");
        }
    }

    fn handle_flood_request(&mut self, mut flood_request: FloodRequest, session_id: u64) {
        debug!("Server {}: Handling flood request: {:?}", self.get_id(), flood_request);

        //Inserting self in flood request
        flood_request.increment(self.get_id(), NodeType::Server);

        //Creating and sending flood response
        let mut response = flood_request.generate_response(session_id);
        response.routing_header.increase_hop_index();
        self.send_packet(response.clone());
    }

    fn handle_flood_response(&mut self, flood_response: FloodResponse) {
        info!("Server {}: Handling flood response: {:?}", self.get_id(), flood_response);

        let path = &flood_response.path_trace;

        self.update_routes_to_clients(path);
        self.update_topology(path);
    }

    fn update_routes_to_clients(&mut self, path: &[(NodeId, NodeType)]) {
        if let Some((id, NodeType::Client)) = path.last() {
            if self
                .get_routes()
                .get(id)
                .map_or(true, |prev_path| prev_path.len() > path.len())
            {
                self.get_clients().insert(*id);

                // Update the routing table with the new, shorter path.
                self.get_routes().insert(
                    *id,
                    path.iter().map(|entry| entry.0.clone()).collect(),
                );
                info!("Server {}: Updated route to client {}: {:?}", self.get_id(), id, path);

                // Resend responses that were waiting for the route to the client.
                if !self.get_queries_to_process().is_empty() && self.get_queries_to_process().front().unwrap().0 == *id {
                    self.reprocess_query();
                }
            }
        }
    }

    fn reprocess_query(&mut self) {
        let queries = self.get_queries_to_process().clone();

        for (client_id, query) in queries {

            if !self.get_routes().contains_key(&client_id) {
                return;
            }

            self.process_query(query, client_id);
            self.get_queries_to_process().pop_front();
        }
    }

    fn update_topology(&mut self, path: &[(NodeId, NodeType)]) {
        info!("Server {}: Updating topology with path: {:?}", self.get_id(), path);
        for i in 0..path.len() - 1 {
            let current_id = path[i].0;
            let current_type = path[i].1;
            let next_id = path[i + 1].0;

            // Add the connection between the current and next node in both directions.
            self.get_topology()
                .entry(current_id)
                .or_insert_with(HashSet::new)
                .insert(next_id);

            self.get_topology()
                .entry(next_id)
                .or_insert_with(HashSet::new)
                .insert(current_id);

            self.get_nodes().insert(current_id, current_type);

        }
    }

    //NACK
    fn handle_nack(&mut self, nack: Nack, session_id: u64, last_node_id: NodeId){
        debug!("Server {}: Handling NACK for session {}: {:?}", self.get_id(), session_id, nack);

        match nack.nack_type {
            NackType::UnexpectedRecipient(_) => {
                self.send_again_fragment(session_id, nack.fragment_index);
            },
            NackType::Dropped => {
                self.handle_nack_dropped(session_id, nack.fragment_index, last_node_id);
            },
            NackType::DestinationIsDrone => {
                self.send_again_fragment(session_id, nack.fragment_index);
            },
            NackType::ErrorInRouting(error_node) => {
                self.update_topology_and_routes(error_node);
                self.send_again_fragment(session_id, nack.fragment_index);
            }
        }
    }

    fn handle_nack_dropped(&mut self, session_id: u64, fragment_index: u64, last_node_id: NodeId) {
        let drones_and_counters = self.get_drops_counter().get_mut(&session_id).unwrap();
        let Some(counter) = drones_and_counters.get_mut(&last_node_id) else {
            drones_and_counters.insert(last_node_id, 1);

            self.send_again_fragment(session_id, fragment_index);
            return;
        };

        *counter += 1;

        // If the counter reaches 10, send an event to call technicians to fix the drone.
        if *counter == 10 {
            *counter = 0;
            let me = (self.get_id(), NodeType::Server);
            self.get_event_sender().send(ServerEvent::CallTechniciansToFixDrone(last_node_id, me)).unwrap();
            self.wait_for_drone_fix(last_node_id);
        }

        self.send_again_fragment(session_id, fragment_index);
    }

    fn wait_for_drone_fix(&mut self, last_node_id: NodeId) {
        loop {
            match self.get_from_controller_command().recv() {
                Ok(command) => {
                    match command {
                        ServerCommand::DroneFixed(node_id) => {
                            if node_id == last_node_id {
                                break;
                            }
                        }
                        _ => { self.handle_command(command) }
                    }
                }
                Err(err) => {
                    error!("Server {}: Error receiving command from the controller: {}", self.get_id(), err);
                }
            }
        }
    }

    fn send_nack(&self, nack: Nack, routing_header: SourceRoutingHeader, session_id: u64){
        let packet= Self::create_packet(PacketType::Nack(nack), routing_header, session_id);
        self.send_packet(packet);
    }

    //ACK
    fn handle_ack(&mut self, _ack: Ack){
        //UI stuff I guess?
    }

    fn send_ack(&self, ack: Ack, routing_header: SourceRoutingHeader, session_id: u64) {
        let mut packet= Self::create_packet(PacketType::Ack(ack), routing_header, session_id);
        packet.routing_header.increase_hop_index();
        self.send_packet(packet);
    }

    //PACKET
    fn create_packet(pack_type: PacketType, routing_header: SourceRoutingHeader, session_id: u64, ) -> Packet {
        Packet {
            pack_type,
            routing_header,
            session_id,
        }
    }

    fn send_packet(&self, packet: Packet) {
        let next_hop_id = packet.routing_header.hops[1];

        let Some(first_carrier) = self.get_packet_send_not_mutable().get(&next_hop_id) else {
            match packet.pack_type {
                PacketType::Ack(_) | PacketType::Nack(_) | PacketType::FloodResponse(_) => {
                    self.get_event_sender().send(ServerEvent::ControllerShortcut(packet)).unwrap();
                    return;
                }
                _ => {
                    error!("Server {}: No sender found for node {}", self.get_id(), next_hop_id);
                    return;
                }
            }
        };

        first_carrier.send(packet).unwrap();
    }

    fn find_path_to(&mut self, destination_id: NodeId) -> Option<Vec<NodeId>>{
        match self.get_routes().get(&destination_id) {
            None => {
                None
            }
            Some(route) => {
                Some(route.clone())
            }
        }
    }

    fn create_source_routing(route: Vec<NodeId>) -> SourceRoutingHeader{
        SourceRoutingHeader {
            hop_index: 1,
            hops: route,
        }
    }

    //FRAGMENT TO DECIDE IF IMPLEMENTING DEFAULT FOR EACH ONE

    fn handle_fragment(&mut self, fragment: Fragment, routing_header: SourceRoutingHeader, session_id: u64, ){
        // Packet Verification
        if self.get_id() != routing_header.hops[routing_header.hop_index] {
            // Send Nack (UnexpectedRecipient)
            let nack = Nack {
                fragment_index: fragment.fragment_index,
                nack_type: NackType::UnexpectedRecipient(self.get_id()),
            };
            self.send_nack(nack, routing_header.get_reversed(), session_id);
            return;
        }else{
            // Send Ack
            let ack = Ack {
                fragment_index: fragment.fragment_index,
            };
            self.send_ack(ack, routing_header.get_reversed(), session_id);
        }

        info!("Handling Fragment {:?}", fragment);

        //Getting vec of data from fragment
        let mut data_to_add :Vec<u8> = fragment.data.to_vec();
        data_to_add.truncate(fragment.length as usize);

        //Fragment reassembly
        // Check if it exists already
        if let Some(reassembling_message) = self.get_reassembling_messages().get_mut(&session_id) {

            // Check for valid fragment index and length
            let offset = reassembling_message.len();
            if offset + fragment.length as usize > reassembling_message.capacity()
            {
                info!("Nack");
                //Maybe cancelling also message in reassembling_message
                // ... error handling logic ...
                return;

            }else{
                // Copy data to the correct offset in the vector.
                reassembling_message.splice(((fragment.fragment_index*128) as usize)..((fragment.fragment_index*128) as usize), data_to_add);

                // Check if all fragments have been received
                let reassembling_message_clone = reassembling_message.clone();
                info!("N fragments + current fragment length{}", fragment.total_n_fragments*128 + fragment.length as u64);
                self.if_all_fragments_received_process(&reassembling_message_clone, &fragment, session_id, routing_header);
            }

        }else {
            //Check if it is only 1 fragment
            if !self.if_all_fragments_received_process(&data_to_add, &fragment, session_id, routing_header)
            {
                // New message, create a new entry in the HashMap.
                let mut reassembling_message = Vec::with_capacity(fragment.total_n_fragments as usize * 128);

                //Copying data
                reassembling_message.splice(((fragment.fragment_index*128) as usize)..((fragment.fragment_index*128) as usize), data_to_add);

                //Inserting message for future fragments
                self.get_reassembling_messages()
                    .insert(session_id, reassembling_message.clone());
            }
        }
    }

    fn if_all_fragments_received_process(&mut self, message: &Vec<u8>, current_fragment: &Fragment, session_id: u64, routing_header: SourceRoutingHeader) -> bool {
        // Message Processing

        info!("Message length {} n_fragments {} current.fragment length {}, fragment_index: {}", message.len(), current_fragment.total_n_fragments, current_fragment.length, current_fragment.fragment_index );
        if(message.len() as u64) == ((current_fragment.total_n_fragments-1)*128 + current_fragment.length as u64)
        {

            let reassembled_data = message.clone(); // Take ownership of the data
            self.get_drops_counter().remove(&session_id);
            self.get_reassembling_messages().remove(&session_id); // Remove from map
            self.process_reassembled_message(reassembled_data, routing_header.hops[0]);
            return true;
        }
        false
    }

    fn process_reassembled_message(&mut self, data: Vec<u8>, src_id: NodeId) {
        match String::from_utf8(data.clone()) {
            Ok(data_string) => match serde_json::from_str(&data_string) {
                Ok(query) => self.process_query(query, src_id),
                Err(_) => {
                    panic!("Damn, not the right struct")
                }
            },
            Err(e) => println!("Argh, {:?}", e),
        }
    }

    fn save_query_to_process(&mut self, src_id: NodeId, query: Query) {
        if self.get_queries_to_process().is_empty() && self.get_clients().is_empty() {
            self.discover();
        }

        debug!("Server {}: Query {:?} will be processed after finding route to the Client {}",
            self.get_id(), query, src_id);

        self.get_queries_to_process().push_back((src_id, query));
        return;
    }

    fn send_fragments(&mut self, session_id: u64, n_fragments: usize, response_in_vec_bytes: &[u8], header: SourceRoutingHeader) {

        //Storing the all the fragments to send
        self.get_sending_messages().insert(session_id, (response_in_vec_bytes.to_vec(), header.destination().unwrap()));

        info!("Sending fragments n_fragments: {}", n_fragments);
        //Sending
        for i in 0..n_fragments{

            info!("Sending fragment fragment: {}", i);
            //Preparing data of fragment
            let mut data:[u8;128] = [0;128];
            if(i+1)*128>response_in_vec_bytes.len(){
                data[0..(response_in_vec_bytes.len()-(i*128))].copy_from_slice(&response_in_vec_bytes[i*128..response_in_vec_bytes.len()]);

            }else{
                data.copy_from_slice(&response_in_vec_bytes[i*128..(1+i)*128]);
            }

            //Generating fragment
            let fragment = Fragment::new(
                i as u64,
                n_fragments as u64,
                data,
            );

            //Generating packet
            let packet = Self::create_packet(
                PacketType::MsgFragment(fragment),
                header.clone(),
                session_id,
            );

            self.send_packet(packet);
            self.get_drops_counter().insert(session_id, HashMap::new());
        }
    }

    fn update_topology_and_routes(&mut self, error_node: NodeId) {
        // Remove the node that caused the error from the topology.
        for (_, neighbors) in self.get_topology().iter_mut() {
            neighbors.remove(&error_node);
        }
        self.get_topology().remove(&error_node);
        info!("Server {}: Removed node {} from the topology", self.get_id(), error_node);

        // Replace the paths that contain the node that caused the error with an empty vector.
        for route in self.get_routes().values_mut() {
            if route.contains(&error_node) {
                *route = Vec::new();
            }
        }
        info!("Server {}: Routes with node {} cleared", self.get_id(), error_node);

        // Collect server IDs that need new routes.
        let clients_to_update: Vec<NodeId> = self
            .get_routes()
            .iter()
            .filter(|(_, path)| path.is_empty())
            .map(|(client_id, _)| *client_id)
            .collect();

        // Find new routes for the collected client IDs.
        for client_id in clients_to_update {
            if let Some(new_path) = self.bfs(client_id) {
                if let Some(path) = self.get_routes().get_mut(&client_id) {
                    *path = new_path;
                }
            } else {
                error!("Server {}: No route found to the client {}", self.get_id(), client_id);
            }
        }
    }

    fn bfs(&mut self, client_id: NodeId) -> Option<Vec<NodeId>> {
        // Initialize a queue for breadth-first search and a set to track visited nodes.
        let mut queue: VecDeque<(NodeId, Vec<NodeId>)> = VecDeque::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let nodes = self.get_nodes().clone();

        // Start from the current node with an initial path containing just the current node.
        queue.push_back((self.get_id(), vec![self.get_id()]));

        // Perform breadth-first search.
        while let Some((current, path)) = queue.pop_front() {
            // If the destination node is reached, return the path.
            if current == client_id {
                return Some(path);
            }

            // Mark the current node as visited.
            visited.insert(current);

            // Explore the neighbors of the current node.
            if let Some(neighbors) = self.get_topology().get(&current) {
                for neighbor in neighbors {
                    // Only visit unvisited neighbors.
                    if visited.contains(neighbor) {
                        continue;
                    }

                    // Skip servers and clients in the search.
                    if *neighbor != client_id {
                        match nodes.get(neighbor) {
                            Some(NodeType::Server) | Some(NodeType::Client) => continue,
                            _ => {},
                        }
                    }

                    let mut new_path = path.clone();
                    new_path.push(*neighbor); // Extend the path to include the neighbor.
                    queue.push_back((*neighbor, new_path)); // Add the neighbor to the queue.
                }
            }
        }
        None    // Return None if no path to the server is found.
    }

    fn send_again_fragment(&mut self, session_id: u64, fragment_index: u64){

        //Getting right message and destination id
        let message_and_destination = self.get_sending_messages_not_mutable().get(&session_id).unwrap();

        //Preparing fields for Fragment
        let length_response = message_and_destination.0.len();
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        let offset = fragment_index*128;
        let mut data:[u8;128] = [0;128];
        if (offset+128) > length_response as u64 {
            data[0..(message_and_destination.0.len() as u64 - offset) as usize].copy_from_slice(&message_and_destination.0[offset as usize..message_and_destination.0.len()]);
        }else{
            data.copy_from_slice(&message_and_destination.0[offset as usize..(offset+128) as usize]);
        }

        //Generating fragment
        let fragment = Fragment::new(
            fragment_index,
            n_fragments as u64,
            data,
        );

        //Finding route
        let recipient_id = message_and_destination.1;
        let Some(route) = self.find_path_to(recipient_id) else {
            error!("Server {}: No route found to the client {}", self.get_id(), recipient_id);
            return;
        };

        //Generating packet
        let packet = Self::create_packet(
            PacketType::MsgFragment(fragment),
            Self::create_source_routing(route),
            session_id,
        );
        self.send_packet(packet);

    }

    //Common functions
    fn give_type_back(&mut self, src_id: NodeId){

        info!("Sending back type back");

        //Get data
        let response = Response::ServerType(self.get_server_type());

        //Serializing type
        let response_as_string = serde_json::to_string(&response).unwrap();
        let response_in_vec_bytes = response_as_string.as_bytes();
        let length_response = response_in_vec_bytes.len();

        //Counting fragments
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        // Finding route
        let Some(route) = self.find_path_to(src_id) else {
            error!("Server {}: No route found to the client {}", self.get_id(), src_id);
            return;
        };

        //Generating header
        let header = Self::create_source_routing(route);

        // Generating ids
        let session_id = self.generate_unique_session_id();

        //Send fragments
        info!("Sending fragments");
        self.send_fragments(session_id, n_fragments, response_in_vec_bytes, header);
    }

    fn generate_unique_flood_id(&mut self) -> u64 {
        let counter_flood_id = self.get_flood_id();
        let id = self.get_id();
        match format!("{}{}", id, counter_flood_id).parse() {
            Ok(id) => id,
            Err(e) => panic!("{}, Not right number", e)
        }
    }

    fn generate_unique_session_id(&mut self) -> u64 {
        let counter_session_id = self.get_session_id();
        let id = self.get_id();
        match format!("{}{}", id, counter_session_id).parse() {
            Ok(id) => id,
            Err(e) => panic!("{}, Not right number", e)
        }
    }
}


///Communication Server functions
pub trait CommunicationServer {
    fn add_client(&mut self, client_id: NodeId);
    fn give_list_back(&mut self, client_id: NodeId);
    fn forward_message_to(&mut self, message: Message);
}

///Content Server functions
pub trait TextServer {
    fn give_list_back(&mut self, client_id: NodeId);
    fn give_file_back(&mut self, client_id: NodeId,  file_key: String);
}

///Media server functions
pub trait MediaServer {
    fn give_media_back(&mut self, client_id: NodeId, reference: String);
}

use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet, VecDeque};
use log::{info, warn};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{NodeType, Packet, PacketType}
};
use crate::general_use::{ClientCommand, ClientEvent, ClientType,
                         ServerCommand, ServerEvent, ServerType, ServerId,
                         Query,
                         DisplayDataWebBrowser, DisplayDataCommunicationServer, DisplayDataMediaServer,
                         DisplayDataChatClient, DisplayDataTextServer, DisplayDataDrone,
                         SpecificNodeType, DroneId};
use crate::websocket::WsCommand;
use std::collections::hash_map::Entry;
use rand::Rng;
use crate::ui_traits::SimulationControllerMonitoring;

pub struct SimulationState {
    pub nodes: HashMap<NodeId, NodeType>,
    pub topology: HashMap<NodeId, Vec<NodeId>>,
    pub packet_history: Vec<PacketInfo>,
}


#[derive(Debug, Clone)]
pub struct PacketInfo {
    pub source: NodeId,
    pub destination: NodeId,
    pub packet_type: PacketType,
    pub dropped: bool,
}

pub struct SimulationController {
    pub state: SimulationState,
    pub drone_event_sender: Sender<DroneEvent>,
    pub drone_event_receiver: Receiver<DroneEvent>,
    pub client_event_sender: Sender<ClientEvent>,
    pub client_event_receiver: Receiver<ClientEvent>,
    pub server_event_sender: Sender<ServerEvent>,
    pub server_event_receiver: Receiver<ServerEvent>,
    pub command_senders_drones: HashMap<NodeId, Sender<DroneCommand>>,
    pub command_senders_clients: HashMap<NodeId, (Sender<ClientCommand>, ClientType)>,
    pub command_senders_servers: HashMap<NodeId, (Sender<ServerCommand>, ServerType)>,
    pub packet_senders: HashMap<NodeId, Sender<Packet>>,

    fixed_drones: HashSet<NodeId>,

    //for the monitoring
    pub web_clients_data: HashMap<NodeId, DisplayDataWebBrowser>,
    pub chat_clients_data: HashMap<NodeId, DisplayDataChatClient>,
    pub comm_servers_data: HashMap<NodeId, DisplayDataCommunicationServer>,
    pub text_servers_data: HashMap<NodeId, DisplayDataTextServer>,
    pub media_servers_data: HashMap<NodeId, DisplayDataMediaServer>,
    pub drones_data: HashMap<NodeId, DisplayDataDrone>,
    pub updating_nodes: HashSet<NodeId>,
    pub ws_command_receiver: Receiver<WsCommand>,
}


impl SimulationController {
    pub fn new(
        drone_event_sender: Sender<DroneEvent>,
        drone_event_receiver: Receiver<DroneEvent>,
        client_event_sender: Sender<ClientEvent>,
        client_event_receiver: Receiver<ClientEvent>,
        server_event_sender: Sender<ServerEvent>,
        server_event_receiver: Receiver<ServerEvent>,

        //for the monitoring
        ws_command_receiver: Receiver<WsCommand>,
    ) -> Self {
        Self {
            state: SimulationState {
                nodes: HashMap::new(),
                topology: HashMap::new(),
                packet_history: Vec::new(),
            },
            command_senders_drones: HashMap::new(),
            command_senders_clients: HashMap::new(),
            command_senders_servers: HashMap::new(),
            drone_event_sender,
            drone_event_receiver,
            client_event_sender,
            client_event_receiver,
            server_event_sender,
            server_event_receiver,
            packet_senders: HashMap::new(),

            fixed_drones: HashSet::new(),

            //for the monitoring
            web_clients_data: HashMap::new(),
            chat_clients_data: HashMap::new(),
            comm_servers_data: HashMap::new(),
            text_servers_data: HashMap::new(),
            media_servers_data: HashMap::new(),
            drones_data: HashMap::new(),
            updating_nodes: HashSet::new(),
            ws_command_receiver,
        }
    }

    pub(crate) fn process_packet_sent_events(&mut self) {
        if let Ok(event) = self.drone_event_receiver.try_recv() {
            if let DroneEvent::PacketSent(packet) = event {
                self.handle_packet_sent(packet);
            }
        }
    }

    pub(crate) fn process_packet_dropped_events(&mut self) {
        if let Ok(event) = self.drone_event_receiver.try_recv() {
            if let DroneEvent::PacketDropped(packet) = event {
                self.handle_packet_dropped(packet);
            }
        }
    }

    pub(crate) fn process_controller_shortcut_events(&mut self) {
        if let Ok(event) = self.drone_event_receiver.try_recv() {  // Receive event or continue if none is available
            if let DroneEvent::ControllerShortcut(packet) = event {   // Check event type
                self.send_shortcut(packet);
            }
        }
    }

    pub(crate) fn send_shortcut(&self, packet: Packet){
        match packet.pack_type {
            PacketType::Ack(_) | PacketType::Nack(_) | PacketType::FloodResponse(_) => {
                if let Some(destination) = self.get_destination_from_packet(&packet) {  // Try to get destination

                    // Determine where to send the packet based on the destination ID and node type
                    if self.command_senders_clients.contains_key(&destination) {          //If it's client

                        if let Some((client_sender, _)) = self.command_senders_clients.get(&destination) {
                            if let Err(e) = client_sender.send(ClientCommand::ShortcutPacket(packet.clone())) {
                                warn!("Error sending to client {}: {:?}", destination, e);
                            }
                        } else {

                            warn!("No sender found for client {}", destination);
                        }
                    } else if self.command_senders_servers.contains_key(&destination) {   // If it's server
                        if let Some((server_sender, _)) = self.command_senders_servers.get(&destination) {
                            if let Err(e) = server_sender.send(ServerCommand::ShortcutPacket(packet.clone())) {
                                warn!("Error sending to server {}: {:?}", destination, e);
                            }
                        } else {
                            warn!("No sender found for server {}", destination);
                        }
                    } else {
                        warn!("Invalid destination or unknown node type: {}", destination);
                    }
                } else {
                    warn!("Could not determine destination for ControllerShortcut");
                }
            }
            _ => warn!("Unexpected packet type in ControllerShortcut: {:?}", packet.pack_type),
        }
    }

    /// Registers a drone with the simulation controller.
    pub fn register_drone(&mut self, node_id: NodeId, command_sender: Sender<DroneCommand>) {
        self.command_senders_drones.insert(node_id, command_sender);
    }

    pub fn register_server(&mut self, node_id: NodeId, command_sender: Sender<ServerCommand>, server_type: ServerType) {

        self.command_senders_servers.insert(node_id, (command_sender, server_type));
    }

    pub fn register_client(&mut self, node_id: NodeId, command_sender: Sender<ClientCommand>, client_type: ClientType) {
        self.command_senders_clients.insert(node_id, (command_sender, client_type));
    }

    pub fn register_client_on_server(&mut self, client_id: NodeId, server_id: NodeId) -> Result<(), String> {
        if let Some((client_command_sender, _)) = self.command_senders_clients.get(&client_id) {
            if let Err(e) = client_command_sender.send(ClientCommand::RegisterToServer(server_id)) {
                return Err(format!("Failed to send AskServerType command to client {}: {:?}", client_id, e));
            }
            Ok(())
        } else {
            Err(format!("Client with id {} not found", client_id))
        }
    }

    pub fn request_clients_list(&self, client_id: NodeId, server_id: NodeId) -> Result<(), String> {
        if let Some((client_sender, _)) = self.command_senders_clients.get(&client_id) {
            if client_sender.send(ClientCommand::AskListClients(server_id)).is_err() {
                return Err(format!("Failed to send command AskListClients to the client {}.", client_id));
            }
            Ok(())
        } else {
            Err(format!("Client with ID {} not found", client_id))
        }
    }

    pub fn send_message(&self, client_id: NodeId, receiver_client_id: NodeId, msg: String) -> Result<(), String> {  // Removed server_id parameter
        if let Some((client_sender, _)) = self.command_senders_clients.get(&client_id) {
            // Send the message to the client without specifying the server
            if let Err(e) = client_sender.send(ClientCommand::SendMessageTo(receiver_client_id, msg)) {  // Changed ClientCommand
                return Err(format!("Failed to send SendMessageTo command to client {}: {:?}", client_id, e));  // Error handling
            }
            Ok(()) // Return Ok on successful send
        } else {
            Err(format!("Client with ID {} not found", client_id))  // Handle the error
        }
    }

    pub fn ask_list_files(&self, client_id: NodeId, server_id: ServerId) -> Result<(), String> {
        if let Some((client_sender, _)) = self.command_senders_clients.get(&client_id) {
            if client_sender.send(ClientCommand::RequestListFile(server_id)).is_err() {
                return Err(format!("Failed to send command AskListFiles to the client {}.", client_id));
            }
            Ok(())
        } else {
            Err(format!("Client with ID {} not found", client_id))
        }
    }

    pub fn ask_file_from_server(&mut self, client_id: NodeId, server_id: NodeId, query: Query) -> Result<(), String> {
        if let Some((client_sender, _)) = self.command_senders_clients.get(&client_id) {
            if let Err(e) = client_sender.send(ClientCommand::RequestText(server_id, match query {
                Query::AskFile(file) => file.parse().unwrap(),
                _ => panic!("Wrong type of Query, supposed to be AskFile"),
            })) {
                return Err(format!("Failed to send command AskFile to client {}: {:?}", client_id, e));
            }
            Ok(())
        }else {
            Err(format!("Client with id {} not found", client_id))
        }
    }

    /*   pub fn ask_media_from_server(&mut self, client_id: NodeId, server_id: NodeId, query: Query) -> Result<(), String> {
           if let Some((client_sender, _)) = self.command_senders_clients.get(&client_id) {
               if let Err(e) = client_sender.send(ClientCommand::RequestMedia(server_id, match query{
                   Query::AskMedia(reference) => reference,
                   _ => panic!("Wrong type of Query, supposed to be AskMedia")
               })) {
                   return Err(format!("Failed to send command AskMedia to client {}: {:?}", client_id, e));
               }
               Ok(())
           }else {
               Err(format!("Client with id {} not found", client_id))
           }
       }*/

    /// Spawns a new drone.
    pub fn create_drone<T: Drone + Send + 'static>(&mut self,
                                                   drone_id: NodeId,
                                                   _event_sender: Sender<DroneEvent>,
                                                   command_receiver: Receiver<DroneCommand>,
                                                   packet_receiver: Receiver<Packet>,
                                                   connected_nodes: HashMap<NodeId, Sender<Packet>>,
                                                   pdr: f32,
    ) -> Result<T, String> {

        let drone = T::new(
            drone_id,
            self.drone_event_sender.clone(),
            command_receiver,
            packet_receiver,
            connected_nodes.clone(),
            pdr,
        );
        Ok(drone)
    }

    pub(super) fn fix_drone(&mut self, drone_id: DroneId, sender: (NodeId, NodeType)) -> Result<(DroneId, f32), ()> {
        if self.fixed_drones.contains(&drone_id) {
            info!("Drone {} is already fixed", drone_id);
            Ok((drone_id, if let Some(drone_data) = self.drones_data.get_mut(&drone_id){
                drone_data.pdr
            }else{
                0.0
            }))
        } else {
            let mut rng = rand::thread_rng();
            let new_pdr = rng.gen_range(0.0..=0.1); // Generate random PDR between 0 and 0.1
            self.set_packet_drop_rate(drone_id, new_pdr); // Update the drone's PDR
            info!("Drone {} has been fixed! New PDR: {}", drone_id, new_pdr);

            self.fixed_drones.insert(drone_id);
            if let Some(drone_data) = self.drones_data.get_mut(&drone_id) {
                drone_data.pdr = new_pdr;
            }
            match sender.1 {
                NodeType::Client => {
                    if let Some((client_command_sender, _)) = self.command_senders_clients.get(&sender.0) {
                        if let Err(e) = client_command_sender.send(ClientCommand::DroneFixed(drone_id)) {
                            warn!("Failed to send DroneFixed to client {}: {:?}", sender.0, e);
                        }
                    }
                    Ok((drone_id, new_pdr))
                },
                NodeType::Server => {
                    if let Some((server_command_sender, _)) = self.command_senders_servers.get(&sender.0) {
                        if let Err(e) = server_command_sender.send(ServerCommand::DroneFixed(drone_id)) {
                            warn!("Failed to send DroneFixed to server {}: {:?}", sender.0, e);
                        }
                    }
                    Ok((drone_id, new_pdr))
                },
                _ => {
                    Err(())
                }
            }
        }


    }

    pub(crate) fn create_topology_with_types(&self) -> HashMap<NodeId, (Vec<NodeId>, SpecificNodeType)> {
        let mut topology_with_types = HashMap::new();
        for (node_id, connected_nodes) in &self.state.topology {
            let node_type = if self.command_senders_clients.contains_key(node_id) {
                match self.command_senders_clients.get(node_id).unwrap().1 {
                    ClientType::Chat => SpecificNodeType::ChatClient,
                    ClientType::Web => SpecificNodeType::WebBrowser,
                }
            } else if self.command_senders_servers.contains_key(node_id) {
                match self.command_senders_servers.get(node_id).unwrap().1 {
                    ServerType::Communication => SpecificNodeType::CommunicationServer,
                    ServerType::Media => SpecificNodeType::MediaServer,
                    ServerType::Text => SpecificNodeType::TextServer,
                    _ => SpecificNodeType::Drone, // Or handle other server types
                }
            } else {
                SpecificNodeType::Drone
            };
            topology_with_types.insert(*node_id, (connected_nodes.clone(), node_type));
        }
        topology_with_types
    }

    fn get_source_from_packet(&self, packet: &Packet) -> NodeId {
        if let Some(first_hop) = packet.routing_header.hops.first() {
            return *first_hop;
        }

        match &packet.pack_type {
            PacketType::MsgFragment(_) => {
                if packet.routing_header.hop_index == 1 {
                    *packet.routing_header.hops.first().unwrap()
                } else {
                    packet.routing_header.hops[packet.routing_header.hop_index - 2]
                }
            }
            PacketType::FloodRequest(flood_req) => flood_req.initiator_id,
            PacketType::FloodResponse(flood_res) => flood_res.path_trace.last().unwrap().0,
            PacketType::Ack(_) | PacketType::Nack(_) => 255,
        }
    }


    fn get_destination_from_packet(&self, packet: &Packet) -> Option<NodeId> {
        packet.routing_header.hops.last().copied()
    }

    /// Handles `PacketSent` events, adding packet information to the history.
    fn handle_packet_sent(&mut self, packet: Packet) {
        let destination = self.get_destination_from_packet(&packet).unwrap_or(255); // Provide default if None

        self.state.packet_history.push(PacketInfo {
            source: self.get_source_from_packet(&packet),
            destination,
            packet_type: packet.pack_type.clone(),
            dropped: false,
        });
    }

    /// Handles `PacketDropped` events, adding packet information to the history.
    fn handle_packet_dropped(&mut self, packet: Packet) {
        self.state.packet_history.push(PacketInfo {
            source: self.get_source_from_packet(&packet),
            destination: self.get_destination_from_packet(&packet).unwrap_or(255), // 255 is a valid default
            packet_type: packet.pack_type.clone(),
            dropped: true,
        });
    }

    pub fn add_sender(&mut self, node_id: NodeId, node_type: NodeType, connected_node_id: NodeId, sender: Sender<Packet>) {
        match node_type {
            NodeType::Drone => {
                if let Some(command_sender) = self.command_senders_drones.get(&node_id) {

                    if let Err(e) = command_sender.send(DroneCommand::AddSender(connected_node_id, sender)) {
                        warn!("Failed to send AddSender command to drone {}: {:?}", node_id, e);
                    } else{
                        match self.drones_data.entry(node_id){
                            Entry::Occupied(mut entry) => {
                                let display_data_drone = entry.get_mut();
                                display_data_drone.connected_nodes_ids.push(connected_node_id);
                            },
                            _=> {}
                        }
                    }
                } else {
                    warn!("Drone {} not found in controller", node_id);
                }
            }
            NodeType::Client => {
                if let Some((command_sender, _)) = self.command_senders_clients.get(&node_id) {
                    if let Err(e) = command_sender.send(ClientCommand::AddSender(connected_node_id, sender.clone())) {
                        warn!("Failed to send AddSender command to client {}: {:?}", node_id, e);
                    }
                } else {
                    warn!("Client {} not found in controller", node_id);
                }
            }
            NodeType::Server => {

                if let Some((command_sender, _)) = self.command_senders_servers.get(&node_id) {
                    if let Err(e) = command_sender.send(ServerCommand::AddSender(connected_node_id, sender.clone())) {
                        warn!("Failed to send AddSender command to server {}: {:?}", node_id, e);
                    }
                } else {
                    warn!("Server {} not found in controller", node_id);
                }
            }
        }
    }

    pub fn remove_sender(&mut self, node_id: NodeId, node_type: NodeType, connected_node_id: NodeId) -> Result<(), String> {
        match node_type {
            NodeType::Drone => {
                if let Some(command_sender) = self.command_senders_drones.get(&node_id) {
                    if let Err(e) = command_sender.send(DroneCommand::RemoveSender(connected_node_id)) {  // Send command, return error if fails
                        return Err(format!("Failed to send RemoveSender command to drone {}: {:?}", node_id, e));
                    }
                    Ok(())

                } else {

                    Err(format!("Drone with ID {} not found", node_id))  // Return error if no sender
                }
            }
            NodeType::Client => {
                if let Some((command_sender, _)) = self.command_senders_clients.get(&node_id) {
                    if let Err(e) = command_sender.send(ClientCommand::RemoveSender(connected_node_id)) {
                        return Err(format!("Failed to send RemoveSender command to client {}: {:?}", node_id, e));
                    }
                    Ok(())
                } else {
                    Err(format!("Client with ID {} not found", node_id))
                }
            }
            NodeType::Server => {

                if let Some((command_sender, _)) = self.command_senders_servers.get(&node_id) {

                    if let Err(e) = command_sender.send(ServerCommand::RemoveSender(connected_node_id)) {
                        return Err(format!("Failed to send RemoveSender command to server {}: {:?}", node_id, e));
                    }
                    Ok(())
                } else {
                    Err(format!("Server with ID {} not found", node_id))
                }
            }
        }
    }

    pub fn set_packet_drop_rate(&mut self, drone_id: NodeId, pdr: f32) {
        if let Some(command_sender) = self.command_senders_drones.get(&drone_id) {
            if let Err(e) = command_sender.send(DroneCommand::SetPacketDropRate(pdr)) { // Error handling
                warn!("Failed to send SetPacketDropRate command to drone {}: {:?}", drone_id, e);
            }
        } else {
            warn!("Drone {} not found in controller", drone_id);
        }
    }

    /*- This function sends a Crash command to the specified drone_id.
It uses the command_senders map to find the appropriate sender channel.
*/
    pub fn request_drone_crash(&mut self, drone_id: NodeId, sender_to_gui: &Sender<String>) -> Result<(), String> {

        if self.is_drone_critical(drone_id) {
            if let Err(e) = sender_to_gui.send("DroneNotCrashed".to_string()) {
                warn!("Error sending crash result to WebSocket: {}", e);
            }
            return Err(format!("Cannot crash drone {}: critical for connectivity", drone_id));
        }

        let neighbors = self.state.topology.get(&drone_id).cloned(); // Get drone's neighbors

        if let Some(command_sender) = self.command_senders_drones.get(&drone_id) {
            if let Err(e) = command_sender.send(DroneCommand::Crash) {
                warn!("Failed to send Crash command to drone {}: {:?}", drone_id, e);
                return Err(format!("Failed to send Crash command to drone {}: {:?}", drone_id, e)); //Return error if send failed
            }
        } else {
            return Err(format!("Drone {} not found in controller", drone_id)); //Return error if drone not found
        }
        // Remove senders from neighbors.
        if let Some(neighbors) = neighbors {
            for neighbor in neighbors {
                if let Some(NodeType::Drone) = self.state.nodes.get(&neighbor){        //If neighbour is a drone
                    if let Err(err) = self.remove_sender(neighbor, NodeType::Drone, drone_id){  //Remove the sender
                        warn!("{}", err);
                    }
                    if let Some(connected_nodes) = self.state.topology.get_mut(&neighbor) {
                        connected_nodes.retain(|&id| id != drone_id);
                    }
                } else if let Some(NodeType::Client) = self.state.nodes.get(&neighbor){   //If neighbour is a client
                    if let Err(err) = self.remove_sender(neighbor, NodeType::Client, drone_id){
                        warn!("{}", err);
                    }
                    if let Some(connected_nodes) = self.state.topology.get_mut(&neighbor) {
                        connected_nodes.retain(|&id| id != drone_id);
                    }
                } else if let Some(NodeType::Server) = self.state.nodes.get(&neighbor){   //If neighbour is a server
                    if let Err(err) = self.remove_sender(neighbor, NodeType::Server, drone_id){
                        warn!("{}", err);
                    }
                    if let Some(connected_nodes) = self.state.topology.get_mut(&neighbor) {
                        connected_nodes.retain(|&id| id != drone_id);
                    }
                }
            }
        }
        self.state.topology.remove(&drone_id);
        self.command_senders_drones.remove(&drone_id);
        if let Err(e) = sender_to_gui.send("DroneCrashed".to_string()) {
            warn!("Error sending crash result to WebSocket: {}", e);
        }

        Ok(())
    }

    fn is_drone_critical(&self, drone_id: NodeId) -> bool {
        // Create a copy of the topology without the drone and its connections
        let mut temp_topology = self.state.topology.clone();
        temp_topology.remove(&drone_id);
        for (_, neighbors) in temp_topology.iter_mut() {
            neighbors.retain(|&n| n != drone_id);
        }

        // Iterate through all clients and check their reachability to at least one server.
        for (client_id, _) in &self.command_senders_clients {
            if !self.check_client_reachability(*client_id, &temp_topology) {
                return true; // Drone is critical if any client loses all server connections.
            }
        }
        false // Drone is not critical.
    }

    fn check_client_reachability(&self, client_id: NodeId, topology: &HashMap<NodeId, Vec<NodeId>>) -> bool{

        let mut reachable_servers = HashSet::new();    //To store all reachable servers

        //Check reachability for every server
        for (server_id, _) in &self.command_senders_servers{
            if self.is_reachable(client_id, *server_id, topology) {
                reachable_servers.insert(*server_id);
            }
        }
        !reachable_servers.is_empty()  //Returns true if there is at least one reachable server.
    }

    fn is_reachable(&self, start_node: NodeId, end_node: NodeId, topology: &HashMap<NodeId, Vec<NodeId>>) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(start_node);

        while let Some(current_node) = queue.pop_front() {
            if current_node == end_node {
                return true;
            }
            visited.insert(current_node);

            if let Some(neighbors) = topology.get(&current_node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        false
    }

    pub fn get_list_clients(&self) -> Vec<(ClientType, NodeId)> {
        self.command_senders_clients
            .iter()
            .map(|(&id, (_, client_type))| (*client_type, id))
            .collect()
    }

    pub fn get_list_servers(&self) -> Vec<(ServerType, NodeId)> {
        self.command_senders_servers
            .iter()
            .map(|(&id, (_, server_type))| (*server_type, id))
            .collect()
    }

    pub fn get_server_type(&self, node_id: NodeId) -> ServerType {
        self.command_senders_servers.get(&node_id).map(|(_, server_type)| *server_type).unwrap_or(ServerType::Undefined) // Default if not found
    }

    ///This is the function for asking the server it's type, given the id of the server
    pub fn ask_which_type(&self, client_id: NodeId, server_id: NodeId) -> Result<ServerType, String> {

        if let Some((client_command_sender, _)) = self.command_senders_clients.get(&client_id) {
            if let Err(e) = client_command_sender.send(ClientCommand::AskTypeTo(server_id)) {
                return Err(format!("Failed to send AskServerType command to client {}: {:?}", client_id, e));
            }
            return Ok(self.get_server_type(server_id)); // Return the server type
        }
        Err(format!("Client with id {} not found", client_id))
    }

    pub fn start_flooding_on_client(&self, client_id: NodeId) -> Result<(), String> {

        if let Some((client_command_sender, _)) = self.command_senders_clients.get(&client_id) {
            if let Err(e) = client_command_sender.send(ClientCommand::StartFlooding) {
                return Err(format!("Failed to send StartFlooding command to client {}: {:?}", client_id, e));
            }
            Ok(())
        } else {
            Err(format!("Client with ID {} not found", client_id))
        }
    }

    pub fn ask_server_type_with_client_id(&mut self, client_id: NodeId, server_id: NodeId) -> Result<(), String> {
        if let Some((client_command_sender, _)) = self.command_senders_clients.get(&client_id) {
            if let Err(e) = client_command_sender.send(ClientCommand::AskTypeTo(server_id)) {
                return Err(format!("Failed to send AskServerType command to client {}: {:?}", client_id, e));
            }
            Ok(())
        } else {
            Err(format!("Client with ID {} not found", client_id))
        }
    }
}
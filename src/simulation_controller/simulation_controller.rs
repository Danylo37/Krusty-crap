use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::thread::sleep;
use std::time::Duration;
use log::warn;
use serde::Serialize;
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
                         SpecificNodeType};
use crate::websocket::WsCommand;
use std::collections::hash_map::Entry;

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

// Define DisplayDataSimulationController and DroneMonitoringData structs here
#[derive(Debug, Serialize)]
pub(super) struct DisplayDataSimulationController {
    data_title: String,
    web_clients_data: HashMap<NodeId, DisplayDataWebBrowser>,
    chat_clients_data: HashMap<NodeId, DisplayDataChatClient>,
    comm_servers_data: HashMap<NodeId, DisplayDataCommunicationServer>,
    text_servers_data: HashMap<NodeId, DisplayDataTextServer>,
    media_servers_data: HashMap<NodeId, DisplayDataMediaServer>,
    drones_data: HashMap<NodeId, DisplayDataDrone>,
    topology: HashMap<NodeId, (Vec<NodeId>, SpecificNodeType)>,
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

    drones_pdr: HashMap<NodeId, f32>,

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

            drones_pdr: HashMap::new(),

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

    /// Runs the main simulation loop.
    /// This function continuously processes events, updates the GUI (not implemented), and sleeps briefly.
    pub fn run(&mut self) {  // Note: &mut self since we're modifying state directly
        loop {
            self.process_packet_sent_events();
            self.process_packet_dropped_events();
            self.process_controller_shortcut_events();
            self.process_client_events();
            self.process_server_events();
            sleep(Duration::from_millis(100));
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
                                                   event_sender: Sender<DroneEvent>,
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

    /// Processes incoming events from drones.
    /// This function handles `PacketSent`, `PacketDropped`, and `ControllerShortcut` events.
    fn process_packet_sent_events(&mut self) {
        if let Ok(event) = self.drone_event_receiver.try_recv() {
            if let DroneEvent::PacketSent(packet) = event {
                self.handle_packet_sent(packet);
            }
        }
    }

    fn process_packet_dropped_events(&mut self) {
        if let Ok(event) = self.drone_event_receiver.try_recv() {
            if let DroneEvent::PacketDropped(packet) = event {
                self.handle_packet_dropped(packet);
            }
        }
    }

    fn process_controller_shortcut_events(&mut self) {
        if let Ok(event) = self.drone_event_receiver.try_recv() {  // Receive event or continue if none is available
            if let DroneEvent::ControllerShortcut(packet) = event {   // Check event type

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
        }
    }

    fn process_client_events(&mut self){
        while let Ok(event) = self.client_event_receiver.try_recv(){
            match event {
                ClientEvent::WebClientData(id, data, data_scope) => {
                    self.web_clients_data.insert(id, data);
                },
                ClientEvent::ChatClientData(id, data, data_scope) => {
                    self.chat_clients_data.insert(id, data);
                },
                other => {
                    warn!("Unexpected client event: {:?}", other);
                }
            }
        }
    }

    fn process_server_events(&mut self){
        while let Ok(event) = self.server_event_receiver.try_recv() {
            match event{
                ServerEvent::CommunicationServerData(id, data, data_scope) => {
                    self.comm_servers_data.insert(id, data);
                },
                ServerEvent::TextServerData(id, data, data_scope) =>{
                    self.text_servers_data.insert(id, data);
                },
                ServerEvent::MediaServerData(id, data, data_scope) =>{
                    self.media_servers_data.insert(id, data);
                },
                other => {
                    warn!("Unexpected server event: {:?}", other);
                }
            }
        }
    }



    fn update_and_send_data_to_gui(&mut self, sender_to_gui: &Sender<String>) {
        //Drones
        let mut drone_data: HashMap<NodeId, DisplayDataDrone> = HashMap::new();
        for (drone_id, _) in self.command_senders_drones.iter() {
            //Get drone's neighbors and pdr
            let neighbors = self.state.topology.get(drone_id).cloned().unwrap_or_default();
            let pdr = self.drones_pdr.get(drone_id).unwrap_or(&0.0);

            // Insert drone data into the HashMap
            drone_data.insert(
                *drone_id,
                DisplayDataDrone {
                    node_id: *drone_id,
                    node_type: SpecificNodeType::Drone,
                    drone_brand: self.drones_data.get(drone_id).unwrap().drone_brand.clone(), //Get drone brand from drones_data
                    connected_nodes_ids: neighbors,
                    pdr: *pdr,
                },
            );
        }
        // Topology with types included in HashMap<NodeId, Vec<NodeId>>
        let mut topology_with_types = HashMap::new();

        for (node_id, connected_nodes) in self.state.topology.iter() {
            // Determine SpecificNodeType efficiently:
            let node_type = if let Some(node) = self.state.nodes.get(node_id) {
                match node {
                    NodeType::Client => {
                        match self.command_senders_clients.get(node_id) {
                            Some((_, client_type)) => match client_type {
                                ClientType::Chat => SpecificNodeType::ChatClient,
                                ClientType::Web => SpecificNodeType::WebBrowser,
                            },
                            None => {
                                warn!("Client type not found for client ID {}", node_id); // Handle the unexpected case
                                SpecificNodeType::Drone // Or another default or error handling
                            }
                        }
                    }
                    NodeType::Server => {
                        match self.command_senders_servers.get(node_id) {
                            Some((_, server_type)) => match server_type {
                                ServerType::Communication => SpecificNodeType::CommunicationServer,
                                ServerType::Media => SpecificNodeType::MediaServer,
                                ServerType::Text => SpecificNodeType::TextServer,
                                _ => {
                                    warn!("Unexpected server type for server ID {}", node_id);
                                    SpecificNodeType::Drone // Or another default or error handling
                                }
                            },
                            None => {
                                warn!("Server type not found for server ID {}", node_id);
                                SpecificNodeType::Drone  // Or another default or error handling
                            }
                        }
                    }
                    NodeType::Drone => SpecificNodeType::Drone,
                }
            } else {
                warn!("Node type not found for node ID {}", node_id); 
                SpecificNodeType::Drone  // Or another default or error handling
            };
            topology_with_types.insert(*node_id, (connected_nodes.clone(), node_type));
        }

        let display_data = DisplayDataSimulationController {
            data_title: "Network Data".to_string(),
            web_clients_data: self.web_clients_data.clone(),
            chat_clients_data: self.chat_clients_data.clone(),
            comm_servers_data: self.comm_servers_data.clone(),
            text_servers_data: self.text_servers_data.clone(),
            media_servers_data: self.media_servers_data.clone(),
            drones_data: drone_data,
            topology: topology_with_types,
        };
        // Serialize and send display data
        let json_string = serde_json::to_string(&display_data).unwrap();

        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");

        //Now, after sending data, request updates from clients and servers

        //Clients
        for (client_id, (client_com_sender, _)) in self.command_senders_clients.iter() {
            self.updating_nodes.insert(*client_id);  //Add this client to the updating_nodes set

            if let Err(err) = client_com_sender.send(ClientCommand::UpdateMonitoringData) {
                warn!("Error sending UpdateMonitoringData to client {}: {:?}", client_id, err);
            }
        }

        //Servers
        for (server_id, (server_com_sender, _)) in self.command_senders_servers.iter() {
            self.updating_nodes.insert(*server_id);    //Add this server to the updating_nodes set

            if let Err(err) = server_com_sender.send(ServerCommand::UpdateMonitoringData) {
                warn!("Error sending UpdateMonitoringData to server {}: {:?}", server_id, err);
            }
        }
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
    pub fn request_drone_crash(&mut self, drone_id: NodeId) -> Result<(), String> {
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

        Ok(())
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

    pub fn request_known_servers(&mut self, client_id: NodeId) -> Result<Vec<(ServerType, NodeId)>, String> {
        if let Some((client_command_sender, _)) = self.command_senders_clients.get(&client_id) {

            if client_command_sender.send(ClientCommand::GetKnownServers).is_err() {
                return Err(format!("Client {} disconnected", client_id));
            }

            //wait for KnownServers event
            let timeout = Duration::from_secs(1);

            //Using recv_client_event_timeout
            if let Some(event) = self.recv_client_event_timeout(timeout) {
                match event {
                    ClientEvent::KnownServers(servers) => {
                        self.update_known_servers(servers);
                        let server_options: Vec<(ServerType, NodeId)> = self.command_senders_servers   //Clone servers
                            .clone()   // Clone the servers vector to avoid the move
                            .iter()
                            .map(|(&id, &(_, server_type))| (server_type, id))
                            .collect();
                        // Update known servers in the controller and return the list for UI
                        return Ok(server_options); // Return the processed server list
                    },
                    _ => return Err("Unexpected client event".to_string()),
                }
            } else {
                return Err(format!("Timeout waiting for KnownServers from client {}", client_id))
            }
        } else {
            return Err(format!("Client with ID {} not found", client_id))
        }
    }


    fn recv_client_event_timeout(&self, timeout: Duration) -> Option<ClientEvent> {
        let start = std::time::Instant::now();

        loop {
            if let Ok(event) = self.client_event_receiver.try_recv() {
                return Some(event);
            }

            if start.elapsed() >= timeout {
                return None; // Timeout
            }
            sleep(Duration::from_millis(10));
        }

    }



    fn update_known_servers(&mut self, servers: Vec<(NodeId, ServerType, bool)>) {
        for (server_id, server_type, _) in servers { // Iterate over servers and their types

            if let Some((sender, _)) = self.command_senders_servers.get(&server_id) { // Check if server already exists in controller
                self.command_senders_servers.insert(server_id, (sender.clone(), server_type)); //Update server type
            } else {                                                                           //If server not found create it
                let (sender, receiver) = unbounded();        //Create channels for server
                self.command_senders_servers.insert(server_id, (sender, server_type));           //Insert server in controller
            }
        }
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
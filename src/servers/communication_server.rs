use crossbeam_channel::{select_biased, Receiver, Sender};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
};
use log::{error, info, warn};
use crate::general_use::{
    DataScope, DisplayDataCommunicationServer, Message, Query, Response, ServerCommand, ServerEvent,
    ServerType, SpecificNodeType
};
//UI
use crate::ui_traits::Monitoring;
use wg_2024::{
    network::NodeId,
    packet::Packet,
};
use crate::clients::client_chen::NodeType;
use crate::general_use::DataScope::{UpdateAll, UpdateSelf};
use super::server::CommunicationServer as CharTrait;
use super::server::Server as MainTrait;

type FloodId = u64;
type SessionId = u64;

#[derive(Debug)]
pub struct CommunicationServer{

    //Basic data
    pub id: NodeId,

    //Fragment-related
    pub reassembling_messages: HashMap<SessionId, Vec<u8>>,
    pub sending_messages: HashMap<SessionId, (Vec<u8>, NodeId)>,

    //Flood-related
    pub clients: HashSet<NodeId>,                               // Available clients
    pub topology: HashMap<NodeId, HashSet<NodeId>>,             // Nodes and their neighbours
    pub nodes: HashMap<NodeId, NodeType>,                       // Nodes and their types
    pub routes: HashMap<NodeId, Vec<NodeId>>,                   // Routes to the servers
    pub flood_ids: Vec<FloodId>,
    pub counter: (FloodId, SessionId),

    //Drop counter
    pub drops_counter: HashMap<SessionId, HashMap<NodeId, u8>>,

    //Channels
    pub to_controller_event: Sender<ServerEvent>,
    pub from_controller_command: Receiver<ServerCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,

    //Characteristic-Server fields
    pub list_users: Vec<NodeId>,

    //Queries to process
    pub queries_to_process: VecDeque<(NodeId, Query)>,
}

impl CommunicationServer{
    pub fn new(
        id: NodeId,
        to_controller_event: Sender<ServerEvent>,
        from_controller_command: Receiver<ServerCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        info!("Starting Communication server with ID: {}", id);
        CommunicationServer {
            id,

            reassembling_messages: Default::default(),
            sending_messages: Default::default(),

            clients: Default::default(),                                   // Available clients
            topology: Default::default(),
            nodes: Default::default(),
            routes: Default::default(),
            flood_ids: Default::default(),
            counter: (0, 0),

            to_controller_event,
            from_controller_command,
            packet_recv,
            packet_send,

            list_users: Vec::new(),

            drops_counter: HashMap::new(),

            queries_to_process: VecDeque::new(),
        }
    }
}

impl Monitoring for CommunicationServer {
    fn send_display_data(&mut self, data_scope: DataScope) {
        let neighbors =  self.packet_send.keys().cloned().collect();
        let display_data = DisplayDataCommunicationServer{
            node_id: self.id,
            node_type: SpecificNodeType::CommunicationServer,
            flood_id: self.flood_ids.last().cloned().unwrap_or(0),
            connected_node_ids: neighbors,
            known_clients: self.clients.clone(),
            routing_table: self.routes.clone(),
            registered_clients: self.list_users.clone(),
        };

        self.to_controller_event.send(ServerEvent::CommunicationServerData(self.id, display_data, data_scope)).expect("Failed to send communication server data");
    }
    fn run_with_monitoring(
        &mut self,
    ) {
        self.send_display_data(UpdateAll);
        loop {
            select_biased! {
                recv(self.get_from_controller_command()) -> command_res => {
                    if let Ok(command) = command_res {
                        match command {
                            ServerCommand::UpdateMonitoringData => {
                                self.send_display_data(UpdateAll);
                            }
                            ServerCommand::StartFlooding => {
                                self.discover();
                            }
                            ServerCommand::AddSender(id, sender) => {
                                self.get_packet_send().insert(id, sender);
                                self.send_display_data(UpdateSelf);
                            }
                            ServerCommand::RemoveSender(id) => {
                                self.get_packet_send().remove(&id);
                                self.update_topology_and_routes(id);
                                self.send_display_data(UpdateSelf);
                            }
                            ServerCommand::ShortcutPacket(packet) => {
                                self.handle_packet(packet);
                                self.send_display_data(UpdateSelf);
                            },
                            _ => {}
                        }
                    }
                },
                recv(self.get_packet_recv()) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                        self.send_display_data(UpdateSelf);
                    }
                },
            }
        }
    }
}

impl MainTrait for CommunicationServer{
    fn get_id(&self) -> NodeId{ self.id }
    fn get_server_type(&self) -> ServerType{ ServerType::Communication }

    fn get_session_id(&mut self) -> u64{
        self.counter.1 += 1;
        self.counter.1
    }

    fn get_flood_id(&mut self) -> u64{
        self.counter.0 += 1;
        self.counter.0
    }

    fn push_flood_id(&mut self, flood_id: FloodId){ self.flood_ids.push(flood_id); }
    fn get_clients(&mut self) -> &mut HashSet<NodeId>{ &mut self.clients }
    fn get_topology(&mut self) -> &mut HashMap<NodeId, HashSet<NodeId>>{ &mut self.topology }
    fn get_nodes(&mut self) -> &mut HashMap<NodeId, NodeType> { &mut self.nodes }
    fn get_routes(&mut self) -> &mut HashMap<NodeId, Vec<NodeId>>{ &mut self.routes }

    fn get_event_sender(&self) -> &Sender<ServerEvent>{ &self.to_controller_event }
    fn get_from_controller_command(&mut self) -> &mut Receiver<ServerCommand>{ &mut self.from_controller_command }
    fn get_packet_recv(&mut self) -> &mut Receiver<Packet>{ &mut self.packet_recv }
    fn get_packet_send(&mut self) -> &mut HashMap<NodeId, Sender<Packet>>{ &mut self.packet_send }
    fn get_packet_send_not_mutable(&self) -> &HashMap<NodeId, Sender<Packet>>{ &self.packet_send }
    fn get_reassembling_messages(&mut self) -> &mut HashMap<u64, Vec<u8>>{ &mut self.reassembling_messages }
    fn process_query(&mut self, query: Query, src_id: NodeId) {
        // Check if there is a route to the client, save query and start the discovery process if it's not.
        if self.routes.get(&src_id).is_none() {
            warn!("Server {}: Error sending response to query {:?}: no route to the Client {}",
                self.id, query, src_id);

            self.save_query_to_process(src_id, query);
            return;
        }

        match query {
            Query::AskType => self.give_type_back(src_id),

            Query::RegisterClient(node_id) => self.add_client(node_id),
            Query::AskListClients => self.give_list_back(src_id),
            Query::SendMessage(message) => self.forward_message_to(message),
            _ => {}
        }
    }
    fn get_sending_messages(&mut self) ->  &mut HashMap<u64, (Vec<u8>, u8)>{ &mut self.sending_messages }

    fn get_sending_messages_not_mutable(&self) -> &HashMap<u64, (Vec<u8>, u8)>{ &self.sending_messages }

    fn get_drops_counter(&mut self) -> &mut HashMap<u64, HashMap<NodeId, u8>>{ &mut self.drops_counter }

    fn get_queries_to_process(&mut self) -> &mut VecDeque<(NodeId, Query)>{ &mut self.queries_to_process }
}

impl CharTrait for CommunicationServer {
    fn add_client(&mut self, client_id: NodeId) {
        self.list_users.push(client_id);

        let response = Response::ClientRegistered;

        //Serializing message to send
        let response_as_string = serde_json::to_string(&response).unwrap();
        let response_in_vec_bytes = response_as_string.as_bytes();
        let length_response = response_in_vec_bytes.len();

        //Counting fragments
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        // Finding route
        let Some(route) = self.find_path_to(client_id) else {
            error!("Server {}: No route found to the client {}", self.get_id(), client_id);
            return;
        };

        //Generating header
        let header = Self::create_source_routing(route);

        // Generating ids
        let session_id = self.generate_unique_session_id();

        //Send fragments
        self.send_fragments(session_id, n_fragments,response_in_vec_bytes, header);
    }

    fn give_list_back(&mut self, client_id: NodeId) {

        //Get list
        let list_clients = self.list_users.clone();

        //Creating data to send
        let response = Response::ListClients(list_clients);

        //Serializing message to send
        let response_as_string = serde_json::to_string(&response).unwrap();
        let response_in_vec_bytes = response_as_string.as_bytes();
        let length_response = response_in_vec_bytes.len();

        //Counting fragments
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        // Finding route
        let Some(route) = self.find_path_to(client_id) else {
            error!("Server {}: No route found to the client {}", self.get_id(), client_id);
            return;
        };

        //Generating header
        let header = Self::create_source_routing(route);

        // Generating ids
        let session_id = self.generate_unique_session_id();

        //Send fragments
        self.send_fragments(session_id, n_fragments,response_in_vec_bytes, header);

    }

    fn forward_message_to(&mut self, message: Message) {

        //Creating data to send
        let response = Response::MessageReceived(message.clone());

        //Serializing message to send
        let response_as_string = serde_json::to_string(&response).unwrap();
        let response_in_vec_bytes = response_as_string.as_bytes();
        let length_response = response_in_vec_bytes.len();

        //Counting fragments
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        // Finding route
        let recipient_id = message.get_recipient();
        let Some(route) = self.find_path_to(recipient_id) else {
            error!("Server {}: No route found to the client {}", self.get_id(), recipient_id);
            return;
        };

        //Generating header
        let header = Self::create_source_routing(route);

        // Generating fragment
        let session_id = self.generate_unique_session_id();

        self.send_fragments(session_id, n_fragments,response_in_vec_bytes, header);
    }
}

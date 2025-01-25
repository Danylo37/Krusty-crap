use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::future::Future;
use tokio::sync::mpsc;
use tokio::select;
use wg_2024::{
    network::{NodeId},
    packet::{
        Packet,
        PacketType,
    },
};
use crate::clients::client_chen::Serialize;
use crate::general_use::{Query, Response, ServerCommand, ServerEvent, ServerType};
use crate::ui_traits::{crossbeam_to_tokio_bridge, Monitoring};
use super::server::TextServer as CharTrait;
use super::server::Server as MainTrait;

type FloodId = u64;
type SessionId = u64;
#[derive(Debug)]
pub struct TextServer{

    //Basic data
    pub id: NodeId,

    //Fragment-related
    pub reassembling_messages: HashMap<SessionId, Vec<u8>>,
    pub sending_messages: HashMap<SessionId, (Vec<u8>, NodeId)>,

    //Flood-related
    pub clients: Vec<NodeId>,                                   // Available clients
    pub topology: HashMap<NodeId, Vec<NodeId>>,             // Nodes and their neighbours
    pub routes: HashMap<NodeId, Vec<NodeId>>,                   // Routes to the servers
    pub flood_ids: Vec<FloodId>,
    pub counter: (FloodId, SessionId),

    //Channels
    pub to_controller_event: Sender<ServerEvent>,
    pub from_controller_command: Receiver<ServerCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,

    //Characteristic-Server fields
    pub content: HashMap<String, String>,
}

impl TextServer{
    pub fn new(
        id: NodeId,
        content: HashMap<String, String>,
        to_controller_event: Sender<ServerEvent>,
        from_controller_command: Receiver<ServerCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        TextServer {
            id,

            reassembling_messages: Default::default(),
            sending_messages: Default::default(),

            clients: Default::default(),                                   // Available clients
            topology: Default::default(),
            routes: Default::default(),
            flood_ids: Default::default(),
            counter: (0, 0),

            to_controller_event,
            from_controller_command,
            packet_recv,
            packet_send,

            content,
        }
    }
}

#[derive(Debug, Serialize)]
struct DisplayDataTextServer{
    node_id: NodeId,
    node_type: String,
    flood_id: crate::general_use::FloodId,
    //session_id: crate::general_use::SessionId,
    connected_node_ids: HashSet<NodeId>,
    routing_table: HashMap<NodeId, Vec<NodeId>>,
    text_files: Vec<String>,
}

impl Monitoring for TextServer {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>) {
        let neighbors =  self.packet_send.keys().cloned().collect();
        let display_data = DisplayDataTextServer {
            node_id: self.id,
            node_type: "Text Server".to_string(),
            flood_id: self.flood_ids.last().cloned().unwrap_or(0),
            connected_node_ids: neighbors,
            routing_table: self.routes.clone(),
            text_files: self.content.clone(),
        };

        let json_string = serde_json::to_string(&display_data).unwrap();
        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
    }
    fn run_with_monitoring(
        &mut self,
        sender_to_gui: Sender<String>,
    ) {
        self.send_display_data(sender_to_gui.clone());
        loop {
            select_biased! {
                recv(self.get_from_controller_command()) -> command_res => {
                    if let Ok(command) = command_res {
                        match command {
                            ServerCommand::AddSender(id, sender) => {
                                self.get_packet_send().insert(id, sender);

                            }
                            ServerCommand::RemoveSender(id) => {
                                self.get_packet_send().remove(&id);
                            }
                            ServerCommand::ShortcutPacket(packet) => {
                                 match packet.pack_type {
                                    PacketType::Nack(nack) => self.handle_nack(nack, packet.session_id),
                                    PacketType::Ack(ack) => self.handle_ack(ack),
                                    PacketType::MsgFragment(fragment) => self.handle_fragment(fragment, packet.routing_header ,packet.session_id),
                                    PacketType::FloodRequest(flood_request) => self.handle_flood_request(flood_request, packet.session_id),
                                    PacketType::FloodResponse(flood_response) => self.handle_flood_response(flood_response),
                                }
                            }
                        }
                        self.send_display_data(sender_to_gui.clone());
                    }
                },
                recv(self.get_packet_recv()) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        match packet.pack_type {
                            PacketType::Nack(nack) => self.handle_nack(nack, packet.session_id),
                            PacketType::Ack(ack) => self.handle_ack(ack),
                            PacketType::MsgFragment(fragment) => self.handle_fragment(fragment, packet.routing_header ,packet.session_id),
                            PacketType::FloodRequest(flood_request) => self.handle_flood_request(flood_request, packet.session_id),
                            PacketType::FloodResponse(flood_response) => self.handle_flood_response(flood_response),
                        }
                        self.send_display_data(sender_to_gui.clone());
                    }
                },
            }
        }
    }
}

impl MainTrait for TextServer{
    fn get_id(&self) -> NodeId{ self.id }
    fn get_server_type(&self) -> ServerType{ ServerType::Text }

    fn get_session_id(&mut self) -> u64{
        self.counter.1 += 1;
        self.counter.1
    }

    fn get_flood_id(&mut self) -> u64{
        self.counter.0 += 1;
        self.counter.0
    }

    fn push_flood_id(&mut self, flood_id: FloodId){ self.flood_ids.push(flood_id); }
    fn get_clients(&mut self) -> &mut Vec<NodeId>{ &mut self.clients }
    fn get_topology(&mut self) -> &mut HashMap<NodeId, Vec<NodeId>>{ &mut self.topology }
    fn get_routes(&mut self) -> &mut HashMap<NodeId, Vec<NodeId>>{ &mut self.routes }

    fn get_from_controller_command(&mut self) -> &mut Receiver<ServerCommand>{ &mut self.from_controller_command }
    fn get_packet_recv(&mut self) -> &mut Receiver<Packet>{ &mut self.packet_recv }
    fn get_packet_send(&mut self) -> &mut HashMap<NodeId, Sender<Packet>>{ &mut self.packet_send }
    fn get_packet_send_not_mutable(&self) -> &HashMap<NodeId, Sender<Packet>>{ &self.packet_send }
    fn get_reassembling_messages(&mut self) -> &mut HashMap<u64, Vec<u8>>{ &mut self.reassembling_messages }
    fn get_sending_messages(&mut self) ->  &mut HashMap<u64, (Vec<u8>, u8)>{ &mut self.sending_messages }
    fn get_sending_messages_not_mutable(&self) -> &HashMap<u64, (Vec<u8>, u8)>{ &self.sending_messages }


    fn process_reassembled_message(&mut self, data: Vec<u8>, src_id: NodeId){
        match String::from_utf8(data.clone()) {
            Ok(data_string) => match serde_json::from_str(&data_string) {
                Ok(Query::AskType) => self.give_type_back(src_id),

                Ok(Query::AskListFiles) => self.give_list_back(src_id),
                Ok(Query::AskFile(file_key)) => self.give_file_back(src_id, file_key),

                Err(_) => {
                    panic!("Damn, not the right struct")
                }
                _ => {}
            },
            Err(e) => println!("Argh, {:?}", e),
        }
    }
}

impl CharTrait for TextServer{
    fn give_list_back(&mut self, client_id: NodeId) {

        //Get list
        let list_files = self.content.clone();

        //Creating data to send
        let response = Response::ListFiles(list_files.keys().cloned().collect::<Vec<String>>());

        //Serializing message to send
        let response_as_string = serde_json::to_string(&response).unwrap();
        let response_in_vec_bytes = response_as_string.as_bytes();
        let length_response = response_in_vec_bytes.len();

        //Counting fragments
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        //Generating header
        let route: Vec<NodeId> = self.find_path_to(client_id);
        let header = Self::create_source_routing(route);

        // Generating ids
        let session_id = self.generate_unique_session_id();

        //Send fragments
        self.send_fragments(session_id, n_fragments,response_in_vec_bytes, header);

    }

    fn give_file_back(&mut self, client_id: NodeId, file_key: String) {

        //Get file
        let file:&String = self.content.get(&file_key).unwrap();

        //Creating data to send
        let response = Response::File(file.clone());

        //Serializing message to send
        let response_as_string = serde_json::to_string(&response).unwrap();
        let response_in_vec_bytes = response_as_string.as_bytes();
        let length_response = response_in_vec_bytes.len();

        //Counting fragments
        let mut n_fragments = length_response / 128+1;
        if n_fragments == 0 {
            n_fragments -= 1;
        }

        //Generating header
        let route: Vec<NodeId> = self.find_path_to(client_id);
        let header = Self::create_source_routing(route);

        // Generating ids
        let session_id = self.generate_unique_session_id();

        //Send fragments
        self.send_fragments(session_id, n_fragments,response_in_vec_bytes, header);

    }
}

use crate::clients::client_chen::prelude::*;
use crate::ui_traits::{crossbeam_to_tokio_bridge, Monitoring};
use crate::clients::client_chen::{ClientChen, CommandHandler, FragmentsHandler, PacketsReceiver, Router, Sending};
use rmp_serde::encode::to_vec;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver, select_biased};
use crate::clients::Client;
use crate::general_use::MediaRef;

#[derive(Debug, Serialize)]
struct DisplayDataWebBrowser {
    node_id: NodeId,
    node_type: String,
    flood_id: FloodId,
    session_id: SessionId,
    connected_node_ids: HashSet<NodeId>,
    registered_communication_servers: HashMap<ServerId, Vec<ClientId>>,
    registered_content_servers: HashSet<ServerId>,
    routing_table: HashMap<NodeId, Vec<Vec<NodeId>>>,
    curr_received_file_list: Vec<String>,
    chosen_file_text: String,
    serialized_media: HashMap<MediaRef, String>,
}


impl Monitoring for ClientChen{
    fn send_display_data(&mut self, sender_to_gui:Sender<String>){
        self.update_connected_nodes();
        let transformed_routing_table: HashMap<NodeId, Vec<Vec<NodeId>>> = self
            .communication
            .routing_table
            .iter()
            .map(|(node_id, routes)| {
                let paths: Vec<Vec<NodeId>> = routes
                    .keys()
                    .cloned()
                    .collect();
                (*node_id, paths)
            })
            .collect();

        // Create the DisplayData struct
        let display_data = DisplayDataWebBrowser {
            node_id: self.metadata.node_id,
            node_type: "Web Browser".to_string(),
            flood_id: self.status.flood_id,
            session_id: self.status.session_id,
            connected_node_ids: self.communication.connected_nodes_ids.clone(),
            registered_communication_servers: self.communication.registered_communication_servers.clone(),
            registered_content_servers: self.communication.registered_content_servers.clone(),
            routing_table: transformed_routing_table,
            curr_received_file_list: self.storage.current_list_file.clone(),
            chosen_file_text: self.storage.current_requested_text_file.clone(),
            serialized_media: self.storage.current_received_serialized_media.clone(),
        };

        // Serialize the DisplayData to MessagePack binary
        let json_string = serde_json::to_string(&display_data).unwrap();
        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
    }

    fn run_with_monitoring(&mut self, sender_to_gui: Sender<String>) {
        self.send_display_data(sender_to_gui.clone());
        loop {
            select_biased! {
                recv(self.communication_tools.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        // Handle the command
                        self.handle_controller_command_with_monitoring(command, sender_to_gui.clone());

                        // Things to do after handling the command
                        self.handle_fragments_in_buffer_with_checking_status();
                        self.send_packets_in_buffer_with_checking_status();
                        self.update_routing_checking_status();

                        // Update the network
                        self.send_display_data(sender_to_gui.clone());
                    }
                },
                recv(self.communication_tools.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        // Handle the packet
                        self.handle_received_packet(packet);

                        // Things to do after handling the packets
                        self.handle_fragments_in_buffer_with_checking_status();
                        self.send_packets_in_buffer_with_checking_status();
                        self.update_routing_checking_status();

                        // Update the network
                        self.send_display_data(sender_to_gui.clone());
                    }
                },
            }
        }
    }
}
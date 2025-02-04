use crate::clients::client_chen::prelude::*;
use crate::ui_traits::Monitoring;
use crate::clients::client_chen::{ClientChen, CommandHandler, FragmentsHandler, PacketsReceiver, Router, Sending};
use std::collections::{HashMap};
use crossbeam_channel::{Sender, select_biased};
use log::log;
use crate::general_use::{DataScope, DisplayDataWebBrowser};
use crate::general_use::ClientEvent::WebClientData;
use crate::general_use::DataScope::UpdateAll;

impl Monitoring for ClientChen{
    fn send_display_data(&mut self, sender_to_gui: Sender<String>, data_scope: DataScope){
        self.update_connected_nodes();
        // Create the DisplayData struct
        let display_data = DisplayDataWebBrowser {
            node_id: self.metadata.node_id,
            node_type: "Web Browser".to_string(),
            flood_id: self.status.flood_id,
            session_id: self.status.session_id,
            connected_node_ids: self.communication.connected_nodes_ids.clone(),
            routing_table: self.communication.routing_table.clone(),
            curr_received_file_list: self.storage.current_list_file.clone(),
            chosen_file_text: self.storage.current_requested_text_file.clone(),
            serialized_media: self.storage.current_received_serialized_media.clone(),
        };
        self.send_event(WebClientData(self.metadata.node_id, display_data, data_scope));
        info!("Data sent with scope {:?}", data_scope);
    }

    fn run_with_monitoring(&mut self, sender_to_gui: Sender<String>) {
        loop {
            select_biased! {
                recv(self.communication_tools.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        // Handle the command
                        self.handle_controller_command_with_monitoring(command, sender_to_gui.clone());
                        // Things to do after handling the command
                        self.handle_fragments_in_buffer_with_checking_status();
                        self.send_packets_in_buffer_with_checking_status();
                    }
                },
                recv(self.communication_tools.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        // Handle the packet
                        self.handle_received_packet(packet);

                        // Things to do after handling the packets
                        self.handle_fragments_in_buffer_with_checking_status();
                        self.send_packets_in_buffer_with_checking_status();
                    }
                },
            }
        }
    }
}
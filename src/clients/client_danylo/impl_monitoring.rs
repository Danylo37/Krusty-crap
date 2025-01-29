use crate::general_use::{ClientCommand, ClientEvent, ClientId, DataScope, DisplayDataChatClient, FloodId, ServerId, ServerType, SessionId};
use crate::ui_traits::Monitoring;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use crossbeam_channel::Sender;
use log::{debug, info};
use wg_2024::network::NodeId;
use crate::general_use::DataScope::{UpdateAll, UpdateSelf};
use super::{ChatClientDanylo, ChatHistory};


impl Monitoring for ChatClientDanylo {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>, data_scope: DataScope) {
        let connected_nodes_ids = self.packet_send.keys().cloned().collect();
        let display_data = DisplayDataChatClient {
            node_id: self.id,
            node_type: "Chat Client".to_string(),
            flood_ids: self.flood_ids.clone(),
            session_ids: self.session_ids.clone(),
            neighbours: connected_nodes_ids,
            discovered_servers: self.servers.clone(),
            //registered_communication_servers: self.is_registered.clone(),
            available_clients: self.clients.clone(),
            chats: self.chats.clone(),
        };

        self.controller_send.send(ClientEvent::ChatClientData(self.id, display_data, data_scope)).expect("Failed to send chat client data");
        info!("Client {}: Sending chat client data", self.id);  // todo
    }
    fn run_with_monitoring(
        &mut self,
        sender_to_gui: Sender<String>,
    )  {
        info!("Running ChatClientDanylo with ID: {}", self.id);
        self.send_display_data(sender_to_gui.clone(),DataScope::UpdateAll);
        loop {
            crossbeam_channel::select_biased! {
                recv(self.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        self.handle_command_with_monitoring(command, sender_to_gui.clone());
                        //self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                        //self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
                    }
                },
            }
        }
    }
}

impl ChatClientDanylo{
    pub(crate) fn handle_command_with_monitoring(&mut self, command: ClientCommand, sender_to_gui: Sender<String>) {
        info!("Client {}: Handling command: {:?}", self.id, command);

        match command {
            ClientCommand::UpdateMonitoringData => {
                self.send_display_data(sender_to_gui.clone(), DataScope::UpdateAll);
            },
            ClientCommand::AddSender(id, sender) => {
                self.add_sender(id, sender);
                self.send_display_data(sender_to_gui.clone(), UpdateSelf);
            }
            ClientCommand::RemoveSender(id) => {
                self.remove_sender(id);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::ShortcutPacket(packet) => {
                info!("Client {}: Shortcut packet received from SC: {:?}", self.id, packet);
                self.handle_packet(packet);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::GetKnownServers => {
                self.send_known_servers();
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::StartFlooding => {
                self.discovery();
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::AskTypeTo(server_id) => {
                self.request_server_type(server_id);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::SendMessageTo(to, message) => {
                self.send_message_to(to, message);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::RegisterToServer(server_id) => {
                self.request_to_register(server_id);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            ClientCommand::AskListClients(server_id) => {
                self.request_clients_list(server_id);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            }
            _ => {}
        }
    }
}
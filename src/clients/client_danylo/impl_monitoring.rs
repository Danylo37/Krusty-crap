use crate::general_use::{ClientCommand, ClientId, FloodId, ServerId, ServerType, SessionId};
use crate::ui_traits::Monitoring;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use crossbeam_channel::Sender;
use log::{debug, info};
use wg_2024::network::NodeId;
use super::{ChatClientDanylo, ChatHistory};

#[derive(Debug, Serialize)]
pub struct ChatClientDisplayData {
    // Client metadata
    node_id: NodeId,
    node_type: String,

    // Used IDs
    flood_ids: Vec<FloodId>,
    session_ids: Vec<SessionId>,

    // Connections
    neighbours: HashSet<NodeId>,
    discovered_servers: HashMap<ServerId, ServerType>,
    //registered_communication_servers: HashMap<ServerId, bool>,
    available_clients: HashMap<ServerId, Vec<ClientId>>,

    // Chats
    chats: HashMap<ClientId, ChatHistory>,
}


impl Monitoring for ChatClientDanylo {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>) {
        let connected_nodes_ids = self.packet_send.keys().cloned().collect();
        let display_data = ChatClientDisplayData {
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

        // Serialize the DisplayData to MessagePack binary
        let json_string = serde_json::to_string(&display_data).unwrap();
        let _ = sender_to_gui.send(json_string).is_err();
    }
    fn run_with_monitoring(
        &mut self,
        sender_to_gui: Sender<String>,
    )  {
        info!("Running ChatClientDanylo with ID: {}", self.id);
        self.send_display_data(sender_to_gui.clone());
        loop {
            crossbeam_channel::select_biased! {
                recv(self.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        self.handle_command_with_monitoring(command, sender_to_gui.clone());
                        self.send_display_data(sender_to_gui.clone());
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_packet(packet);
                        self.send_display_data(sender_to_gui.clone());
                    }
                },
            }
        }
    }
}

impl ChatClientDanylo{
    pub(crate) fn handle_command_with_monitoring(&mut self, command: ClientCommand, sender_to_gui: Sender<String>) {
        debug!("Client {}: Handling command: {:?}", self.id, command);

        match command {
            ClientCommand::UpdateMonitoringData => {
                self.send_display_data(sender_to_gui.clone());
            },
            ClientCommand::AddSender(id, sender) => {
                self.packet_send.insert(id, sender);
                info!("Client {}: Added sender for node {}", self.id, id);
            }
            ClientCommand::RemoveSender(id) => {
                self.packet_send.remove(&id);
                info!("Client {}: Removed sender for node {}", self.id, id);
            }
            ClientCommand::ShortcutPacket(packet) => {
                info!("Client {}: Shortcut packet received from SC: {:?}", self.id, packet);
                self.handle_packet(packet);
            }
            ClientCommand::GetKnownServers => {
                self.handle_get_known_servers()
            }
            ClientCommand::StartFlooding => {
                self.discovery()
            }
            ClientCommand::AskTypeTo(server_id) => {
                self.request_server_type(server_id)
            }
            ClientCommand::SendMessageTo(to, message) => {
                self.send_message_to(to, message)
            }
            ClientCommand::RegisterToServer(server_id) => {
                self.request_to_register(server_id)
            }
            ClientCommand::AskListClients(server_id) => {
                self.request_clients_list(server_id)
            }
            _ => {}
        }
    }
}
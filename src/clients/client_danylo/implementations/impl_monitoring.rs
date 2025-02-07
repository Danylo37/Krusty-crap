use crossbeam_channel::Sender;
use log::info;

use crate::{
    general_use::{
        ClientCommand, ClientEvent, DataScope, DisplayDataChatClient, DataScope::{UpdateAll, UpdateSelf}
    },
    ui_traits::Monitoring,
};
use crate::general_use::SpecificNodeType;
use super::{ChatClientDanylo, Senders, PacketHandler, CommandHandler};

impl Monitoring for ChatClientDanylo {
    fn send_display_data(&mut self, data_scope: DataScope) {
        let connected_nodes_ids = self.packet_send.keys().cloned().collect();
        let display_data = DisplayDataChatClient {
            node_id: self.id,
            node_type: SpecificNodeType::ChatClient,
            flood_ids: self.flood_ids.clone(),
            routes: self.routes.clone(),
            session_ids: self.session_ids.clone(),
            neighbours: connected_nodes_ids,
            discovered_servers: self.servers.clone(),
            available_clients: self.clients.clone(),
            chats: self.chats.clone(),
        };

        self.send_event(ClientEvent::ChatClientData(self.id, display_data, data_scope));
    }
    fn run_with_monitoring(
        &mut self
    )  {
        info!("Running ChatClientDanylo with ID: {}", self.id);
        self.send_display_data(UpdateAll);
        loop {
            crossbeam_channel::select_biased! {
                recv(self.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        info!("Client {}: Received command: {:?}", self.id, command);
                        self.handle_command_with_monitoring(command);
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        info!("Client {}: Received packet: {:?}", self.id, packet);
                        self.handle_packet(packet);
                        self.send_display_data(DataScope::UpdateSelf);
                    }
                },
            }
        }
    }
}

impl ChatClientDanylo{
    fn handle_command_with_monitoring(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::UpdateMonitoringData => {
                self.send_display_data(UpdateAll);
            },
            ClientCommand::AddSender(id, sender) => {
                self.add_sender(id, sender);
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::RemoveSender(id) => {
                self.remove_sender(id);
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::ShortcutPacket(packet) => {
                info!("Client {}: Shortcut packet received from SC: {:?}", self.id, packet);
                self.handle_packet(packet);
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::GetKnownServers => {
                self.send_known_servers();
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::StartFlooding => {
                self.discovery();
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::AskTypeTo(server_id) => {
                self.request_server_type(server_id);
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::SendMessageTo(to, message) => {
                self.send_message_to(to, message);
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::RegisterToServer(server_id) => {
                self.request_to_register(server_id);
                self.send_display_data(UpdateSelf);
            }
            ClientCommand::AskListClients(server_id) => {
                self.request_clients_list(server_id);
                self.send_display_data(UpdateSelf);
            }
            _ => {}
        }
    }
}
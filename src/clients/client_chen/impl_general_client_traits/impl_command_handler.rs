use crate::clients::client_chen::{ClientChen, CommandHandler, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::general_client_traits::*;
use crate::general_use::{DataScope, ServerType};
use crate::general_use::DataScope::{UpdateAll, UpdateSelf};
use crate::ui_traits::Monitoring;

impl CommandHandler for ClientChen{
    fn handle_controller_command(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::AddSender(target_node_id, sender) => {
                self.communication_tools.packet_send.insert(target_node_id, sender);
            }
            ClientCommand::RemoveSender(target_node_id) => {
                self.communication_tools.packet_send.remove(&target_node_id);
            }

            ClientCommand::StartFlooding => {
                self.do_flooding();
            }
            ClientCommand::GetKnownServers => {
                let servers: Vec<(ServerId, ServerType, bool)> = self
                    .get_discovered_servers_from_topology()
                    .iter()
                    .map(|server_id| {
                        self.network_info.topology.get(server_id).map_or(
                            // Default to undefined server info if not found
                            (*server_id, ServerType::Undefined, false),
                            |server| {
                                if let SpecificInfo::ServerInfo(server_info) = &server.specific_info {
                                    let server_type = server_info.server_type;
                                    (*server_id, server_type, false)
                                } else {
                                    (*server_id, ServerType::Undefined, false)
                                }
                            },
                        )
                    })
                    .collect();
                self.send_event(ClientEvent::KnownServers(servers));
            }

            ClientCommand::AskTypeTo(server_id) => {
                self.ask_server_type(server_id);
            }
            ClientCommand::RequestListFile(server_id) => {
                println!("CLIENT PROCESSING ASK FILE LIST COMMAND");
                self.ask_list_files(server_id);
            }
            ClientCommand::RequestText(server_id, file_ref) => {
                self.ask_file(server_id, file_ref);
            }
            ClientCommand::RequestMedia(media_ref) => {
                self.ask_media(media_ref);
            }
            ClientCommand::DroneFixed(_drone_id) => {
                let packet_status_map = self.storage.packets_status.get(&self.status.session_id).cloned();

                if let Some(packet_status_map) = packet_status_map {
                    for (fragment_index, status) in packet_status_map {
                        if matches!(status, PacketStatus::WaitingForFixing) {   // this is considering that we are sending packets one session at time.
                            if let Some(output_buffer_map) = self.storage.output_buffer.get(&self.status.session_id) {
                                if let Some(packet) = output_buffer_map.get(&fragment_index) {
                                    self.send(packet.clone());
                                } else {
                                    warn!("Packet not found in output buffer for fragment index: {}", fragment_index);
                                }
                            } else {
                                warn!("Output buffer not found for session ID: {}", self.status.session_id);
                            }
                        }
                    }
                } else {
                    warn!("Packet status map not found for session ID: {}", self.status.session_id);
                }
            }
            _ => {}
        }
    }

    fn handle_controller_command_with_monitoring(&mut self, command: ClientCommand) {
        match command {
            ClientCommand::UpdateMonitoringData => {
                //debug!("I'm here sending data with scope UpdateAll");
                self.send_display_data(UpdateAll);
            },
            _=> {
                self.handle_controller_command(command);
                self.send_display_data(UpdateSelf);
            },
        }
    }
}
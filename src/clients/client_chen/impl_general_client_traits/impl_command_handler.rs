use crate::clients::client_chen::{ClientChen, CommandHandler, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::general_client_traits::*;
use crate::general_use::{ServerType};
use crate::general_use::DataScope::{UpdateAll};
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
                println!("
                CLIENT PROCESSING ASK FILE LIST COMMAND");
                self.ask_list_files(server_id);
            }
            ClientCommand::RequestText(server_id, file_ref) => {
                self.ask_file(server_id, file_ref);
            }
            ClientCommand::RequestMedia(media_ref) => {
                self.ask_media(media_ref);
            }
            ClientCommand::DroneFixed(drone_id) => {
                println!("*******************************************************************\n Client [{}] is processing the drone [{}] fixed\n*******************************************************************", self.metadata.node_id, drone_id);


                // Collect (session_id, fragment_index) pairs where the status is WaitingForFixing(drone_id)
                let filtered_pairs: Vec<(SessionId, FragmentIndex)> = self
                    .storage
                    .packets_status
                    .iter()
                    .flat_map(|(session_id, packet_status_map)| {
                        packet_status_map.iter().filter_map(move |(&fragment_index, status)| {
                            if let PacketStatus::WaitingForFixing(dr) = status {
                                if *dr == drone_id {
                                    return Some((*session_id, fragment_index));
                                }
                            }
                            None
                        })
                    })
                    .collect();

                // Process the filtered packets
                for (session_id, fragment_index) in filtered_pairs {
                    match self.storage.output_buffer.get(&session_id) {
                        Some(output_buffer_map) => match output_buffer_map.get(&fragment_index) {
                            Some(packet) => self.send(packet.clone()), // Consider removing .clone() if not needed
                            None => println!(
                    "Packet not found in output buffer | session_id: {}, fragment_index: {}",
                    session_id, fragment_index
                ),
                        },
                        None => println!(
                "Output buffer not found | session_id: {}",
                session_id
            ),
                    }
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
            },
        }
    }
}
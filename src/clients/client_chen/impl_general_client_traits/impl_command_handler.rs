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
                // Get the registered servers before the closure
                let registered_servers = self.get_registered_servers();

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
                                    let is_registered = registered_servers.contains(server_id);
                                    (*server_id, server_type, is_registered)
                                } else {
                                    (*server_id, ServerType::Undefined, false)
                                }
                            },
                        )
                    })
                    .collect();
                self.send_events(ClientEvent::KnownServers(servers));
            }

            ClientCommand::AskTypeTo(server_id) => {
                self.send_query(server_id, Query::AskType);
            }
            ClientCommand::RequestListFile(server_id) => {
                self.send_query(server_id, Query::AskListFiles);
            }
            ClientCommand::RequestText(server_id, file) => {
                self.send_query(server_id, Query::AskFile(file));
            }
            ClientCommand::RequestMedia(server_id, media_ref) => {
                self.send_query(server_id, Query::AskMedia(media_ref));
            }
            _=>{}
        }
    }

    fn handle_controller_command_with_monitoring(&mut self, command: ClientCommand, sender_to_gui: Sender<String>) {
        match command {
            ClientCommand::UpdateMonitoringData => {
                debug!("I'm here sending data with scope UpdateAll");
                self.send_display_data(sender_to_gui.clone(), DataScope::UpdateAll);
            },

            ClientCommand::AddSender(target_node_id, sender) => {
                debug!("Received command to add sender");
                self.communication_tools.packet_send.insert(target_node_id, sender);
                self.send_display_data(sender_to_gui.clone(), UpdateSelf);
            },
            ClientCommand::RemoveSender(target_node_id) => {
                debug!("Received command to remove sender");
                self.communication_tools.packet_send.remove(&target_node_id);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },

            ClientCommand::StartFlooding => {
                debug!("Received command to start flooding");
                self.do_flooding();
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },
            ClientCommand::GetKnownServers => {
                debug!("Received command to get the know servers");
                // Get the registered servers before the closure
                let registered_servers = self.get_registered_servers();

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
                                    let is_registered = registered_servers.contains(server_id);
                                    (*server_id, server_type, is_registered)
                                } else {
                                    (*server_id, ServerType::Undefined, false)
                                }
                            },
                        )
                    })
                    .collect();
                self.send_events(ClientEvent::KnownServers(servers));
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },

            ClientCommand::AskTypeTo(server_id) => {
                debug!("Received command to get the ask type to");
                self.send_query(server_id, Query::AskType);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },
            ClientCommand::RequestListFile(server_id) => {
                debug!("Received command to request file list");
                self.send_query(server_id, Query::AskListFiles);
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },
            ClientCommand::RequestText(server_id, file) => {
                debug!("Received command to request text");
                self.send_query(server_id, Query::AskFile(file));
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },
            ClientCommand::RequestMedia(server_id, media_ref) => {
                debug!("Received command to request media");
                self.send_query(server_id, Query::AskMedia(media_ref));
                self.send_display_data(sender_to_gui.clone(),DataScope::UpdateSelf);
            },
            _=>{}
        }
    }
}
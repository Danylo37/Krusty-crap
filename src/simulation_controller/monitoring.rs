use std::collections::{HashSet};
use crossbeam_channel::{select_biased, Sender};
use log::{debug, info};
use crate::clients::client_chen::{NodeId};
use crate::simulation_controller::SimulationController;
use crate::ui_traits::{SimulationControllerMonitoring};
use crate::websocket::{WsCommand};
use crate::general_use::{ClientCommand, ClientEvent, DataScope, DisplayDataSimulationController, ServerCommand, ServerEvent};

impl SimulationControllerMonitoring for SimulationController {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>) {
        let topology_with_types = self.create_topology_with_types();

        let display_data = DisplayDataSimulationController{
            data_title: "Network Data".to_string(),
            web_clients_data: self.web_clients_data.clone(),
            chat_clients_data: self.chat_clients_data.clone(),
            comm_servers_data: self.comm_servers_data.clone(),
            text_servers_data: self.text_servers_data.clone(),
            media_servers_data: self.media_servers_data.clone(),
            drones_data: self.drones_data.clone(),
            topology: topology_with_types,
        };
        let json_string = serde_json::to_string(&display_data).unwrap();
        info!("Controller has sent the data of all the nodes {:?}", display_data);
        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
    }

    fn run_with_monitoring(&mut self, sender_to_gui: Sender<String>) {
        // Initiate discovery process for all clients
        for (_, (sender, _)) in self.command_senders_clients.iter(){
            sender.send(ClientCommand::StartFlooding).unwrap();
        }
        //Reminder: I put here the edge_nodes because I'm assuming the clients and the server must be fixed
        //created from the network initializer
        let mut edge_nodes = self.command_senders_clients.keys().cloned().collect::<HashSet<NodeId>>();
        edge_nodes.extend(self.command_senders_servers.keys().cloned().collect::<HashSet<NodeId>>());

        self.updating_nodes = edge_nodes.clone();
        loop {
            self.process_packet_sent_events();
            self.process_packet_dropped_events();
            self.process_controller_shortcut_events();
            select_biased! {
                recv(self.ws_command_receiver) -> command_res => {
                    debug!("Controller received command {:?}", command_res);
                    if let Ok(command) = command_res {
                        self.handle_ws_command(command, &sender_to_gui);
                    }
                },
                recv(self.client_event_receiver) -> client_event => {
                    if let Ok(event) = client_event {
                        //debug!("Controller received client event {:?}", event);
                        let mut conditional_data_scope = DataScope::UpdateAll;
                        match event{
                            ClientEvent::WebClientData(id, data, data_scope) => {
                                match data_scope{
                                    DataScope::UpdateAll => {
                                        conditional_data_scope = DataScope::UpdateAll;
                                        self.web_clients_data.insert(id, data);
                                        self.updating_nodes.remove(&id);
                                    },
                                    DataScope::UpdateSelf =>{
                                        conditional_data_scope = DataScope::UpdateSelf;
                                        self.web_clients_data.insert(id, data.clone());
                                        let json_string = serde_json::to_string(&data).unwrap();
                                        info!("Client {} has sent json data with scope UpdateSelf {:?} ", id, json_string);
                                        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
                                    }
                                }

                            },
                            ClientEvent::ChatClientData(id, data, data_scope) => {
                                match data_scope{
                                    DataScope::UpdateAll =>{
                                        conditional_data_scope = DataScope::UpdateAll;
                                        self.chat_clients_data.insert(id, data);
                                        self.updating_nodes.remove(&id);
                                    },
                                    DataScope::UpdateSelf =>{
                                        conditional_data_scope = DataScope::UpdateSelf;
                                        self.chat_clients_data.insert(id, data.clone());
                                        let json_string = serde_json::to_string(&data).unwrap();
                                        info!("Sent json data with scope UpdateSelf {:?} ", json_string);
                                        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
                                    }
                                }
                            },
                            ClientEvent::CallTechniciansToFixDrone(id, sender) => {
                                self.fix_drone(id, sender, sender_to_gui.clone());
                            },
                            ClientEvent::ControllerShortcut(packet) => {
                                self.send_shortcut(packet);
                            },
                            _ => {}
                        }
                        if self.updating_nodes.is_empty() && conditional_data_scope == DataScope::UpdateAll {
                            self.send_display_data(sender_to_gui.clone());
                            self.updating_nodes = edge_nodes.clone();
                            //eprintln!("updating_node: {:?}", self.updating_nodes);
                        }
                    }
                },

                recv(self.server_event_receiver) -> server_event => {
                    if let Ok(event) = server_event {
                        debug!("Controller received server event {:?}", event);
                        let mut conditional_data_scope = DataScope::UpdateAll;
                        match event{
                            ServerEvent::CommunicationServerData(id, data, data_scope) =>{
                                match data_scope{
                                    DataScope::UpdateAll =>{
                                        conditional_data_scope = DataScope::UpdateAll;
                                        self.comm_servers_data.insert(id, data);
                                        self.updating_nodes.remove(&id);
                                    },
                                    DataScope::UpdateSelf =>{
                                        conditional_data_scope = DataScope::UpdateSelf;
                                        self.comm_servers_data.insert(id, data.clone());
                                        let json_string = serde_json::to_string(&data).unwrap();
                                        debug!("Sent json data  with scope UpdateSelf {:?}", json_string);
                                        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
                                    },
                                }
                            }
                            ServerEvent::TextServerData(id, data, data_scope) =>{
                                match data_scope{
                                    DataScope::UpdateAll =>{
                                        conditional_data_scope = DataScope::UpdateAll;
                                        self.text_servers_data.insert(id, data);
                                        self.updating_nodes.remove(&id);
                                    },
                                    DataScope::UpdateSelf =>{
                                        conditional_data_scope = DataScope::UpdateSelf;
                                        self.text_servers_data.insert(id, data.clone());
                                        let json_string = serde_json::to_string(&data).unwrap();
                                        debug!("Sent json data {:?} with scope UpdateSelf", json_string);
                                        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
                                    },
                                }

                            },
                            ServerEvent::MediaServerData(id, data, data_scope) =>{
                                match data_scope{
                                    DataScope::UpdateAll =>{
                                        conditional_data_scope = DataScope::UpdateAll;
                                        self.media_servers_data.insert(id, data);
                                        self.updating_nodes.remove(&id);
                                    },
                                    DataScope::UpdateSelf =>{
                                        conditional_data_scope = DataScope::UpdateSelf;
                                        self.media_servers_data.insert(id, data.clone());
                                        let json_string = serde_json::to_string(&data).unwrap();
                                        debug!("Sent json data with scope UpdateSelf {:?} ", json_string);
                                        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
                                    },
                                }

                            },
                            ServerEvent::CallTechniciansToFixDrone(id, sender) => {
                                self.fix_drone(id, sender, sender_to_gui.clone());
                            }
                            ServerEvent::ControllerShortcut(packet) => {
                                self.send_shortcut(packet);
                            },
                        }

                        if self.updating_nodes.is_empty() && conditional_data_scope == DataScope::UpdateAll {
                            self.send_display_data(sender_to_gui.clone());
                            self.updating_nodes = edge_nodes.clone();
                            //eprintln!("updating_node: {:?}", self.updating_nodes);
                        }
                    }
                },

            }
        }
    }
}

impl SimulationController {
    fn handle_ws_command(&mut self, command: WsCommand, sender_to_gui: &Sender<String>) {
        match command {
            WsCommand::WsUpdateData => {
                //println!("CONTROLLER RECEIVED NETWORK DATA UPDATE COMMAND");
                // Update data from the simulation controller
                let clients: Vec<NodeId> = self.command_senders_clients.keys().cloned().collect();
                let servers: Vec<NodeId> = self.command_senders_servers.keys().cloned().collect();

                // Ask every client to update its data
                for client in clients {
                    if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client).cloned() {
                        sender_to_client
                            .send(ClientCommand::UpdateMonitoringData)
                            .expect("error in sending monitoring data to the websocket");
                    }
                }

                // Ask every server to update its data
                for server in servers {
                    if let Some((sender_to_server, _)) = self.command_senders_servers.get(&server).cloned() {
                        sender_to_server
                            .send(ServerCommand::UpdateMonitoringData)
                            .expect("error in sending monitoring data to the websocket");
                    }
                }
            }

            WsCommand::WsAskFileList { client_id, server_id } => {
                println!("CONTROLLER PROCESSING ASK FILE LIST COMMAND");
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned() {
                    sender_to_client
                        .send(ClientCommand::RequestListFile(server_id))
                        .expect("error in sending ask file list to the websocket");
                    println!("CONTROLLER SENT ASK FILE LIST COMMAND");
                }
            }

            WsCommand::WsAskFileContent { client_id, server_id, file_ref } => {
                println!("CONTROLLER RECEIVED ASK FILE CONTENT COMMAND");
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned() {
                    sender_to_client
                        .send(ClientCommand::RequestText(server_id, file_ref))
                        .expect("error in sending ask file content to the websocket");
                }
            }

            WsCommand::WsAskMedia { client_id, media_ref } => {
                println!("CONTROLLER RECEIVED ASK FILE MEDIA COMMAND");
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned() {
                    sender_to_client
                        .send(ClientCommand::RequestMedia(media_ref))
                        .expect("error in sending ask media to the websocket");
                }
            }

            WsCommand::WsSendMessage {
                source_client_id,
                dest_client_id,
                message,
            } => {
                println!("CONTROLLER RECEIVED SEND MESSAGE COMMAND");
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&source_client_id).cloned() {
                    sender_to_client
                        .send(ClientCommand::SendMessageTo(dest_client_id, message))
                        .expect("error in sending message to the websocket");
                }
            }

            WsCommand::WsAskListRegisteredClientsToServer { client_id, server_id } => {
                println!("CONTROLLER RECEIVED ASK REGISTERED CLIENTS IN THE SERVER COMMAND");
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned() {
                    sender_to_client
                        .send(ClientCommand::AskListClients(server_id))
                        .expect("error in sending register to the websocket");
                }
            }
            WsCommand::WsCrashDrone {
                drone_id
            } => {
                match self.request_drone_crash(drone_id, sender_to_gui){
                    Ok(_) => {
                        println!("*********************************\n\
                        drone crashed\n\
                        *********************************");
                    }
                    Err(_) => {
                        println!("<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<\n\
                        couldn't crash drone\n\
                        <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<");
                    }
                }
            }
        }
    }
}
use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use crossbeam_channel::{select_biased, Sender};
use log::{debug, info};
use tungstenite::client;
use wg_2024::controller::DroneCommand;
use crate::clients::client_chen::{NodeId, Serialize};
use crate::simulation_controller::SimulationController;
use crate::ui_traits::{SimulationControllerMonitoring};
use crate::websocket::{WsCommand};
use crate::general_use::{ClientCommand, ClientEvent, DataScope, DisplayDataChatClient, DisplayDataCommunicationServer, DisplayDataMediaServer, DisplayDataSimulationController, DisplayDataTextServer, DisplayDataWebBrowser, ServerCommand, ServerEvent};
use crate::network_initializer::DroneBrand;

impl SimulationControllerMonitoring for SimulationController {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>, data_scope: DataScope) {
        let display_data = DisplayDataSimulationController{
            data_title: "Network Data".to_string(),
            web_clients_data: self.web_clients_data.clone(),
            chat_clients_data: self.chat_clients_data.clone(),
            comm_servers_data: self.comm_servers_data.clone(),
            text_servers_data: self.text_servers_data.clone(),
            media_servers_data: self.media_servers_data.clone(),
            drones_data: self.drones_data.clone(),
            topology: self.state.topology.clone(),
        };
        let json_string = serde_json::to_string(&display_data).unwrap();
        info!("Controller has sent the data of all the nodes {:?}", display_data);
        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
    }

    fn run_with_monitoring(&mut self, sender_to_gui: Sender<String>) {
        ///Reminder: I put here the edge_nodes because I'm assuming the clients and the server must be fixed
        ///created from the network initializer
        let mut edge_nodes = self.command_senders_clients.keys().cloned().collect::<HashSet<NodeId>>();
        edge_nodes.extend(self.command_senders_servers.keys().cloned().collect::<HashSet<NodeId>>());

        self.updating_nodes = edge_nodes.clone();
        loop {
            select_biased! {
                recv(self.ws_command_receiver) -> command_res => {
                    debug!("Controller received command {:?}", command_res);
                    if let Ok(command) = command_res {
                        self.handle_ws_command(sender_to_gui.clone(), command);
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
                            _ =>{}
                        }
                        if self.updating_nodes.is_empty() && conditional_data_scope == DataScope::UpdateAll {
                            self.send_display_data(sender_to_gui.clone(), DataScope::UpdateAll);
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
                            _=> {},
                        }

                        if self.updating_nodes.is_empty() && conditional_data_scope == DataScope::UpdateAll {
                            self.send_display_data(sender_to_gui.clone(), DataScope::UpdateAll);
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
    fn handle_ws_command(&mut self, sender_to_gui: Sender<String>, command: WsCommand) {
        match command {
            WsCommand::WsUpdateData=> {
                // Update data from the simulation controller
                let clients: Vec<NodeId> = self.command_senders_clients.keys().cloned().collect();
                let servers: Vec<NodeId> = self.command_senders_servers.keys().cloned().collect();

                // Ask every client to update its data
                for client in clients{
                    if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client).cloned(){
                        sender_to_client.send(ClientCommand::UpdateMonitoringData).expect("error in sending monitoring data to the websocket");
                    }
                }

                // Ask every server to update its data
                for server in servers{
                    if let Some((sender_to_server, _)) = self.command_senders_servers.get(&server).cloned(){
                        sender_to_server.send(ServerCommand::UpdateMonitoringData).expect("error in sending monitoring data to the websocket");
                    }
                }
            },  //in general, it asks all the nodes to send the data to the monitor

            WsCommand::WsAskFileList(client_id, server_id) => {
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned(){
                    sender_to_client.send(ClientCommand::AskListClients(server_id)).expect("error in sending asks data to the websocket");
                }
            },
            WsCommand::WsAskFileContent(client_id, server_id, file_ref) => {
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned(){
                    sender_to_client.send(ClientCommand::RequestText(server_id, file_ref)).expect("error in sending asks data to the websocket");
                }
            },
            WsCommand::WsAskMedia(client_id, media_ref ) => {
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned(){
                    sender_to_client.send(ClientCommand::RequestMedia(media_ref)).expect("error in sending media data to the websocket");
                }
            }
            WsCommand::WsSendMessage(source_id, destination_id, message) =>{
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&source_id).cloned(){
                    sender_to_client.send(ClientCommand::SendMessageTo(destination_id, message)).expect("error in sending message to the websocket");
                }
            }
            WsCommand::WsAskListRegisteredClientsToServer(client_id, server_id) => {
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned(){
                    sender_to_client.send(ClientCommand::RegisterToServer(server_id)).expect("error in sending register to the websocket");
                }
            }
            _=>{}
        }
    }

}
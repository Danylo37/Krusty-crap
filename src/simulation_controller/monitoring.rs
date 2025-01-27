use std::collections::{HashMap, HashSet};
use crossbeam_channel::{select_biased, Sender};
use log::info;
use wg_2024::controller::DroneCommand;
use crate::clients::client_chen::{NodeId, Serialize};
use crate::simulation_controller::SimulationController;
use crate::ui_traits::Monitoring;
use crate::websocket::{ClientCommandWs, DroneCommandWs, ServerCommandWs, WsCommand};
use crate::general_use::{ClientCommand, ClientEvent, DisplayDataChatClient, DisplayDataCommunicationServer, DisplayDataMediaServer, DisplayDataTextServer, DisplayDataWebBrowser, ServerCommand, ServerEvent};

//todo! send also the drone specific data (e.g. pdr, status: Crashed or NotCrashed, ...)
#[derive(Debug, Serialize)]
pub struct DisplayDataSimulationController{
    //drones
    pub data_title: String,
    pub web_clients_data: HashMap<NodeId, DisplayDataWebBrowser>,
    pub chat_clients_data: HashMap<NodeId, DisplayDataChatClient>,
    pub comm_servers_data: HashMap<NodeId, DisplayDataCommunicationServer>,
    pub text_servers_data: HashMap<NodeId, DisplayDataTextServer>,
    pub media_servers_data: HashMap<NodeId, DisplayDataMediaServer>,
    pub drones: Vec<NodeId>,
    pub topology: HashMap<NodeId, Vec<NodeId>>,
}
impl Monitoring for SimulationController {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>) {
        let display_data = DisplayDataSimulationController{
            data_title: "Network Data".to_string(),
            web_clients_data: self.web_clients_data.clone(),
            chat_clients_data: self.chat_clients_data.clone(),
            comm_servers_data: self.comm_servers_data.clone(),
            text_servers_data: self.text_servers_data.clone(),
            media_servers_data: self.media_servers_data.clone(),
            drones: self.command_senders_drones.keys().cloned().collect(),
            topology: self.state.topology.clone(),
        };
        let json_string = serde_json::to_string(&display_data).unwrap();
        eprintln!("Sent json data {:?}", json_string);
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
                    eprintln!("Controller received command {:?}", command_res);
                    if let Ok(command) = command_res {
                        self.handle_ws_command(sender_to_gui.clone(), command);
                    }
                },
                recv(self.client_event_receiver) -> client_event => {
                    eprintln!("Controller received client event");
                    if let Ok(event) = client_event {
                        match event{
                            ClientEvent::WebClientData(id, data) => {
                                self.web_clients_data.insert(id, data);
                                self.updating_nodes.remove(&id);
                            },
                            ClientEvent::ChatClientData(id, data) => {
                                self.chat_clients_data.insert(id, data);
                                self.updating_nodes.remove(&id);
                            },
                            _ =>{}
                        }
                        if self.updating_nodes.is_empty() {
                            self.send_display_data(sender_to_gui.clone());
                            self.updating_nodes = edge_nodes.clone();
                            eprintln!("updating_node: {:?}", self.updating_nodes);
                        }
                    }
                },
                recv(self.server_event_receiver) -> server_event => {
                    eprintln!("Controller received server event");
                    if let Ok(event) = server_event {
                        match event{
                            ServerEvent::CommunicationServerData(id, data) =>{
                                self.comm_servers_data.insert(id, data);
                                self.updating_nodes.remove(&id);
                            }
                            ServerEvent::TextServerData(id, data) =>{
                                self.text_servers_data.insert(id, data);
                                self.updating_nodes.remove(&id);
                            },
                            ServerEvent::MediaServerData(id, data) =>{
                                self.media_servers_data.insert(id, data);
                                self.updating_nodes.remove(&id);
                            },
                            _=> {},
                        }
                        if self.updating_nodes.is_empty() {
                            self.send_display_data(sender_to_gui.clone());
                            self.updating_nodes = edge_nodes.clone();
                            eprintln!("updating_node: {:?}", self.updating_nodes);
                        }
                    }
                }
            }
        }
    }
}

impl SimulationController {
    fn handle_ws_command(&mut self, sender_to_gui: Sender<String>, command: WsCommand) {
        match command {
            WsCommand::WsUpdateData=> {
                eprintln!("Now I handle the updating data");
                // Update data from the simulation controller
                self.send_display_data(sender_to_gui.clone());
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
            WsCommand::WsDroneCommand(drone_id, drone_command) => {
                if let Some(sender_to_drone) = self.command_senders_drones.get(&drone_id).cloned(){
                    match drone_command{
                        DroneCommandWs::Crash => {
                            sender_to_drone.send(DroneCommand::Crash).expect("error in sending drone command to the drone");
                        }
                        _ => {
                            //todo()! other commands still to implement
                        }
                    }

                }
            },
            WsCommand::WsClientCommand(client_id ,client_command) => {
                if let Some((sender_to_client, _)) = self.command_senders_clients.get(&client_id).cloned(){

                    match client_command{
                        ClientCommandWs::UpdateMonitoringData => {
                            sender_to_client.send(ClientCommand::UpdateMonitoringData).expect("error in sending client command to the client");
                        }
                        _ => {
                            //todo()! other commands still to implement
                        }
                    }
                }
            },
            WsCommand::WsServerCommand(server_id, server_command) => {
                if let Some((sender_to_server, _)) = self.command_senders_servers.get(&server_id).cloned() {
                    match server_command {
                        ServerCommandWs::UpdateMonitoringData => {
                            sender_to_server.send(ServerCommand::UpdateMonitoringData).expect("error in sending client command to the client");
                        }
                        _ => {
                            //todo()! other commands still to implement
                        }
                    }
                }
            },
        }
    }

}
use std::collections::{HashMap, HashSet};
use crossbeam_channel::{select_biased, Sender};
use log::info;
use wg_2024::controller::DroneCommand;
use crate::clients::client_chen::{NodeId, Serialize};
use crate::simulation_controller::SimulationController;
use crate::ui_traits::Monitoring;
use crate::websocket::{ClientCommandWs, DroneCommandWs, ServerCommandWs, WsCommand};
use crate::general_use::{ClientCommand, ServerCommand};


#[derive(Debug, Serialize)]
struct DisplayDataSimulationController{
    //drones
    node_type: String,
    drones: Vec<NodeId>,
    clients: Vec<NodeId>,
    servers: Vec<NodeId>,
    topology: HashMap<NodeId, Vec<NodeId>>
}
impl Monitoring for SimulationController {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>) {
        let display_data = DisplayDataSimulationController{
            node_type: "SimulationController".to_string(),
            drones: self.command_senders_drones.keys().cloned().collect(),
            clients: self.command_senders_clients.keys().cloned().collect(),
            servers: self.command_senders_servers.keys().cloned().collect(),
            topology: self.state.topology.clone(),
        };
        let json_string = serde_json::to_string(&display_data).unwrap();
        sender_to_gui.send(json_string).expect("error in sending displaying data to the websocket");
    }

    fn run_with_monitoring(&mut self, sender_to_gui: Sender<String>) {
        loop {
            select_biased! {
                recv(self.ws_command_receiver) -> command_res => {
                    eprintln!("Controller received command {:?}", command_res);
                    if let Ok(command) = command_res {
                        self.handle_ws_command(sender_to_gui.clone(), command);
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
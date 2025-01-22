use crate::general_use::{ClientId, FloodId, Message, ServerId, ServerType, SessionId};
use crate::ui_traits::{crossbeam_to_tokio_bridge, Monitoring};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use crossbeam_channel::Sender;
use futures_util::select_biased;
use tokio::time::interval;
use wg_2024::network::NodeId;
use super::ChatClientDanylo;

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
    registered_communication_servers: HashMap<ServerId, bool>,
    available_clients: HashMap<ServerId, Vec<ClientId>>,

    // Inbox
    received_messages: Vec<(ClientId, Message)>,
}


impl Monitoring for ChatClientDanylo {
    fn send_display_data(&mut self, sender_to_gui: Sender<String>) {
        let display_data = ChatClientDisplayData {
            node_id: self.id,
            node_type: "ChatClientDanylo".to_string(),
            flood_ids: self.flood_ids.clone(),
            session_ids: self.session_ids.clone(),
            neighbours: self.packet_send.keys().cloned().collect(),
            discovered_servers: self.servers.clone(),
            registered_communication_servers: self.is_registered.clone(),
            available_clients: self.clients.clone(),
            received_messages: self.inbox.clone(),
        };

        // Serialize the DisplayData to MessagePack binary
        let json_string = serde_json::to_string(&display_data).unwrap();
        let _ = sender_to_gui.send(json_string).is_err();
    }
    fn run_with_monitoring(
        &mut self,
        sender_to_gui: Sender<String>,
    ) {
            loop {
                select_biased! {
                    // Handle incoming packets from the tokio mpsc channel
                    packet_res = packet_tokio_rx.recv() => {
                        if let Some(packet) = packet_res {
                            self.handle_packet(packet);
                        } else {
                            //eprintln!("Error receiving packet");
                        }
                    },
                    // Handle controller commands from the tokio mpsc channel
                    command_res = controller_tokio_rx.recv() => {
                        if let Some(command) = command_res {
                            self.handle_command(command);
                        } else {
                            //eprintln!("Error receiving controller command");
                        }
                    },

                }
            }
        }
    }
}
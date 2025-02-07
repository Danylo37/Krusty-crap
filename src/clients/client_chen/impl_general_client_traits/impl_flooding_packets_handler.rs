use crate::clients::client_chen::{ClientChen, ClientInformation, DroneInformation, FloodingPacketsHandler, NodeInfo, Router, Sending, ServerInformation, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::general_client_traits::*;

impl FloodingPacketsHandler for ClientChen {
    fn handle_flood_request(&mut self, _packet: Packet, request: &mut FloodRequest) {
        //println!("{:?} Client {} has received flood request that contains the path: {:?}", self.metadata.client_type ,self.metadata.node_id , request.path_trace);
        // Prepare the flood response.
        self.status.session_id += 1;
        request.path_trace.push((self.metadata.node_id, self.metadata.node_type));
        let mut response = request.generate_response(self.status.session_id);

        //you send directly because the source routing header is there
        self.send(response.clone());
    }

    /// When you receive a flood response, you need first to update the topology with the elements of the path_traces
    /// everyone's connected_node_ids (using the hashset's methods).
    fn handle_flood_response(&mut self, _packet: Packet, response: &FloodResponse) {
        // Debugging: Print the received path trace
        //println!("{:?} Client {} has received flood response with the path: {:?}", self.metadata.client_type ,self.metadata.node_id , response.path_trace);
        // Check if path_trace is empty
        if response.path_trace.is_empty() {
            info!("ERROR: path_trace is empty!");
            return;
        }

        // Ensure the flood_id matches
        if response.flood_id != self.status.flood_id {
            return;
        }

        // Update the network topology
        let mut path_iter = response.path_trace.iter().peekable(); //we make the vector peekable
        let mut previous_node: Option<NodeId> = None;

        while let Some(&(node_id, node_type)) = path_iter.next() {
            // Peek the next node in the path_trace (use the item without consuming it)
            let next_node = path_iter.peek().map(|&(next_id, _)| next_id);
            // Ensure entry exists for the node, so create a raw one when it is not created for the node
            let entry = self.network_info.topology.entry(node_id).or_insert_with(|| {
                match node_type {
                    NodeType::Server => {
                        NodeInfo {
                            node_id,
                            specific_info: SpecificInfo::ServerInfo(ServerInformation {
                                server_type: ServerType::Undefined,
                                connected_nodes_ids: HashSet::new(),
                            }),
                        }
                    },
                    NodeType::Client => NodeInfo {
                        node_id,
                        specific_info: SpecificInfo::ClientInfo(ClientInformation {
                            connected_nodes_ids: HashSet::new(),
                        }),
                    },
                    NodeType::Drone => NodeInfo {
                        node_id,
                        specific_info: SpecificInfo::DroneInfo(DroneInformation {
                            connected_nodes_ids: HashSet::new(),
                        }),
                    },
                }
            });

            // Safely update connected_nodes_ids
            match &mut entry.specific_info {
                SpecificInfo::ServerInfo(server_info) => {
                    if let Some(prev) = previous_node {
                        server_info.connected_nodes_ids.insert(prev);
                    }
                    if let Some(&next) = next_node {
                        server_info.connected_nodes_ids.insert(next);
                    }
                }
                SpecificInfo::ClientInfo(client_info) => {
                    if let Some(prev) = previous_node {
                        client_info.connected_nodes_ids.insert(prev);
                    }
                    if let Some(&next) = next_node {
                        client_info.connected_nodes_ids.insert(next);
                    }
                }
                SpecificInfo::DroneInfo(drone_info) => {
                    if let Some(prev) = previous_node {
                        drone_info.connected_nodes_ids.insert(prev);
                    }
                    if let Some(&next) = next_node {
                        drone_info.connected_nodes_ids.insert(next);
                    }
                }
            }

            // Update previous_node safely
            previous_node = Some(node_id);
        }

        // Update routing table
        if let Some((destination_id, destination_type)) = response.path_trace.last().copied() {
            if destination_type == NodeType::Drone || response.flood_id != self.status.flood_id {
                return;
            }
            // Use match to call the correct update function
            match destination_type {
                NodeType::Server => {
                    println!("|{:?}| CLIENT [{}] UPDATE SERVER [{}] with the path: {:?}", self.metadata.client_type ,destination_id, self.metadata.node_id , response.path_trace);
                    self.update_routing_for_server(destination_id, response.path_trace.clone());
                }
                NodeType::Client => {
                    self.update_routing_for_client(destination_id, response.path_trace.clone());
                }
                _ => {}
            }
        }
    }

}
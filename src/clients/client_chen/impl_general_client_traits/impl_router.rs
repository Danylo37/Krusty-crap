use crate::clients::client_chen::{ClientChen, NodeInfo, PacketCreator, Router, Sending, ServerInformation, SpecificInfo};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::general_client_traits::*;
use crate::clients::client_chen::SpecificInfo::ServerInfo;
use crate::general_use::ServerType::Undefined;

impl Router for ClientChen {
    ///main method of for discovering the routing
    fn do_flooding(&mut self) {
        // New ids for the flood and new session because of the flood response packet
        self.status.flood_id += 1;
        self.status.session_id += 1;

        self.communication.routing_table.clear();
        self.network_info.topology.clear();

        // Initialize the flood request with the current flood_id, id, and node type
        let flood_request = FloodRequest::initialize(self.status.flood_id, self.metadata.node_id, NodeType::Client);

        // Prepare the packet with the current session_id and flood_request
        let packet = Packet::new_flood_request(
            SourceRoutingHeader::empty_route(),
            self.status.session_id,
            flood_request,
        );

        // Collect the connected node IDs into a temporary vector
        let connected_nodes: Vec<_> = self.communication_tools.packet_send.keys().cloned().collect();

        // Send the packet to each connected node
        for &node_id in &connected_nodes {
            self.send_packet_to_connected_node(node_id, packet.clone()); // Assuming `send_packet_to_connected_node` takes a cloned packet
        }
    }

    fn update_routing_for_server(&mut self, destination_id: NodeId, path_trace: Vec<(NodeId, NodeType)>) {
        let hops = self.get_hops_from_path_trace(path_trace);
        self.communication.routing_table.insert(destination_id, hops.clone());
        //println!("Successfully updated routing table for server {}", destination_id);
        //println!("The routing table is: {:?}", self.communication.routing_table);
        let srh = SourceRoutingHeader::initialize(hops);
        if let Some(node_info) = self.network_info.topology.get(&destination_id){
            if let ServerInfo(server_info) = &node_info.specific_info{
                if server_info.server_type == Undefined  || self.communication.routing_table.get(&destination_id).is_none(){
                    self.send_query_by_routing_header(srh, Query::AskType);
                }
            }
        }
        //println!("Successfully sent the Query::AskType to the server {}", destination_id);
    }
    fn update_routing_for_client(&mut self, destination_id: NodeId, path_trace: Vec<(NodeId, NodeType)>) {
        let hops = self.get_hops_from_path_trace(path_trace.clone());
        self.communication.routing_table.insert(destination_id, hops);
        info!("Successfully updated routing table for client {}", destination_id);
        info!("The routing table is: {:?}", self.communication.routing_table);
    }

    ///auxiliary function
    fn get_flood_response_initiator(&mut self, flood_response: FloodResponse) -> NodeId {
        flood_response.path_trace.last().map(|(id, _)| *id).unwrap()
    }

    fn update_topology_entry_for_server(&mut self, initiator_id: NodeId, server_type: ServerType) {
        match self.network_info.topology.entry(initiator_id) {
            Entry::Occupied(mut entry) => {
                let node_info = entry.get_mut();
                match &mut node_info.specific_info {
                    SpecificInfo::ServerInfo(server_info) => {
                        server_info.server_type = server_type;  // Update server type correctly
                    }
                    _ => {
                        // If the node exists but is not a ServerInfo, replace it
                        node_info.specific_info = SpecificInfo::ServerInfo(ServerInformation {
                            connected_nodes_ids: Default::default(),
                            server_type,
                        });
                    }
                }
            }
            Entry::Vacant(entry) => {
                // If the node doesn't exist, insert it as a new ServerInfo
                entry.insert(NodeInfo {
                    node_id: initiator_id,
                    specific_info: SpecificInfo::ServerInfo(ServerInformation {
                        connected_nodes_ids: Default::default(),
                        server_type,
                    }),
                });
            }
        }
    }

}




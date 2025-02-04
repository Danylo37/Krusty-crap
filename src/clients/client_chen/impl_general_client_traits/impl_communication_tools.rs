use crate::clients::client_chen::{ClientChen, SpecificInfo};
use crate::clients::client_chen::general_client_traits::*;
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::SpecificInfo::ServerInfo;

impl CommunicationTrait for ClientChen{
    fn get_discovered_servers_from_topology(&mut self) -> HashSet<ServerId> {
        self.network_info.topology.iter()
            .filter_map(|(&node_id, node_info)| {
                if let SpecificInfo::ServerInfo(_) = &node_info.specific_info {
                    Some(node_id) // Ensure node_id can be converted to ServerId
                } else {
                    None
                }
            })
            .collect()
    }

    fn get_edge_nodes_from_topology(&mut self) -> HashSet<NodeId> {
        self.network_info.topology.iter()
            .filter_map(|(&node_id, node_info)| {
                match &node_info.specific_info {
                    SpecificInfo::ServerInfo(_) | SpecificInfo::ClientInfo(_) => Some(node_id),
                    _ => None,
                }
            })
            .collect()
    }

    fn get_content_servers_from_topology(&mut self) -> HashSet<ServerId> {
        self.network_info.topology.iter()
            .filter_map(|(&node_id, node_info)| {
                if let SpecificInfo::ServerInfo(server_info) = &node_info.specific_info {
                    if matches!(server_info.server_type, ServerType::Media | ServerType::Text) {
                        Some(node_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

}
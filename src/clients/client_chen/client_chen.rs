use crate::clients::Client as TraitClient;
///--------------------------
///todo!
/// 1) maybe do a flooding to update those things when the clients starts to run.
/// 2) protocol communication between the client and simulation controller
/// 3) testing
/// 4) handle the chat messages
/// 5) web browser client traits
/// 6) implement the client trait to my client: so the connected_nodes_ids are directly derived from the packet_send
/// 7) do a node removal node function to remove in the packet senders
/// Note: when you send the packet with routing the hop_index is increased in the receiving by a drone

use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::{CommandHandler, CommunicationTrait, FragmentsHandler, PacketsReceiver, Router, Sending};
use crate::general_use::{ClientType, DataScope, DisplayDataWebBrowser, DroneId, MediaRef, SpecificNodeType};
use crate::general_use::ClientEvent::WebClientData;

#[derive(Clone)]
pub(crate) struct ClientChen {
    // Client's metadata
    pub(crate) metadata: NodeMetadata,
    // Status information
    pub(crate) status: NodeStatus,
    // Communication-related data
    pub(crate) communication: CommunicationInfo,
    // Communication tools
    pub(crate) communication_tools: CommunicationTools,
    // Storage for packets and messages
    pub(crate) storage: NodeStorage,
    // Information about the current network topology
    pub(crate) network_info: NetworkInfo,

}

impl TraitClient for ClientChen {
    fn new(
        id: NodeId,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        packet_recv: Receiver<Packet>,
        controller_send: Sender<ClientEvent>,
        controller_recv: Receiver<ClientCommand>,
    ) -> Self {

        let connected_nodes = packet_send.keys().cloned().collect();

        Self {
            // Client's metadata
            metadata: NodeMetadata {
                node_id: id,
                node_type: NodeType::Client,
                client_type: ClientType::Web,
            },

            // Status
            status: NodeStatus {
                flood_id: 0, // Initial value to be 0 for every new client
                session_id:  (id as u64) * 10u64.pow(18), //(id as u64) << 56, not human-readable but more efficient and reserves more space for the sessions for each id.
            },

            // Communication-related data
            communication: CommunicationInfo {
                connected_nodes_ids: connected_nodes,
                routing_table: HashMap::new(),
                drops_counter: HashMap::new(),
            },

            // Communication tools
            communication_tools: CommunicationTools {
                packet_send,
                packet_recv,
                controller_send,
                controller_recv,
            },

            // Storage
            storage: NodeStorage {
                //irresolute_path_traces: HashMap::new(),
                fragment_assembling_buffer: HashMap::new(),
                output_buffer: HashMap::new(),
                packets_status: HashMap::new(),
                current_list_file: Vec::new(),
                current_requested_text_file: String::new(),
                current_text_media_list: Vec::new(),
                current_chosen_media_ref: "".to_string(),
                current_received_serialized_media: Default::default(),
                current_chosen_media: String::new(),
            },


            // Network Info
            network_info: NetworkInfo{
                topology: HashMap::new(),
            },
        }
    }

    fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.communication_tools.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        self.handle_controller_command(command);

                        // Things to do after handling the command
                        self.handle_fragments_in_buffer_with_checking_status();
                        self.send_packets_in_buffer_with_checking_status();
                    }
                },
                recv(self.communication_tools.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        self.handle_received_packet(packet);
                        // Things to do after handling the command
                        self.handle_fragments_in_buffer_with_checking_status();
                        self.send_packets_in_buffer_with_checking_status();
                    }
                },
            }
        }
    }
}

impl ClientChen{
    pub(crate) fn update_connected_nodes(&mut self) {
        self.communication.connected_nodes_ids = self.communication_tools.packet_send.keys().cloned().collect();
    }

    pub(crate) fn send_display_data_simplified(&mut self, data_scope: DataScope){
            self.update_connected_nodes();
            // Create the DisplayData struct
            let display_data = DisplayDataWebBrowser {
                node_id: self.metadata.node_id,
                node_type: SpecificNodeType::WebBrowser,
                flood_id: self.status.flood_id,
                session_id: self.status.session_id,
                connected_node_ids: self.communication.connected_nodes_ids.clone(),
                routing_table: self.communication.routing_table.clone(),
                discovered_text_servers : self.get_text_servers_from_topology().clone(),
                discovered_media_servers : self.get_media_servers_from_topology().clone(),
                curr_received_file_list: self.storage.current_list_file.clone(),
                chosen_file_text: self.storage.current_requested_text_file.clone(),
                serialized_media: self.storage.current_received_serialized_media.clone(),
            };
            self.send_event(WebClientData(self.metadata.node_id, display_data, data_scope));
    }
}

// Metadata about the client
#[derive(Clone)]
pub(crate) struct NodeMetadata {
    pub(crate) node_id: NodeId,
    pub(crate) node_type: NodeType,
    pub(crate) client_type: ClientType,
}

// Status of the client
#[derive(Clone,Serialize, Deserialize)]
pub(crate) struct NodeStatus {
    pub(crate) flood_id: FloodId,
    pub(crate) session_id: SessionId,
}

// Communication-related information
#[derive(Clone)]
pub(crate) struct CommunicationInfo {
    pub(crate) connected_nodes_ids: HashSet<NodeId>,
    pub(crate) routing_table: HashMap<NodeId, Vec<NodeId>>, // Routing information per protocol
    pub(super) drops_counter: HashMap<SessionId, HashMap<DroneId, u8>>, // Counter for dropped packets
}


// Tools for communication
#[derive(Clone)]
pub(crate) struct CommunicationTools {
    pub(crate) packet_send: HashMap<NodeId, Sender<Packet>>,  // Sender for each connected node
    pub(crate) packet_recv: Receiver<Packet>,                // Unique receiver for this client
    pub(crate) controller_send: Sender<ClientEvent>,         // Sender for Simulation Controller
    pub(crate) controller_recv: Receiver<ClientCommand>,     // Receiver for Simulation Controller
}

// Storage-related data
#[derive(Clone)]
pub struct NodeStorage {
    pub(crate) fragment_assembling_buffer: HashMap<SessionId, HashMap<FragmentIndex, Packet>>, // Temporary storage for recombining fragments
    pub(crate) output_buffer: HashMap<SessionId, HashMap<FragmentIndex, Packet>>,              // Buffer for outgoing messages
    pub(crate) packets_status: HashMap<SessionId, HashMap<FragmentIndex, PacketStatus>>,       // Map every packet with the status of sending
    pub(crate) current_list_file: Vec<String>,                                  // Files received from media servers
    pub(crate) current_requested_text_file: String,
    pub(crate) current_text_media_list: Vec<MediaRef>,
    pub(crate) current_chosen_media_ref: MediaRef,
    pub(crate) current_received_serialized_media: HashMap<MediaRef, String>,
    pub(crate) current_chosen_media: String,
}



#[derive(Clone,Serialize, Deserialize)]
pub(crate) struct NetworkInfo{
    pub(crate) topology: HashMap<NodeId, NodeInfo>,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct NodeInfo{
    pub(crate) node_id: NodeId,
    pub(crate) specific_info: SpecificInfo,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpecificInfo {
    ClientInfo(ClientInformation),
    ServerInfo(ServerInformation),
    DroneInfo(DroneInformation),
}

// Manually implement Default for SpecificInfo
impl Default for SpecificInfo {
    fn default() -> Self {
        SpecificInfo::ClientInfo(ClientInformation::default())
    }
}

#[derive(Debug,Clone, Serialize, Deserialize, Default)]
pub struct ClientInformation {
    pub(crate) connected_nodes_ids: HashSet<NodeId>,
}

impl ClientInformation {
    fn new(connected_nodes_ids: HashSet<NodeId>) -> ClientInformation {
        ClientInformation {
            connected_nodes_ids,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInformation {
    pub(crate) connected_nodes_ids: HashSet<NodeId>,
    pub(crate) server_type: ServerType,
}

impl ServerInformation {
    fn new(connected_nodes_ids: HashSet<NodeId>, server_type: ServerType) -> ServerInformation {
        ServerInformation {
            connected_nodes_ids,
            server_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneInformation {
    pub(crate) connected_nodes_ids: HashSet<NodeId>,
}

impl DroneInformation {
    fn new(connected_nodes_ids: HashSet<NodeId>) -> DroneInformation {
        DroneInformation {
            connected_nodes_ids,
        }
    }
}






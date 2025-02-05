use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};

use wg_2024::{
    network::NodeId,
    packet::{Packet, NodeType},
};
use crate::network_initializer::DroneBrand;

pub type MediaRef = String;
pub type FileRef = String;
pub type ServerId = NodeId;
pub type ClientId = NodeId;
pub type DroneId = NodeId;
pub type InitiatorId = NodeId;
pub type DestinationId = NodeId;
pub type SessionId = u64;
pub type FloodId = u64;
pub type FragmentIndex = u64;
pub type UsingTimes = u64;  //to measure traffic of fragments in a path.
pub type ChatHistory = Vec<(Speaker, String)>;
pub type Node = (NodeId, NodeType);

///all the monitoring data
#[derive(Debug, Serialize, Clone)]
pub struct DisplayDataWebBrowser {
    pub node_id: NodeId,
    pub node_type: String,
    pub flood_id: FloodId,
    pub session_id: SessionId,
    pub connected_node_ids: HashSet<NodeId>,
    pub routing_table: HashMap<NodeId, Vec<NodeId>>,
    pub discovered_text_servers: HashSet<ServerId>,
    pub discovered_media_servers: HashSet<ServerId>,
    pub curr_received_file_list: Vec<String>,
    pub chosen_file_text: String,
    pub serialized_media: HashMap<MediaRef, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    from: NodeId,
    to: NodeId,
    content: String,
}

impl Message {
    pub fn new(from: NodeId, to: NodeId, content: String) -> Self {
        Self { from, to, content }
    }

    pub fn get_sender(&self) -> NodeId {
        self.from
    }

    pub fn get_recipient(&self) -> NodeId {
        self.to
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}
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
    pub drones_data: HashMap<NodeId, DisplayDataDrone>,
    pub topology: HashMap<NodeId, Vec<NodeId>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayDataDrone{
    pub(crate) node_id: NodeId,
    pub(crate) drone_brand: DroneBrand,
    pub(crate) connected_nodes_ids: Vec<NodeId>,
    pub(crate) pdr: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct DisplayDataChatClient {
    // Client metadata
    pub node_id: NodeId,
    pub node_type: String,

    // Used IDs
    pub flood_ids: Vec<FloodId>,
    pub session_ids: Vec<SessionId>,

    // Network
    pub routes: HashMap<ServerId, Vec<NodeId>>,

    // Connections
    pub neighbours: HashSet<NodeId>,
    pub discovered_servers: HashMap<ServerId, ServerType>,
    pub available_clients: HashMap<ServerId, Vec<ClientId>>,

    // Chats
    pub chats: HashMap<ClientId, ChatHistory>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayDataCommunicationServer{
    pub node_id: NodeId,
    pub node_type: String,
    pub flood_id: crate::general_use::FloodId,
    pub connected_node_ids: HashSet<NodeId>,
    pub routing_table: HashMap<NodeId, Vec<NodeId>>,
    pub registered_clients: Vec<NodeId>,
}

#[derive(Debug, Clone,  Serialize)]
pub struct DisplayDataMediaServer{
    pub node_id: NodeId,
    pub node_type: String,
    pub flood_id: crate::general_use::FloodId,
    //session_id: crate::general_use::SessionId,
    pub connected_node_ids: HashSet<NodeId>,
    pub routing_table: HashMap<NodeId, Vec<NodeId>>,
    pub media: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DisplayDataTextServer{
    pub node_id: NodeId,
    pub node_type: String,
    pub flood_id: crate::general_use::FloodId,
    //session_id: crate::general_use::SessionId,
    pub connected_node_ids: HashSet<NodeId>,
    pub routing_table: HashMap<NodeId, Vec<NodeId>>,
    pub text_files: Vec<String>,
}

///packet sending status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotSentType{
    ToBeSent,
    Dropped,
    RoutingError,
    DroneDestination,
    BeenInWrongRecipient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Speaker {
    Me,
    HimOrHer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketStatus{
    Sent,                   //Successfully sent packet, that is with ack received
    NotSent(NotSentType),   //Include the packet not successfully sent, that is nack received
    InProgress,             //When we have no ack or nack confirmation
}

/// From controller to Server
#[derive(Debug, Clone)]
pub enum ServerCommand {
    //for monitoring
    UpdateMonitoringData,

    StartFlooding,
    RemoveSender(NodeId),
    AddSender(NodeId, Sender<Packet>),
    ShortcutPacket(Packet),
}

///Server-Controller
#[derive(Debug, Clone)]
pub enum ServerEvent {
    //for monitoring
    CommunicationServerData(InitiatorId, DisplayDataCommunicationServer, DataScope),
    TextServerData(InitiatorId, DisplayDataTextServer, DataScope),
    MediaServerData(InitiatorId, DisplayDataMediaServer, DataScope),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DataScope{
    UpdateAll,
    UpdateSelf,
}


#[derive(Debug, Clone)]
/// From controller to Client
pub enum ClientCommand {
    //Controller functions
    //for monitoring
    UpdateMonitoringData,

    RemoveSender(NodeId),
    AddSender(NodeId, Sender<Packet>),
    SendMessageTo(ClientId, String),  //if you order a client to send messages to another client you can do it
    StartFlooding,
    AskTypeTo(ServerId),
    RequestListFile(ServerId),   //request the list of the file that the server has.
    RequestText(ServerId, FileRef),  //the type File is alias of String, so we are requesting a Text in the File.
    RequestMedia(ServerId, MediaRef), //the type Media is alias of String, we are requesting the content referenced by the MediaRef.
    ShortcutPacket(Packet),
    GetKnownServers,
    RegisterToServer(ServerId),
    AskListClients(ServerId),


    //commands for testing
    RequestRoutes(DestinationId),
}


#[derive(Debug, Clone)]
pub enum ClientEvent {
    //for monitoring
    ChatClientData(InitiatorId, DisplayDataChatClient, DataScope),
    WebClientData(InitiatorId, DisplayDataWebBrowser, DataScope),

    PacketSent(Packet),
    KnownServers(Vec<(NodeId, ServerType, bool)>),

}

//Queries (Client -> Server)
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Query {
    //Common-shared
    AskType,

    //To Communication Server
    RegisterClient(NodeId),
    UnregisterClient(NodeId),
    AskListClients,
    SendMessage(Message),

    //To Content Server
    //(Text)
    AskListFiles,
    AskFile(String),   //changed to File (String)
    //(Media)
    AskMedia(String), // String is the reference found in the files
}

//Server -> Client
#[derive(Deserialize, Serialize, Debug)]
pub enum Response {
    //Common-shared
    ServerType(ServerType),

    //From Communication Server
    ClientRegistered,
    MessageReceived(Message),
    ListClients(Vec<NodeId>),

    //From Content Server
    //(Text)
    ListFiles(Vec<String>),
    File(String),
    //(Media)
    Media(String),

    //General Error
    Err(String)
}

///Material
#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Hash, Eq)]
pub enum ServerType {
    Communication,
    Text,
    Media,
    Undefined,
}

impl Display for ServerType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ServerType::Communication => "Communication",
            ServerType::Text => "Text",
            ServerType::Media => "Media",
            ServerType::Undefined => "Undefined",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Hash, Serialize, Deserialize)]
pub enum ClientType {
    Chat,
    Web,
}
impl ClientType {
    // Returns an iterator over all possible client types
    pub fn iter() -> impl Iterator<Item = ClientType> {
        [
            ClientType::Chat,
            ClientType::Web,
        ]
            .into_iter()
    }
}

impl Display for ClientType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

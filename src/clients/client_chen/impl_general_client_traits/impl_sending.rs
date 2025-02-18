use crate::clients::client_chen::{ClientChen, PacketCreator, Sending};
use crate::clients::client_chen::prelude::*;
use crate::general_use::NotSentType::{RoutingError, ToBeSent};

impl Sending for ClientChen {
    fn send_packets_in_buffer_with_checking_status(&mut self) {
        //looping in the sessions, but now agreed to have only one session in the buffer.
        //so the first for is pretty useless, but it doesn't hurt the program
        let sessions: Vec<SessionId> = self.storage.output_buffer.keys().cloned().collect();
        for session_id in sessions {
            if let Some(fragments) = self.storage.output_buffer.get(&session_id) {
                let fragment_indices: Vec<_> = fragments.keys().cloned().collect();

                //now we are looping in the fragment indexes to see which packets are not sent
                for fragment_index in fragment_indices {
                    if let Some(packet) = self.storage.output_buffer
                        .get(&session_id)
                        .and_then(|fragments| fragments.get(&fragment_index))
                    {
                        if let Some(status) = self.storage.packets_status
                            .get(&session_id)
                            .and_then(|fragments| fragments.get(&fragment_index))
                        {
                            //we will manage these packets based on their sending status
                            self.packets_status_sending_actions(packet.clone(), status.clone());
                        } else {
                            println!("Missing status for session {} fragment {}", session_id, fragment_index);
                        }
                    } else {
                        println!("Missing packet in output_packet_disk for session {} fragment {}", session_id, fragment_index);
                    }
                }
            }
        }
    }

    fn send(&mut self, packet: Packet) {
        if let Some(next_hop) = packet.routing_header.next_hop() {
            self.send_packet_to_connected_node(next_hop, packet);
        } else {
            panic!("No next hop available for packet: {:?}", packet);
        }
    }

    fn send_event(&mut self, client_event: ClientEvent) {
        self.communication_tools.controller_send.send(client_event)
            .unwrap_or_else(|e| error!("Failed to send client event: {}", e));
    }

    fn send_query(&mut self, server_id: ServerId, query: Query) {
        if let Some(query_packets) = self.msg_to_fragments(query, server_id) {
            for query_packet in query_packets {
                self.send(query_packet);
            }
        } else {
            warn!("Failed to fragment query");
        }
    }

    fn send_query_by_routing_header(&mut self, source_routing_header: SourceRoutingHeader, query: Query) {
        if let Some(query_packets) = self.msg_to_fragments_by_routing_header(query, source_routing_header) {
            for query_packet in query_packets {
                self.send(query_packet);
            }
        } else {
            warn!("Failed to fragment query");
        }
    }

    fn send_packet_to_connected_node(&mut self, target_node_id: NodeId, mut packet: Packet) {
        // Store packet with proper nested structure
        let (session_id, fragment_index) = match &packet.pack_type {
            PacketType::MsgFragment(fragment) => (packet.session_id, fragment.fragment_index),
            _ => (packet.session_id, 0),
        };

        self.storage.output_buffer
            .entry(session_id)
            .or_default()
            .insert(fragment_index, packet.clone());

        packet.routing_header.increase_hop_index();
        // Attempt to send packet
        match self.communication_tools.packet_send.get_mut(&target_node_id) {
            Some(sender) => {
                match sender.send(packet.clone()) {
                    Ok(_) => {
                        info!("Successfully sent packet to {}", target_node_id);
                        match packet.pack_type{
                            PacketType::Ack(_) | PacketType::Nack(_) | PacketType::FloodResponse(_) =>{
                                self.update_packet_status(session_id, fragment_index, PacketStatus::Sent);
                                return;
                            }
                            PacketType::MsgFragment(_)=>{
                                self.update_packet_status(session_id, fragment_index, PacketStatus::InProgress);
                                return;
                            }
                            PacketType::FloodRequest(_)=>{
                                self.update_packet_status(session_id, fragment_index, PacketStatus::Sent);
                                return;
                            }
                        }

                    },
                    Err(e) => {
                        error!("Failed to send to {}: {}", target_node_id, e);
                        self.update_packet_status(session_id, fragment_index, PacketStatus::NotSent(ToBeSent));
                    }
                }
            }
            None => {
                warn!("No valid connection to {}", target_node_id);
                match packet.pack_type{
                    PacketType::Ack(_) | PacketType::Nack(_) => {
                        self.communication_tools.controller_send.send(ClientEvent::ControllerShortcut(packet)).unwrap();
                        self.update_packet_status(session_id, fragment_index, PacketStatus::Sent);
                        return
                    }
                    PacketType::FloodRequest(_) => {
                        return
                    }
                    PacketType::MsgFragment(_) | PacketType::FloodResponse(_) => {
                        self.update_packet_status(session_id, fragment_index, PacketStatus::NotSent(RoutingError(target_node_id)));
                        return
                    }
                }
            }
        }
    }

    fn packets_status_sending_actions(&mut self, packet: Packet, packet_status: PacketStatus) {
        if let Some(destination) = packet.routing_header.destination(){

        match packet_status {
            PacketStatus::NotSent(not_sent_type) => {
                self.handle_not_sent_packet(packet, not_sent_type, destination);
            }
            _ => {} // No action needed
        }
    }
}


    fn handle_not_sent_packet(&mut self, mut packet: Packet, not_sent_type: NotSentType, destination: NodeId) {
        let route = self.communication.routing_table.get(&destination);
        match not_sent_type {
            //through
            NotSentType::RoutingError(drone_id) => {
                if let Some(route) = route
                {
                    if !route.is_empty() && !route.contains(&drone_id){
                        let srh = SourceRoutingHeader::initialize(route.clone());
                        packet.routing_header = srh.clone();
                        self.send(packet);
                        println!("error routing packet sent to {}", destination);
                    }

                } else {
                    println!("No valid routes to {}", destination);
                }
            }
            NotSentType::DroneDestination => {
                let (session_id, fragment_index) = match &packet.pack_type {
                    PacketType::MsgFragment(fragment) => (packet.session_id, fragment.fragment_index),
                    _ => (packet.session_id, 0),
                };
                self.storage.output_buffer
                    .entry(session_id)
                    .and_modify(|fragments| { fragments.remove(&fragment_index); });
            }
            NotSentType::ToBeSent => {
                if let Some(route) = route {
                    if !route.is_empty() {
                        let srh = SourceRoutingHeader::initialize(route.clone());
                        packet.routing_header = srh.clone();
                        self.send(packet);
                    }
                }
            },
            NotSentType::BeenInWrongRecipient(_drone_id) => {
                if let Some(route) = route
                {
                    if !route.is_empty(){
                        let srh = SourceRoutingHeader::initialize(route.clone());
                        packet.routing_header = srh.clone();
                        self.send(packet);
                        println!("error routing packet sent to {}", destination);
                    }

                } else {
                    println!("No valid routes to {}", destination);
                }
            }
            _=>{}
        }
    }

    fn update_packet_status(&mut self, session_id: SessionId, fragment_index: FragmentIndex, status: PacketStatus) {
        //println!("packet_status updated session_id {}, fragment_index: {}, status: {:?}", session_id, fragment_index, status);
        self.storage.packets_status
            .entry(session_id)
            .or_default()
            .insert(fragment_index, status);
    }

}
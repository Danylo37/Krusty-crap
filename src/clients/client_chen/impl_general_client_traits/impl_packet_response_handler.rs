use crate::clients::client_chen::{ClientChen, PacketResponseHandler, Router, Sending};
use crate::clients::client_chen::prelude::*;
use crate::clients::client_chen::general_client_traits::*;
use crate::general_use::PacketStatus::{Sent, WaitingForFixing};

impl PacketResponseHandler for ClientChen {
    fn handle_ack(&mut self, ack_packet_session_id: SessionId, ack: &Ack) {
        /*println!(
            "\n==============================================\n\
         ✔ ACK RECEIVED\n\
         ─────────────────────────────────────────────\n\
         Session ID    : {}\n\
         Fragment Index: {}\n\
         =============================================\n",
            ack_packet_session_id, ack.fragment_index
        );*/
        let session_id = ack_packet_session_id;
        let fragment_index = ack.fragment_index;

        // Update packets_status using nested HashMap access
        self.update_packet_status(session_id, fragment_index, Sent);

        // Remove from output_buffer using proper nested structure
        if let Some(fragments) = self.storage.output_buffer.get_mut(&session_id) {
            fragments.remove(&fragment_index);
        }
    }


    fn handle_nack(&mut self, nack_packet: Packet, nack: &Nack) {
        // Handle specific NACK types
        match nack.nack_type.clone() {
            NackType::ErrorInRouting(node_id) => self.handle_error_in_routing(node_id, nack_packet.session_id, nack),
            NackType::DestinationIsDrone => self.handle_destination_is_drone(nack_packet.session_id, nack),
            NackType::Dropped => self.handle_packet_dropped(nack_packet, nack),
            NackType::UnexpectedRecipient(node_id) => self.handle_unexpected_recipient(node_id, nack_packet.session_id, nack),
        }
    }


    fn handle_error_in_routing(&mut self, node_id: NodeId, nack_packet_session_id: SessionId, nack: &Nack) {
        // Clean up packet_send connection
        if self.communication_tools.packet_send.remove(&node_id).is_some() {
            warn!("Removed broken connection to node {} from packet_send", node_id);
        }

        println!("Routing error encountered for node {}: Drone crashed or sender not found", node_id);

        let session_id = nack_packet_session_id;
        let fragment_index = nack.fragment_index;

        self.update_packet_status(
            session_id,
            fragment_index,
            PacketStatus::NotSent(NotSentType::RoutingError(node_id)),
        );

        let opt_packet = self.storage.output_buffer
            .get_mut(&session_id)
            .and_then(|fragments| fragments.get_mut(&fragment_index))
            .cloned();

        let option_packet_to_send = {
            if let Some(mut packet) = opt_packet {
                println!("DEBUGGING SESSION ID: {}", packet.session_id);
                let opt_destination = packet.routing_header.destination();
                if let Some(destination) = opt_destination {
                    let pack = match self.communication.routing_table.get(&destination) {
                        Some(routes) => {
                            // Case 1: Still the wrong path memorized
                            println!("DEBUGGING ERROR NODE IS: {} AND THE ROUTES: {:#?}", node_id, routes);
                            if routes.clone().contains(&node_id) {
                                self.do_flooding();
                                None
                            }
                            // Case 2: We have the ok path, so it returns the packet to send
                            else if !routes.is_empty() {
                                let source_routing_header = SourceRoutingHeader::initialize(routes.clone());
                                packet.routing_header = source_routing_header; // Perform the update
                                println!("DEBUGGING ERROR NODE IS: {} AND THE ROUTES: {:#?}", node_id, routes);
                                Some(packet.clone())
                            }
                            // Case 3: When the routing table doesn't contain the wrong route and is empty
                            else {
                                None
                            }
                        }

                        // No corresponding entry in the routing table
                        None => None,
                    };

                    pack  // Packet to send
                } else {
                    None // Packet to send
                }
            } else {
                warn!("Packet not found in output buffer (Session: {}, Fragment: {})", session_id, fragment_index);
                None
            }
        };

        // Send the packet when conditions are satisfied
        if let Some(p) = option_packet_to_send {
            // Notice that by sending, it will automatically update the PacketStatus
            self.send(p);
            println!("DEBUGGING PACKET SESSION ID SENT: {}", session_id);
        }
    }
    fn handle_destination_is_drone(&mut self, nack_packet_session_id: SessionId, nack: &Nack) {
        let session_id = nack_packet_session_id;
        self.update_packet_status(session_id, nack.fragment_index, PacketStatus::NotSent(NotSentType::DroneDestination));
        self.storage.output_buffer.remove(&(session_id));  //we don't want anymore this packet.
    }
    fn handle_packet_dropped(&mut self, nack_packet: Packet, nack: &Nack) {
        //println!("query packet_dropped");
        let session_id = nack_packet.session_id;

        // When the drone pdr is very high then we need to fix, we give him chance up to 10 times repeating pack drop.
        if let Some(drone) = nack_packet.routing_header.source() {
            let map = self
                .communication
                .drops_counter
                .entry(session_id)
                .or_insert_with(HashMap::new);

            let counter = map.entry(drone).or_insert(0);
            *counter += 1;

            if *counter == 10 {
                *counter = 0;
                let me = (self.metadata.node_id, NodeType::Client);
                self.send_event(ClientEvent::CallTechniciansToFixDrone(drone, me));

                self.storage
                    .packets_status
                    .entry(session_id)
                    .or_insert_with(HashMap::new)
                    .insert(nack.fragment_index, WaitingForFixing(drone));

                return;
            }
        }

        self.update_packet_status(
            session_id,
            nack.fragment_index,
            PacketStatus::NotSent(NotSentType::Dropped),
        );

        if let Some(map) = self.storage.output_buffer.get(&session_id) {
            if let Some(packet) = map.get(&nack.fragment_index) {
                // Notice that the packet status will be automatically updated
                self.send(packet.clone());
            } else {
                println!("Dropped packet not found in output buffer");
            }
        } else {
            println!("Dropped packet not found in output buffer");
        }

        /*println!(
            "\n*******************************************************************\n\
        🚁 Dropped packet resent\n\
        -------------------------------------------------------------------\n\
        📊 Packet status:\n{:#?}\n\
        *******************************************************************",
            self.storage.packets_status
        );*/
    }

    fn handle_unexpected_recipient(&mut self, node_id: NodeId, nack_packet_session_id: SessionId, nack: &Nack) {
        info!("unexpected recipient found {}", node_id);
        let session_id = nack_packet_session_id;
        let fragment_index = nack.fragment_index;

        self.update_packet_status(
            session_id,
            nack.fragment_index,
            PacketStatus::NotSent(NotSentType::BeenInWrongRecipient(node_id)));

        let opt_packet = self.storage.output_buffer
            .get_mut(&session_id)
            .and_then(|fragments| fragments.get_mut(&fragment_index))
            .cloned();

        let option_packet_to_send = {
            if let Some(mut packet) = opt_packet {
                println!("DEBUGGING SESSION ID: {}", packet.session_id);
                let opt_destination = packet.routing_header.destination();
                if let Some(destination) = opt_destination {
                    let pack = match self.communication.routing_table.get(&destination) {
                        Some(routes) => {
                            // Case 1: Still the wrong path memorized
                            println!("DEBUGGING ERROR NODE IS: {} AND THE ROUTES: {:#?}", node_id, routes);
                            if routes.clone().contains(&node_id) {
                                self.do_flooding();
                                None
                            }
                            // Case 2: We have the ok path, so it returns the packet to send
                            else if !routes.is_empty() {
                                let source_routing_header = SourceRoutingHeader::initialize(routes.clone());
                                packet.routing_header = source_routing_header; // Perform the update
                                println!("DEBUGGING ERROR NODE IS: {} AND THE ROUTES: {:#?}", node_id, routes);
                                Some(packet.clone())
                            }
                            // Case 3: When the routing table doesn't contain the wrong route and is empty
                            else {
                                None
                            }
                        }

                        // No corresponding entry in the routing table
                        None => None,
                    };

                    pack  // Packet to send
                } else {
                    None // Packet to send
                }
            } else {
                warn!("Packet not found in output buffer (Session: {}, Fragment: {})", session_id, fragment_index);
                None
            }
        };

        // Send the packet when conditions are satisfied
        if let Some(p) = option_packet_to_send {
            // Notice that by sending, it will automatically update the PacketStatus
            self.send(p);
            println!("DEBUGGING PACKET SESSION ID SENT: {}", session_id);
        }
    }
}
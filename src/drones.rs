use crossbeam_channel::{select_biased, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use rand::Rng;

use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet, PacketType},
};

pub struct KrustyCrapDrone {
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    pdr: f32,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    floods: HashMap<NodeId, HashSet<u64>>,
    crashing_behavior: bool,
}

impl Drone for KrustyCrapDrone {
    fn new(
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
    ) -> Self {
        Self {
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
            floods: HashMap::new(),
            crashing_behavior: false,
        }
    }

    fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.controller_recv) -> command => {
                    if let Ok(command) = command {
                        self.handle_command(command);
                    }
                }
                recv(self.packet_recv) -> packet => {
                    if let Ok(packet) = packet {
                        self.handle_packet(packet);
                    }
                },
            }
            if self.crashing_behavior && self.packet_recv.is_empty() {
                break;
            }
        }
    }
}

impl KrustyCrapDrone {
    fn handle_packet(&mut self, packet: Packet) {
        match packet.pack_type {
            PacketType::Nack(nack) => self.handle_nack(nack, packet.routing_header, packet.session_id),
            PacketType::Ack(ack) => self.handle_ack(ack, packet.routing_header, packet.session_id),
            PacketType::MsgFragment(fragment) => self.handle_fragment(fragment, packet.routing_header, packet.session_id),
            PacketType::FloodRequest(flood_request) => self.handle_flood_request(flood_request, packet.session_id),
            PacketType::FloodResponse(flood_response) => self.handle_flood_response(flood_response, packet.routing_header, packet.session_id),
        }
    }

    fn handle_command(&mut self, command: DroneCommand) {
        match command {
            DroneCommand::AddSender(id, sender) => self.add_sender(id, sender),
            DroneCommand::RemoveSender(id) => self.remove_sender(id),
            DroneCommand::SetPacketDropRate(pdr) => self.pdr = pdr,
            DroneCommand::Crash => self.crashing_behavior = true,
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.insert(id, sender);
    }

    fn remove_sender(&mut self, id: NodeId) {
        self.packet_send.remove(&id);
    }

    fn handle_nack(
        &mut self,
        nack: Nack,
        mut routing_header: SourceRoutingHeader,
        session_id: u64)
    {
        // Increment the hop index in the routing header to reflect progress through the route
        routing_header.increase_hop_index();

        // Create a new Nack packet using the updated routing header
        let packet = Packet::new_nack(routing_header, session_id, nack);

        // Send the Nack packet to the next hop in the route
        self.send_to_next_hop(packet);
    }

    fn handle_ack(
        &mut self,
        ack: Ack,
        mut routing_header: SourceRoutingHeader,
        session_id: u64)
    {
        // Increment the hop index in the routing header to reflect progress through the route
        routing_header.increase_hop_index();

        // Create a new Ack packet using the updated routing header
        let packet = Packet::new_ack(routing_header, session_id, ack.fragment_index);

        // Send the Ack packet to the next hop in the route
        self.send_to_next_hop(packet);
    }

    fn handle_fragment(
        &mut self,
        fragment: Fragment,
        mut routing_header:
        SourceRoutingHeader,
        session_id: u64)
    {
        // Check if the drone is in a crashing state
        // If so, send a Nack 'ErrorInRouting' with 'self.id'
        if self.crashing_behavior {
            self.send_nack(NackType::ErrorInRouting(self.id), fragment.fragment_index, routing_header, session_id);
            return;
        }

        // Retrieve the current hop from the routing header
        // If it doesn't exist, send a Nack 'UnexpectedRecipient' with 'self.id'
        let Some(current_hop_id) = routing_header.current_hop() else {
            self.send_nack(NackType::UnexpectedRecipient(self.id), fragment.fragment_index, routing_header, session_id);
            return;
        };
        // If the current hop isn't the drone's ID, send a Nack 'UnexpectedRecipient' with 'self.id'
        if self.id != current_hop_id {
            self.send_nack(NackType::UnexpectedRecipient(self.id), fragment.fragment_index, routing_header, session_id);
            return;
        }

        // Retrieve the next hop from the routing header.
        // If it doesn't exist, send a Nack 'DestinationIsDrone'
        let Some(next_hop_id) = routing_header.next_hop() else {
            self.send_nack(NackType::DestinationIsDrone, fragment.fragment_index, routing_header, session_id);
            return;
        };

        // Attempt to find the sender for the next hop
        // If the sender isn't found, send a Nack 'ErrorInRouting' with next_hop_id
        let Some(sender) = self.packet_send.get(&next_hop_id) else {
            self.send_nack(NackType::ErrorInRouting(next_hop_id), fragment.fragment_index, routing_header, session_id);
            return;
        };

        // Increment the hop index in the routing header to reflect progress through the route
        routing_header.increase_hop_index();

        // Create a new Fragment packet using the updated routing header, session ID and fragment
        let packet = Packet::new_fragment(routing_header.clone(), session_id, fragment.clone());

        // Simulate packet drop based on the PDR
        // If the random number is less than PDR, drop the packet (send a Nack 'Dropped')
        // And send the 'PacketDropped' event to the simulation controller
        if rand::rng().random_range(0.0..1.0) < self.pdr {
            self.send_nack(NackType::Dropped, fragment.fragment_index, routing_header, session_id);
            self.send_event(DroneEvent::PacketDropped(packet));
            return;
        }

        // Attempt to send the updated fragment packet to the next hop
        // If there is an error, send the packet through the simulation controller
        if sender.send(packet.clone()).is_err() {
            self.send_through_controller(packet.clone());
            return;
        }

        // Send the 'PacketSent' event to the simulation controller
        self.send_event(DroneEvent::PacketSent(packet));
    }

    fn send_nack(&self, nack_type: NackType, fragment_index: u64, mut routing_header: SourceRoutingHeader, session_id: u64) {
        // Create a Nack
        let nack = Nack {
            fragment_index,
            nack_type,
        };

        // Truncate the hops in the routing header up to the current hop index + 1
        // This effectively shortens the route, as we're sending the Nack back along the path
        routing_header.hops.truncate(routing_header.hop_index + 1);
        // Reverse the routing header to indicate the Nack should go backward in the route
        routing_header.reverse();
        // Reset the hop index to 1
        routing_header.hop_index = 1;

        // Create a Nack packet
        let nack_packet = Packet::new_nack(routing_header, session_id, nack);

        // Send the packet to the next hop
        self.send_to_next_hop(nack_packet.clone());
    }

    fn handle_flood_request(&mut self, mut flood_request: FloodRequest, session_id: u64) {
        // Check if the drone is in a crashing state
        // If so, just return
        if self.crashing_behavior {
            return;
        }

        // Add current drone to the flood request's path trace
        flood_request.increment(self.id, NodeType::Drone);

        let flood_id = flood_request.flood_id;
        let initiator_id = flood_request.initiator_id;

        // Flood ID has already been received from this flood initiator
        if self.floods.contains_key(&initiator_id) &&
            self.floods.get(&initiator_id).unwrap().contains(&flood_id) {
            // Generate and send the flood response
            let response = flood_request.generate_response(session_id);
            self.send_to_next_hop(response);
            return;
        }

        // Flood ID has not yet been received from this flood initiator
        if !self.floods.contains_key(&initiator_id) {
            self.floods.insert(initiator_id, HashSet::new());
        }
        self.floods.get(&initiator_id).unwrap().to_owned().insert(flood_id);

        // Check if there's a previous node (sender) in the flood path
        if let Some(sender_id) = self.get_prev_node_id(&flood_request.path_trace) {
            // Get all neighboring nodes except the sender
            let neighbors = self.get_neighbors_except(sender_id);

            // If there are neighbors, forward the flood request to them
            if !neighbors.is_empty() {
                self.forward_flood_request(neighbors, flood_request, session_id);
            } else {
                // If no neighbors, generate and send a response instead
                let response = flood_request.generate_response(session_id);
                self.send_to_next_hop(response);
            }
        } else {
            eprintln!("Unexpected error");
        }
    }

    fn get_prev_node_id(&self, path_trace: &Vec<(NodeId, NodeType)>) -> Option<NodeId> {
        if path_trace.len() > 1 {
            Some(path_trace[path_trace.len() - 2].0);
        }
        None
    }

    fn get_neighbors_except(&self, exclude_id: NodeId) -> Vec<&Sender<Packet>> {
        self.packet_send
            .iter()
            .filter(|(&node_id, _)| node_id != exclude_id)
            .map(|(_, sender)| sender)
            .collect()
    }

    fn forward_flood_request(
        &self,
        neighbors: Vec<&Sender<Packet>>,
        request: FloodRequest,
        session_id: u64)
    {
        // Iterate over each neighbor
        for sender in neighbors {
            // Create an empty routing header, because this is unnecessary in flood request
            let routing_header = SourceRoutingHeader::empty_route();
            // Create a new FloodRequest
            let packet = Packet::new_flood_request(routing_header, session_id, request.clone());

            // Attempt to send the updated fragment packet to the next hop.
            // If there is an error, send the packet through the simulation controller
            if sender.send(packet.clone()).is_err() {
                self.send_through_controller(packet.clone());
                return;
            }

            // Send the 'PacketSent' event to the simulation controller
            self.send_event(DroneEvent::PacketSent(packet));
        }
    }

    fn handle_flood_response(
        &mut self,
        flood_response: FloodResponse,
        mut routing_header: SourceRoutingHeader,
        session_id: u64)
    {
        // Increment the hop index in the routing header to reflect progress through the route
        routing_header.increase_hop_index();

        // Create a new FloodResponse packet with an updated routing header
        let packet = Packet::new_flood_response(routing_header.clone(), session_id, flood_response.clone());

        // Send the packet to the next hop
        self.send_to_next_hop(packet);
    }

    fn get_sender_of_next(&self, routing_header: SourceRoutingHeader) -> Option<&Sender<Packet>> {
        // Attempt to retrieve the current hop ID from the routing header
        // If it is missing, return `None` as we cannot proceed without it
        let Some(current_hop_id) = routing_header.current_hop() else {
            return None;
        };

        // Check if the current hop ID matches this drone's ID
        // If it doesn't match, return `None` because this drone is not the expected recipient
        if self.id != current_hop_id {
            return None;
        }

        // Attempt to retrieve the next hop ID from the routing header
        // If it is missing, return `None` as there is no valid destination to send the packet to
        let Some(next_hop_id) = routing_header.next_hop() else {
            return None;
        };

        // Use the next hop ID to look up the associated sender in the `packet_send` map
        // Return a reference to the sender if it exists, or `None` if not found
        self.packet_send.get(&next_hop_id)
    }

    fn send_to_next_hop(&self, packet: Packet) {
        // Attempt to find the sender for the next hop
        // If there is an error, send the packet through the simulation controller
        let Some(sender) = self.get_sender_of_next(packet.routing_header.clone()) else {
            self.send_through_controller(packet.clone());
            return;
        };

        // Attempt to send the updated fragment packet to the next hop
        // If there is an error, send the packet through the simulation controller
        if sender.send(packet.clone()).is_err() {
            self.send_through_controller(packet.clone());
            return;
        }

        // Send the 'PacketSent' event to the simulation controller
        self.send_event(DroneEvent::PacketSent(packet));
    }

    fn send_through_controller(&self, packet: Packet) {
        // Send the packet through the simulation controller
        self.controller_send.send(DroneEvent::ControllerShortcut(packet.clone())).expect("Unexpected error");
        // Send the 'PacketSent' event to the simulation controller
        self.send_event(DroneEvent::PacketSent(packet));
    }

    fn send_event(&self, event: DroneEvent) {
        match event {
            DroneEvent::PacketSent(packet) => {
                self.controller_send.send(DroneEvent::PacketSent(packet)).expect("Unexpected error");
            }
            DroneEvent::PacketDropped(packet) => {
                self.controller_send.send(DroneEvent::PacketDropped(packet)).expect("Unexpected error");
            }
            DroneEvent::ControllerShortcut(packet) => {
                self.controller_send.send(DroneEvent::ControllerShortcut(packet)).expect("Unexpected error");
            }
        }
    }
}
